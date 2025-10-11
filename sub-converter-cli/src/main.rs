mod args;
mod source;
mod template;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use sub_converter::{InputFormat, InputItem, convert, detect_format};

use args::Args;
use source::{fetch_content, parse_source_spec};
use template::load_template;

fn main() -> Result<()> {
    let args = Args::parse();

    // Collect inputs
    let mut inputs = Vec::with_capacity(args.sources.len());

    // Process each subscription source
    for (idx, source_spec_str) in args.sources.iter().enumerate() {
        let spec = parse_source_spec(source_spec_str)?;

        eprintln!("Fetching subscription source {}: {}", idx + 1, spec.source);
        let content = fetch_content(&spec.source, args.retries)?;

        // Determine format
        let mut format = spec.format.unwrap_or(InputFormat::Auto);
        if matches!(format, InputFormat::Auto) {
            eprintln!("  Auto-detecting format...");
            format = detect_format(&content)
                .with_context(|| format!("Failed to detect format for source: {}", spec.source))?;
        }

        eprintln!("  Format: {:?}", format);

        inputs.push(InputItem { format, content });
    }

    if inputs.is_empty() {
        anyhow::bail!("No valid input sources provided");
    }

    // Build options from target and template
    let template = load_template(args.target, args.template.as_deref())?;

    // Generate output
    eprintln!("Generating configuration...");
    let output_content = convert(inputs, template).context("Failed to generate configuration")?;

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
