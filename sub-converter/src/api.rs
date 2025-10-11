use crate::emit::{Emitter, clash::ClashEmitter, sing_box::SingBoxEmitter};
use crate::error::{Error, Result};
use crate::merge::merge_subscriptions;
use crate::parse::uri::UriListParser;
use crate::parse::{Parser, clash::ClashParser, sing_box::SingBoxParser};
use crate::template::{ClashTemplate, SingBoxTemplate, Template};

/// Automatically detect the format of input content
pub fn detect_format(content: &str) -> Result<InputFormat> {
    let trimmed = content.trim();

    // Try to detect Clash YAML format
    if trimmed.contains("proxies:") {
        return Ok(InputFormat::Clash);
    }

    // Try to detect SingBox JSON format
    if trimmed.starts_with('{') && trimmed.contains("outbounds") {
        // Try to parse as JSON to confirm
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return Ok(InputFormat::SingBox);
        }
    }

    // Try to detect URI list format
    // Check if every non-empty line contains ://
    let lines: Vec<&str> = trimmed
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if !lines.is_empty() && lines.iter().all(|line| line.contains("://")) {
        return Ok(InputFormat::UriList);
    }

    Err(Error::ValidationError {
        reason: "Cannot auto-detect input format, please specify manually".into(),
    })
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
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub clash_template: Option<ClashTemplate>,
    pub singbox_template: Option<SingBoxTemplate>,
}

pub fn convert(inputs: Vec<InputItem>, target: OutputFormat, opt: BuildOptions) -> Result<String> {
    let mut groups = Vec::new();
    for item in inputs.into_iter() {
        let parser: Box<dyn Parser> = match item.format {
            InputFormat::Clash => Box::new(ClashParser),
            InputFormat::SingBox => Box::new(SingBoxParser),
            InputFormat::UriList => Box::new(UriListParser),
        };
        let nodes = parser.parse(&item.content)?;
        groups.push(nodes);
    }
    let sub = merge_subscriptions(groups);
    match target {
        OutputFormat::Clash => {
            let tpl = opt.clash_template.unwrap_or_default();
            ClashEmitter.emit(&sub, &Template::Clash(tpl))
        }
        OutputFormat::SingBox => {
            let tpl = opt.singbox_template.unwrap_or_default();
            SingBoxEmitter.emit(&sub, &Template::SingBox(tpl))
        }
    }
}

#[derive(Default)]
pub struct Builder {
    inputs: Vec<InputItem>,
    target: Option<OutputFormat>,
    opt: BuildOptions,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_input(
        mut self,
        format: InputFormat,
        content: impl Into<String>,
        tag: impl Into<String>,
    ) -> Self {
        self.inputs.push(InputItem {
            format,
            content: content.into(),
            tag: Some(tag.into()),
        });
        self
    }

    pub fn target(mut self, format: OutputFormat) -> Self {
        self.target = Some(format);
        self
    }

    pub fn with_clash_template(mut self, tpl: ClashTemplate) -> Self {
        self.opt.clash_template = Some(tpl);
        self
    }

    pub fn with_singbox_template(mut self, tpl: SingBoxTemplate) -> Self {
        self.opt.singbox_template = Some(tpl);
        self
    }

    pub fn build(self) -> Result<String> {
        let target = self.target.ok_or_else(|| Error::ValidationError {
            reason: "target not set".into(),
        })?;
        convert(self.inputs, target, self.opt)
    }
}
