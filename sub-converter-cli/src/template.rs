//! Template loading and parsing.
//!
//! This module handles loading and parsing template files for
//! Clash and SingBox configurations.

use anyhow::{Context, Result};
use sub_converter::OutputFormat;
use sub_converter::formats::{ClashConfig, SingBoxConfig};
use sub_converter::template::Template;

/// Load template from target and optional template file
pub fn load_template(target: OutputFormat, template_path: Option<&str>) -> Result<Template> {
    match (target, template_path) {
        (OutputFormat::Clash, Some(path)) => load_clash_template(path),
        (OutputFormat::SingBox, Some(path)) => load_singbox_template(path),
        (OutputFormat::Clash, None) => Ok(Template::Clash(ClashConfig::default())),
        (OutputFormat::SingBox, None) => Ok(Template::SingBox(SingBoxConfig::default())),
    }
}

fn load_clash_template(path: &str) -> Result<Template> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read Clash template file: {}", path))?;

    let config: ClashConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse Clash template file: {}", path))?;

    Ok(Template::Clash(config))
}

fn load_singbox_template(path: &str) -> Result<Template> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read SingBox template file: {}", path))?;

    let config: SingBoxConfig = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse SingBox template file: {}", path))?;

    Ok(Template::SingBox(config))
}
