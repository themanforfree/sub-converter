mod args;
mod source;
mod template;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use sub_converter::{Builder, detect_format};

use args::Args;
use source::{fetch_content, parse_source_spec};
use template::load_template;

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize Builder
    let mut builder = Builder::new();

    // Process each subscription source
    for (idx, source_spec_str) in args.sources.iter().enumerate() {
        let spec = parse_source_spec(source_spec_str)?;

        eprintln!("Fetching subscription source {}: {}", idx + 1, spec.source);
        let content = fetch_content(&spec.source, args.retries)?;

        // Determine format
        let format = if let Some(f) = spec.format {
            f
        } else {
            eprintln!("  Auto-detecting format...");
            detect_format(&content)
                .with_context(|| format!("Failed to detect format for source: {}", spec.source))?
        };

        eprintln!("  Format: {:?}", format);

        // Generate tag
        let tag = format!("source-{}", idx + 1);

        builder = builder.add_input(format, content, tag);
    }

    // Set target format
    builder = builder.target(args.target);

    // Load template (if provided)
    if let Some(template_path) = &args.template {
        eprintln!("Loading template: {}", template_path);
        let opt = load_template(template_path, args.target)?;

        if let Some(clash_tpl) = opt.clash_template {
            builder = builder.with_clash_template(clash_tpl);
        }
        if let Some(singbox_tpl) = opt.singbox_template {
            builder = builder.with_singbox_template(singbox_tpl);
        }
    }

    // Generate output
    eprintln!("Generating configuration...");
    let output_content = builder
        .build()
        .context("Failed to generate configuration")?;

    // Write output
    if let Some(output_path) = &args.output {
        fs::write(output_path, &output_content)
            .with_context(|| format!("Failed to write output file: {}", output_path))?;
        eprintln!("✓ Configuration saved to: {}", output_path);
    } else {
        // Output to stdout
        print!("{}", output_content);
    }

    Ok(())
}
