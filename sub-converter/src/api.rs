use crate::emit::{Emitter, clash::ClashEmitter, sing_box::SingBoxEmitter};
use crate::error::{Error, Result};
use crate::merge::merge_subscriptions;
use crate::parse::uri::UriListParser;
use crate::parse::{Parser, clash::ClashParser, sing_box::SingBoxParser};
use crate::template::Template;
use base64::Engine;

/// Automatically detect the format of input content
pub fn detect_format(content: &str) -> Result<InputFormat> {
    let trimmed = content.trim();

    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(trimmed)
        && let Ok(decoded_str) = String::from_utf8(decoded)
        && is_uri_list_format(&decoded_str)
    {
        return Ok(InputFormat::UriListBase64);
    }

    let json_like = trimmed.trim_start().starts_with('{');
    if json_like {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if v.get("proxies").is_some() {
                return Ok(InputFormat::ClashJson);
            }
            if v.get("outbounds").is_some() {
                return Ok(InputFormat::SingBoxJson);
            }
        }
    } else {
        if trimmed.contains("proxies:") {
            return Ok(InputFormat::ClashYaml);
        }
        if trimmed.contains("outbounds:") {
            return Ok(InputFormat::SingBoxYaml);
        }
    }

    if is_uri_list_format(trimmed) {
        return Ok(InputFormat::UriList);
    }

    Err(Error::ValidationError {
        reason: "Cannot auto-detect input format, please specify manually".into(),
    })
}

fn is_uri_list_format(content: &str) -> bool {
    let lines: Vec<&str> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    !lines.is_empty() && lines.iter().all(|line| line.contains("://"))
}

#[derive(Debug, Clone, Copy)]
pub enum InputFormat {
    Auto,
    ClashYaml,
    ClashJson,
    SingBoxYaml,
    SingBoxJson,
    UriList,
    UriListBase64,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    ClashYaml,
    ClashJson,
    SingBoxYaml,
    SingBoxJson,
}

#[derive(Debug, Clone)]
pub struct InputItem {
    pub format: InputFormat,
    pub content: String,
}

pub fn convert(inputs: Vec<InputItem>, template: Template) -> Result<String> {
    let mut groups = Vec::with_capacity(inputs.len());

    for (index, item) in inputs.into_iter().enumerate() {
        let actual_content = match item.format {
            InputFormat::UriListBase64 => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(item.content.trim())
                    .map_err(|e| Error::ParseError {
                        detail: format!("base64 decode: {e}"),
                    })?;
                String::from_utf8(bytes).map_err(|e| Error::ParseError {
                    detail: format!("utf8: {e}"),
                })?
            }
            _ => item.content,
        };

        let parser: Box<dyn Parser> = match item.format {
            InputFormat::ClashYaml | InputFormat::ClashJson => Box::new(ClashParser),
            InputFormat::SingBoxYaml | InputFormat::SingBoxJson => Box::new(SingBoxParser),
            InputFormat::UriList | InputFormat::UriListBase64 | InputFormat::Auto => {
                Box::new(UriListParser)
            }
        };

        let nodes = parser
            .parse(&actual_content)
            .map_err(|e| crate::error::Error::ParseError {
                detail: format!("Input {}: {}", index + 1, e),
            })?;
        groups.push(nodes);
    }

    let subscription = merge_subscriptions(groups);

    match template.target() {
        OutputFormat::ClashYaml | OutputFormat::ClashJson => {
            ClashEmitter.emit(subscription, template)
        }
        OutputFormat::SingBoxYaml | OutputFormat::SingBoxJson => {
            SingBoxEmitter.emit(subscription, template)
        }
    }
}
