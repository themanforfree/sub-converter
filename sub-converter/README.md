# sub-converter (library)

Core library for converting subscription inputs (Clash YAML, SingBox JSON, and URI lists like ss://, trojan://) into Clash or SingBox configurations with optional templates.

## Public API (stable surface)

- Types
  - `InputFormat`, `OutputFormat`, `InputItem`
  - `template::Template` (`Template::Clash(ClashConfig)`, `Template::SingBox(SingBoxConfig)`)
  - `formats::{ClashConfig, SingBoxConfig}`
  - `Error`, `Result`
- Functions
  - `detect_format(content: &str) -> Result<InputFormat>`
  - `convert(inputs: Vec<InputItem>, template: Template) -> Result<String>`

Internal modules (parsing, emitting, IR) are intentionally not exposed to avoid abstraction leakage.

## Quick start

```rust
use sub_converter::{InputItem, InputFormat, OutputFormat, convert, detect_format};
use sub_converter::template::Template;
use sub_converter::formats::{ClashConfig, SingBoxConfig};

fn main() -> Result<(), sub_converter::Error> {
    let content = "ss://YWVzLTI1Ni1nY206cGFzcw@a.com:123#A\n\
                   trojan://pwd@b.com:443#B";

    let fmt = detect_format(content)?; // or specify manually
    let inputs = vec![InputItem { format: fmt, content: content.into() }];

    // Clash output
    let clash = convert(inputs.clone(), Template::Clash(ClashConfig::default()))?;

    // SingBox output
    let singbox = convert(inputs, Template::SingBox(SingBoxConfig::default()))?;

    println!("{}\n{}", clash, singbox);
    Ok(())
}
```

## Multiple inputs and merging

`convert` accepts multiple `InputItem`s (mixed formats). Items are parsed then merged in order.

```rust
let inputs = vec![
    InputItem { format: InputFormat::Clash, content: clash_yaml.into() },
    InputItem { format: InputFormat::SingBox, content: singbox_json.into() },
    InputItem { format: InputFormat::UriList, content: uri_list.into() },
];
let out = convert(inputs, Template::Clash(ClashConfig::default()))?;
```

## Templates

Use defaults via `ClashConfig::default()` / `SingBoxConfig::default()`, or build from serde:

```rust
let tpl_yaml: &str = include_str!("../../templates/clash-example.yaml");
let clash_tpl: ClashConfig = serde_yaml::from_str(tpl_yaml)?;
let out = convert(inputs, Template::Clash(clash_tpl))?;
```

## Error handling

All operations return `Result<T, Error>`, covering validation, parsing, emitting, and template errors.

## Design

- Minimal, stable façade: high-level `detect_format` + `convert`
- Strong typing for templates via `ClashConfig` / `SingBoxConfig`
- Internals remain private to prevent coupling
