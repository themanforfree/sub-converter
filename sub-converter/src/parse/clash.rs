use crate::error::{Error, Result};
use crate::formats::ClashConfig;
use crate::ir::Node;

pub struct ClashParser;

impl super::Parser for ClashParser {
    fn parse(&self, input: &str) -> Result<Vec<Node>> {
        let cfg: ClashConfig = serde_yaml::from_str(input).map_err(|e| Error::ParseError {
            detail: format!("clash yaml: {e}"),
        })?;
        Ok(cfg.proxies.into_iter().map(Into::into).collect())
    }
}
