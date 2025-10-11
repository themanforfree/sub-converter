use crate::error::{Error, Result};
use crate::formats::ClashConfig;
use crate::ir::Node;

pub struct ClashParser;

impl super::Parser for ClashParser {
    fn parse(&self, input: &str) -> Result<Vec<Node>> {
        let input_trim = input.trim_start();
        let cfg: ClashConfig = if input_trim.starts_with('{') {
            let v: serde_yaml::Value =
                serde_json::from_str(input).map_err(|e| Error::ParseError {
                    detail: format!("clash json: {e}"),
                })?;
            serde_yaml::from_value(v).map_err(|e| Error::ParseError {
                detail: format!("clash json->yaml: {e}"),
            })?
        } else {
            serde_yaml::from_str(input).map_err(|e| Error::ParseError {
                detail: format!("clash yaml: {e}"),
            })?
        };
        Ok(cfg.proxies.into_iter().map(Into::into).collect())
    }
}
