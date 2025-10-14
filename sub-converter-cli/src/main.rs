mod args;
mod source;
mod template;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use sub_converter::template::Template;
use sub_converter::{OriginKind, convert};

use args::Args;
use source::fetch_content;
use template::{load_template, list_templates};

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle template listing
    if args.list_templates {
        return list_templates();
    }

    // Validate required arguments for conversion
    let target = args.target.as_ref()
        .ok_or_else(|| anyhow::anyhow!("--target is required for conversion"))?;
    let encoding = args.encoding
        .ok_or_else(|| anyhow::anyhow!("--encoding is required for conversion"))?;

    // For now CLI aggregates multiple sources by simple concatenation for UriList use-cases.
    // Future: support true multi-origin merge externally if needed.
    let mut concatenated = String::new();

    for (idx, source_spec_str) in args.sources.iter().enumerate() {
        eprintln!(
            "Fetching subscription source {}: {}",
            idx + 1,
            source_spec_str
        );
        let content = fetch_content(source_spec_str, args.retries)?;
        if !concatenated.is_empty() {
            concatenated.push('\n');
        }
        concatenated.push_str(&content);
    }

    if concatenated.trim().is_empty() {
        anyhow::bail!("No valid input sources provided");
    }

    let template = load_template(target, args.template.as_deref())?;

    eprintln!("Generating configuration...");
    let output_content = match template {
        Template::Clash(cfg) => convert(
            OriginKind::Auto,
            &concatenated,
            Template::Clash(cfg),
            encoding,
        )
        .context("Failed to generate Clash configuration")?,
        Template::SingBox(cfg) => convert(
            OriginKind::Auto,
            &concatenated,
            Template::SingBox(cfg),
            encoding,
        )
        .context("Failed to generate SingBox configuration")?,
    };

    if let Some(output_path) = &args.output {
        fs::write(output_path, &output_content)
            .with_context(|| format!("Failed to write output file: {}", output_path))?;
        eprintln!("✓ Configuration saved to: {}", output_path);
    } else {
        print!("{}", output_content);
    }

    Ok(())
}
