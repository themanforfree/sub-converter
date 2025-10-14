//! Template loading and parsing.
//!
//! This module handles loading and parsing template files for
//! Clash and SingBox configurations.

use anyhow::{Context, Result};
use sub_converter::formats::{ClashConfig, SingBoxConfig};
use sub_converter::template::Template;

/// Load template from optional template file; default to empty configs
pub fn load_template(target: &str, template_path: Option<&str>) -> Result<Template> {
    match (target, template_path) {
        ("clash", Some(path)) => load_clash_template(path),
        ("singbox", Some(path)) => load_singbox_template(path),
        ("clash", None) => Ok(Template::Clash(ClashConfig::default())),
        ("singbox", None) => Ok(Template::SingBox(SingBoxConfig::default())),
        (other, _) => anyhow::bail!(
            "Unsupported target: {}, expected 'clash' or 'singbox'",
            other
        ),
    }
}

fn load_clash_template(path: &str) -> Result<Template> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read Clash template file: {}", path))?;

    // Allow YAML or JSON file for Clash; parse via YAML (serde_yaml can read YAML superset)
    let config: ClashConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse Clash template file: {}", path))?;

    Ok(Template::Clash(config))
}

fn load_singbox_template(path: &str) -> Result<Template> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read SingBox template file: {}", path))?;

    let config: SingBoxConfig = if content.trim_start().starts_with('{') {
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse SingBox JSON template: {}", path))?
    } else {
        let v: serde_json::Value = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse SingBox YAML template: {}", path))?;
        serde_json::from_value(v)
            .with_context(|| format!("Failed to convert SingBox YAML template: {}", path))?
    };

    Ok(Template::SingBox(config))
}
