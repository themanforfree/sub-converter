use crate::error::{Error, Result};
use crate::ir::Node;

pub mod ss;
pub mod trojan;

pub struct UriListParser;

impl super::Parser for UriListParser {
    fn parse(&self, input: &str) -> Result<Vec<Node>> {
        let mut out = Vec::new();
        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let node_opt = parse_uri_line(line)?;
            if let Some(node) = node_opt {
                out.push(node);
            }
        }
        Ok(out)
    }
}

pub fn parse_uri_line(line: &str) -> Result<Option<Node>> {
    if line.starts_with("ss://") {
        return Ok(Some(ss::parse_ss_uri(line)?));
    }
    if line.starts_with("trojan://") {
        return Ok(Some(trojan::parse_trojan_uri(line)?));
    }
    Err(Error::Unsupported {
        what: format!("unsupported uri scheme: {}", line),
    })
}
