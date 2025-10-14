//! Template loading and parsing.
//!
//! This module handles loading and parsing template files for
//! Clash and SingBox configurations.

use anyhow::{Context, Result};
use sub_converter::formats::{ClashConfig, SingBoxConfig};
use sub_converter::template::Template;
use std::path::Path;

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

/// List all available templates in the templates directory
pub fn list_templates() -> Result<()> {
    // Get the executable's directory and look for a templates folder
    let exe_dir = std::env::current_exe()
        .context("Failed to get executable path")?
        .parent()
        .context("Failed to get executable directory")?
        .to_path_buf();
    
    // Try multiple possible locations for templates directory
    let possible_dirs = vec![
        Path::new("templates").to_path_buf(),  // Current directory
        exe_dir.join("templates"),              // Next to executable
        exe_dir.join("../templates"),           // One level up from exe (when in target/debug)
        exe_dir.join("../../templates"),        // Two levels up (when in target/debug/deps)
        exe_dir.join("../../../templates"),     // Three levels up (from target/debug/deps/...)
    ];

    let mut templates_dir = None;
    for dir in possible_dirs {
        if dir.exists() && dir.is_dir() {
            templates_dir = Some(dir);
            break;
        }
    }

    let templates_dir = match templates_dir {
        Some(dir) => dir,
        None => {
            println!("No templates directory found.");
            println!("Expected location: ./templates");
            return Ok(());
        }
    };

    println!("Available templates in {:?}:\n", templates_dir);

    let mut entries: Vec<_> = std::fs::read_dir(&templates_dir)
        .with_context(|| format!("Failed to read templates directory: {:?}", templates_dir))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file() && 
            (entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") ||
             entry.path().extension().and_then(|s| s.to_str()) == Some("yml") ||
             entry.path().extension().and_then(|s| s.to_str()) == Some("json"))
        })
        .collect();

    entries.sort_by_key(|entry| entry.file_name());

    if entries.is_empty() {
        println!("  (No template files found)");
        return Ok(());
    }

    for entry in entries {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        let path = entry.path();
        
        // Determine template type from filename or content
        let template_type = if file_name_str.contains("clash") {
            "Clash"
        } else if file_name_str.contains("singbox") {
            "SingBox"
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            "SingBox"
        } else {
            "Clash"
        };

        // Try to read the first line to get a description
        let description = if let Ok(content) = std::fs::read_to_string(&path) {
            content.lines()
                .filter(|line| line.starts_with('#') || line.starts_with("//"))
                .next()
                .map(|line| line.trim_start_matches('#').trim_start_matches("//").trim().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        print!("  {} ({})", file_name_str, template_type);
        if !description.is_empty() {
            print!(" - {}", description);
        }
        println!();
    }

    println!("\nUsage: sub-converter-cli -t <TARGET> -T templates/<template-file> <SOURCES>...");
    
    Ok(())
}
