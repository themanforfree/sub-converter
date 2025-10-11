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
        (OutputFormat::ClashYaml, Some(path)) => load_clash_template(path, false),
        (OutputFormat::ClashJson, Some(path)) => load_clash_template(path, true),
        (OutputFormat::SingBoxYaml, Some(path)) => load_singbox_template(path, false),
        (OutputFormat::SingBoxJson, Some(path)) => load_singbox_template(path, true),
        (OutputFormat::ClashYaml, None) => Ok(Template::ClashYaml(ClashConfig::default())),
        (OutputFormat::ClashJson, None) => Ok(Template::ClashJson(ClashConfig::default())),
        (OutputFormat::SingBoxYaml, None) => Ok(Template::SingBoxYaml(SingBoxConfig::default())),
        (OutputFormat::SingBoxJson, None) => Ok(Template::SingBoxJson(SingBoxConfig::default())),
    }
}

fn load_clash_template(path: &str, json: bool) -> Result<Template> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read Clash template file: {}", path))?;

    // Allow YAML or JSON file for Clash; parse via YAML (serde_yaml can read YAML superset)
    let config: ClashConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse Clash template file: {}", path))?;

    if json {
        Ok(Template::ClashJson(config))
    } else {
        Ok(Template::ClashYaml(config))
    }
}

fn load_singbox_template(path: &str, json: bool) -> Result<Template> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read SingBox template file: {}", path))?;

    let config: SingBoxConfig = if json {
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse SingBox JSON template: {}", path))?
    } else {
        let v: serde_json::Value = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse SingBox YAML template: {}", path))?;
        serde_json::from_value(v)
            .with_context(|| format!("Failed to convert SingBox YAML template: {}", path))?
    };

    if json {
        Ok(Template::SingBoxJson(config))
    } else {
        Ok(Template::SingBoxYaml(config))
    }
}
