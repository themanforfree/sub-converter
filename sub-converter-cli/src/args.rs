//! Command-line argument definitions and parsing.

use anyhow::{Result, anyhow};
use clap::Parser;
use sub_converter::template::OutputEncoding;

/// Subscription converter CLI tool
#[derive(Parser, Debug)]
#[command(name = "sub-converter")]
#[command(version, about = "Subscription converter supporting Clash and SingBox formats", long_about = None)]
pub struct Args {
    /// Subscription source list, format: source[:format]
    /// source can be a URL or file path
    /// format options: clash-yaml|clash-json|singbox-yaml|singbox-json|urilist|urilist-b64|auto
    #[arg(required = true)]
    pub sources: Vec<String>,

    /// Output encoding (json|yaml)
    #[arg(short, long, value_parser = parse_output_encoding)]
    pub encoding: OutputEncoding,

    /// Target type (clash|singbox). If not set, inferred from template path, else defaults to clash.
    #[arg(short, long)]
    pub target: String,

    /// Template file path
    #[arg(short = 'T', long)]
    pub template: Option<String>,

    /// Output file path (defaults to stdout)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Number of retries for network requests
    #[arg(short, long, default_value = "3")]
    pub retries: u32,
}

/// Parse output format
fn parse_output_encoding(s: &str) -> Result<OutputEncoding> {
    match s.to_lowercase().as_str() {
        "json" => Ok(OutputEncoding::Json),
        "yaml" => Ok(OutputEncoding::Yaml),
        _ => Err(anyhow!(
            "Unsupported output encoding: {}, supported: json, yaml",
            s
        )),
    }
}
