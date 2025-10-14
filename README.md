# Sub Converter

A subscription converter supporting Clash and SingBox formats.

## Features

- **Multiple Input Formats**: Clash YAML, SingBox JSON, and URI lists (ss://, trojan://)
- **Automatic Format Detection**: Intelligently detects input format
- **Multiple Output Formats**: Convert to Clash or SingBox configuration
- **Template Support**: Customize output with template files
- **Multiple Subscriptions**: Merge multiple subscription sources
- **Flexible Input**: Support both URLs and local files

## Project Structure

This is a Cargo workspace containing two crates:

- **sub-converter**: Core library for parsing and converting subscriptions
- **sub-converter-cli**: Command-line interface

## Installation

```bash
cargo build --release
```

The binary will be located at `target/release/sub-converter-cli`.

## Usage

### Basic Usage

Convert a subscription to Clash format:
```bash
sub-converter-cli -t clash subscription.txt -o config.yaml
```

Convert to SingBox format:
```bash
sub-converter-cli -t singbox subscription.txt -o config.json
```

### With Templates

Use a template file to customize the output:
```bash
sub-converter-cli -t clash -T templates/clash-example.yaml subscription.txt -o config.yaml
```

### Multiple Subscriptions

Merge multiple subscription sources:
```bash
sub-converter-cli -t clash sub1.txt sub2.txt sub3.txt -o config.yaml
```

### Manual Format Specification

Specify the format for each source using `source:format` syntax:
```bash
sub-converter-cli -t clash uri-list.txt:urilist clash-config.yaml:clash -o output.yaml
```

### From URL

Download and convert subscriptions from URLs:
```bash
sub-converter-cli -t clash https://example.com/subscription -o config.yaml
```

## Supported Protocols

- Shadowsocks (ss://)
- Trojan (trojan://)

## Templates

Templates allow you to customize the output configuration. See the [templates](./templates) directory for examples.

- `templates/clash-simple.yaml`: Minimal Clash template
- `templates/clash-example.yaml`: Full-featured Clash template with rule providers
- `templates/singbox-example.json`: Basic SingBox template

To list all available templates:
```bash
sub-converter-cli --list-templates
# or use the short flag
sub-converter-cli -L
```

For more information about templates, see [templates/README.md](./templates/README.md).

## Command-Line Options

```
Usage: sub-converter-cli [OPTIONS] [SOURCES]...

Arguments:
  [SOURCES]...  Subscription source list, format: source[:format]
                source can be a URL or file path
                format options: clash, singbox, urilist

Options:
  -e, --encoding <ENCODING>    Output encoding (json|yaml)
  -t, --target <TARGET>        Output format (clash or singbox)
  -T, --template <TEMPLATE>    Template file path
  -o, --output <OUTPUT>        Output file path (defaults to stdout)
  -r, --retries <RETRIES>      Number of retries for network requests [default: 3]
  -L, --list-templates         List available templates in the templates directory
  -h, --help                   Print help
  -V, --version                Print version
```

### Retry Mechanism

The CLI includes automatic retry support for network requests:
- **Default**: 3 retries with exponential backoff (1s, 2s, 4s...)
- **Customizable**: Use `--retries` to specify the number of attempts
- **Smart retry**: Skips retry on client errors (4xx status codes)
- **Examples**:
  ```bash
  # Use default 3 retries
  sub-converter-cli -t clash https://example.com/sub
  
  # Disable retries
  sub-converter-cli -t clash --retries 1 https://example.com/sub
  
  # Increase retries for unstable networks
  sub-converter-cli -t clash --retries 5 https://example.com/sub
  ```

## Examples

### Example 0: List Available Templates

List all built-in templates:
```bash
sub-converter-cli --list-templates
```

Output:
```
Available templates in "templates":

  clash-example.yaml (Clash) - Clash Template Example
  clash-simple.yaml (Clash) - Simple Clash Template
  singbox-example.json (SingBox)

Usage: sub-converter-cli -t <TARGET> -T templates/<template-file> <SOURCES>...
```

### Example 1: Convert URI list to Clash

Input file `subscription.txt`:
```
ss://YWVzLTI1Ni1nY206cGFzc3dvcmQ@server1.com:8388#Server1
trojan://password@server2.com:443#Server2
```

Command:
```bash
sub-converter-cli -t clash subscription.txt -o config.yaml
```

### Example 2: Merge multiple sources with template

```bash
sub-converter-cli -t clash \
  https://provider1.com/sub \
  ./local-nodes.txt \
  -T templates/clash-example.yaml \
  -o config.yaml
```

### Example 3: Convert Clash to SingBox

```bash
sub-converter-cli -t singbox clash-config.yaml:clash -o singbox-config.json
```

## Development

### Run Tests

```bash
cargo nextest run
```

### Build

```bash
cargo build
```

### Run CLI in Development

```bash
cargo run --bin sub-converter-cli -- [OPTIONS]
```

## License

MIT

