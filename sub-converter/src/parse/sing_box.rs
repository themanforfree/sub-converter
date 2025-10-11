use crate::error::{Error, Result};
use crate::formats::SingBoxConfig;
use crate::ir::Node;

pub struct SingBoxParser;

impl super::Parser for SingBoxParser {
    fn parse(&self, input: &str) -> Result<Vec<Node>> {
        let cfg: SingBoxConfig = serde_json::from_str(input).map_err(|e| Error::ParseError {
            detail: format!("sing-box json: {e}"),
        })?;
        Ok(cfg.outbounds.into_iter().map(Into::into).collect())
    }
}
