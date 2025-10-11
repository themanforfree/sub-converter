use crate::error::{Error, Result};
use crate::formats::SingBoxConfig;
use crate::ir::Node;

pub struct SingBoxParser;

impl super::Parser for SingBoxParser {
    fn parse(&self, input: &str) -> Result<Vec<Node>> {
        let input_trim = input.trim_start();
        let cfg: SingBoxConfig = if input_trim.starts_with('{') {
            serde_json::from_str(input).map_err(|e| Error::ParseError {
                detail: format!("sing-box json: {e}"),
            })?
        } else {
            let v: serde_json::Value =
                serde_yaml::from_str(input).map_err(|e| Error::ParseError {
                    detail: format!("sing-box yaml: {e}"),
                })?;
            serde_json::from_value(v).map_err(|e| Error::ParseError {
                detail: format!("sing-box yaml->json: {e}"),
            })?
        };
        Ok(cfg.outbounds.into_iter().map(Into::into).collect())
    }
}
