mod args;
mod source;
mod template;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use sub_converter::template::Template;
use sub_converter::{OriginKind, convert};

use args::Args;
use source::{fetch_content, parse_source_spec};
use template::load_template;

fn main() -> Result<()> {
    let args = Args::parse();

    // For now CLI aggregates multiple sources by simple concatenation for UriList use-cases.
    // Future: support true multi-origin merge externally if needed.
    let mut concatenated = String::new();

    for (idx, source_spec_str) in args.sources.iter().enumerate() {
        let spec = parse_source_spec(source_spec_str)?;

        eprintln!("Fetching subscription source {}: {}", idx + 1, spec.source);
        let content = fetch_content(&spec.source, args.retries)?;
        if !concatenated.is_empty() {
            concatenated.push('\n');
        }
        concatenated.push_str(&content);
    }

    if concatenated.trim().is_empty() {
        anyhow::bail!("No valid input sources provided");
    }

    // Target selection via string (clash|singbox); for now infer from template file name or user passes correct target.
    // Here we reuse template loader with explicit target string.
    let target_str = if let Some(path) = args.template.as_deref() {
        if path.contains("sing") {
            "singbox"
        } else {
            "clash"
        }
    } else {
        // Default to clash if not specified
        "clash"
    };

    let template = load_template(target_str, args.template.as_deref())?;

    eprintln!("Generating configuration...");
    let output_content = match template {
        Template::Clash(cfg) => convert(
            OriginKind::Auto,
            &concatenated,
            Template::Clash(cfg),
            args.encoding,
        )
        .context("Failed to generate Clash configuration")?,
        Template::SingBox(cfg) => convert(
            OriginKind::Auto,
            &concatenated,
            Template::SingBox(cfg),
            args.encoding,
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
