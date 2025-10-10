//! Template loading and parsing.
//!
//! This module handles loading and parsing template files for
//! Clash and SingBox configurations.

use anyhow::{Context, Result};
use std::fs;
use sub_converter::template::{ClashTemplate, SingBoxTemplate};
use sub_converter::{BuildOptions, OutputFormat};

/// Load template file
pub fn load_template(path: &str, format: OutputFormat) -> Result<BuildOptions> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read template file: {}", path))?;

    let mut opt = BuildOptions::default();

    match format {
        OutputFormat::Clash => {
            let template: ClashTemplate = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse Clash template file: {}", path))?;
            opt.clash_template = Some(template);
        }
        OutputFormat::SingBox => {
            let template: SingBoxTemplate = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse SingBox template file: {}", path))?;
            opt.singbox_template = Some(template);
        }
    }

    Ok(opt)
}
