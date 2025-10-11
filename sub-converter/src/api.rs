use crate::emit::{Emitter, clash::ClashEmitter, sing_box::SingBoxEmitter};
use crate::error::{Error, Result};
use crate::merge::merge_subscriptions;
use crate::parse::uri::UriListParser;
use crate::parse::{Parser, clash::ClashParser, sing_box::SingBoxParser};
use crate::template::Template;

/// Automatically detect the format of input content
pub fn detect_format(content: &str) -> Result<InputFormat> {
    let trimmed = content.trim();

    // Try to detect Clash YAML format
    if is_clash_format(trimmed) {
        return Ok(InputFormat::Clash);
    }

    // Try to detect SingBox JSON format
    if is_singbox_format(trimmed) {
        return Ok(InputFormat::SingBox);
    }

    // Try to detect URI list format
    if is_uri_list_format(trimmed) {
        return Ok(InputFormat::UriList);
    }

    Err(Error::ValidationError {
        reason: "Cannot auto-detect input format, please specify manually".into(),
    })
}

fn is_clash_format(content: &str) -> bool {
    content.contains("proxies:")
}

fn is_singbox_format(content: &str) -> bool {
    if !content.starts_with('{') || !content.contains("outbounds") {
        return false;
    }
    // Try to parse as JSON to confirm
    serde_json::from_str::<serde_json::Value>(content).is_ok()
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
    Clash,
    SingBox,
    UriList,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Clash,
    SingBox,
}

#[derive(Debug, Clone)]
pub struct InputItem {
    pub format: InputFormat,
    pub content: String,
}

pub fn convert(inputs: Vec<InputItem>, template: Template) -> Result<String> {
    let mut groups = Vec::with_capacity(inputs.len());

    for (index, item) in inputs.into_iter().enumerate() {
        let parser: Box<dyn Parser> = match item.format {
            InputFormat::Clash => Box::new(ClashParser),
            InputFormat::SingBox => Box::new(SingBoxParser),
            InputFormat::UriList => Box::new(UriListParser),
        };

        let nodes = parser
            .parse(&item.content)
            .map_err(|e| crate::error::Error::ParseError {
                detail: format!("Input {}: {}", index + 1, e),
            })?;
        groups.push(nodes);
    }

    let subscription = merge_subscriptions(groups);

    match template.target() {
        OutputFormat::Clash => ClashEmitter.emit(subscription, template),
        OutputFormat::SingBox => SingBoxEmitter.emit(subscription, template),
    }
}
