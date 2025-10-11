//! Subscription source handling.
//!
//! This module provides functionality for parsing source specifications
//! and fetching content from URLs or local files.

use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::thread;
use std::time::Duration;
use sub_converter::InputFormat;

/// Source specification
pub struct SourceSpec {
    pub source: String,
    pub format: Option<InputFormat>,
}

/// Parse source specification
pub fn parse_source_spec(spec: &str) -> Result<SourceSpec> {
    // Check if contains : separator
    // Need to handle Windows paths (C:\...) and URLs (http://...)
    // Strategy: find last : and check if it's followed by a valid format name

    if let Some(pos) = spec.rfind(':') {
        let (source_part, format_part) = spec.split_at(pos);
        let format_str = &format_part[1..]; // Skip :

        // Try to parse format
        let format = match format_str.to_lowercase().as_str() {
            "auto" => Some(InputFormat::Auto),
            "clash-yaml" => Some(InputFormat::ClashYaml),
            "clash-json" => Some(InputFormat::ClashJson),
            "singbox-yaml" => Some(InputFormat::SingBoxYaml),
            "singbox-json" => Some(InputFormat::SingBoxJson),
            "urilist" => Some(InputFormat::UriList),
            "urilist-b64" => Some(InputFormat::UriListBase64),
            _ => {
                return Ok(SourceSpec {
                    source: spec.to_string(),
                    format: None,
                });
            }
        };

        Ok(SourceSpec {
            source: source_part.to_string(),
            format,
        })
    } else {
        Ok(SourceSpec {
            source: spec.to_string(),
            format: None,
        })
    }
}

/// Try to decode base64 content if it appears to be encoded
fn try_decode_base64(content: String) -> String {
    let trimmed = content.trim();

    // Skip if content looks like it's already decoded (contains :// which is common in URI lists)
    if trimmed.contains("://") {
        return content;
    }

    // Skip if content looks like YAML or JSON
    if trimmed.starts_with('{') || trimmed.contains("proxies:") || trimmed.contains("outbounds") {
        return content;
    }

    // Try to decode as base64
    if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(trimmed)
        && let Ok(decoded_str) = String::from_utf8(decoded_bytes)
        && decoded_str.contains("://")
    {
        // Decoded content looks valid (contains :// which is expected in URI lists)
        return decoded_str;
    }

    // If decoding failed or result doesn't look valid, return original content
    content
}

/// Fetch content from URL or file with retry support
pub fn fetch_content(source: &str, retries: u32) -> Result<String> {
    let content = if source.starts_with("http://") || source.starts_with("https://") {
        // Download from URL with retries
        fetch_from_url(source, retries)?
    } else {
        // Read from file (no retry needed)
        fs::read_to_string(source).with_context(|| format!("Failed to read file: {}", source))?
    };

    // Try to decode base64 if applicable for uri-list
    Ok(try_decode_base64(content))
}

/// Fetch content from URL with retry mechanism
fn fetch_from_url(url: &str, max_retries: u32) -> Result<String> {
    let mut last_error = None;

    for attempt in 1..=max_retries {
        match reqwest::blocking::get(url) {
            Ok(response) => {
                if response.status().is_success() {
                    return response
                        .text()
                        .with_context(|| format!("Failed to read URL content: {}", url));
                } else {
                    let status = response.status();
                    last_error = Some(anyhow!("HTTP request failed: {} (status: {})", url, status));

                    // Don't retry on client errors (4xx)
                    if status.is_client_error() {
                        break;
                    }
                }
            }
            Err(e) => {
                last_error = Some(anyhow!("Failed to access URL: {}, error: {}", url, e));
            }
        }

        // Don't sleep after the last attempt
        if attempt < max_retries {
            let delay = Duration::from_secs(2u64.pow(attempt - 1)); // Exponential backoff: 1s, 2s, 4s...
            eprintln!("  Retry {}/{} after {:?}...", attempt, max_retries, delay);
            thread::sleep(delay);
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("Failed to fetch from URL: {}", url)))
}
