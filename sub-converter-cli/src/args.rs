//! Command-line argument definitions and parsing.

use anyhow::{Result, anyhow};
use clap::Parser;
use sub_converter::OutputFormat;

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

    /// Output format
    #[arg(short, long, value_parser = parse_output_format)]
    pub target: OutputFormat,

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
fn parse_output_format(s: &str) -> Result<OutputFormat> {
    match s.to_lowercase().as_str() {
        "clash-yaml" => Ok(OutputFormat::ClashYaml),
        "clash-json" => Ok(OutputFormat::ClashJson),
        "singbox-yaml" => Ok(OutputFormat::SingBoxYaml),
        "singbox-json" => Ok(OutputFormat::SingBoxJson),
        _ => Err(anyhow!(
            "Unsupported output format: {}, supported: clash-yaml, clash-json, singbox-yaml, singbox-json",
            s
        )),
    }
}
