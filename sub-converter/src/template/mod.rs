use crate::error::{Error, Result};
use crate::formats::{ClashConfig, SingBoxConfig};
use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Template {
    Clash(ClashConfig),
    SingBox(SingBoxConfig),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputEncoding {
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    Clash,
    Singbox,
}

impl TargetKind {
    pub fn default_encoding(self) -> OutputEncoding {
        match self {
            TargetKind::Clash => OutputEncoding::Yaml,
            TargetKind::Singbox => OutputEncoding::Json,
        }
    }
}

fn parse_clash_from_str(s: &str) -> Result<ClashConfig> {
    let trimmed = s.trim_start();
    if trimmed.starts_with('{') {
        // JSON -> YAML Value -> ClashConfig
        let v: serde_yaml::Value =
            serde_json::from_str(trimmed).map_err(|e| Error::ParseError {
                detail: format!("clash template json: {e}"),
            })?;
        serde_yaml::from_value(v).map_err(|e| Error::ParseError {
            detail: format!("clash template json->yaml: {e}"),
        })
    } else {
        serde_yaml::from_str(trimmed).map_err(|e| Error::ParseError {
            detail: format!("clash template yaml: {e}"),
        })
    }
}

fn parse_singbox_from_str(s: &str) -> Result<SingBoxConfig> {
    let trimmed = s.trim_start();
    if trimmed.starts_with('{') {
        serde_json::from_str(trimmed).map_err(|e| Error::ParseError {
            detail: format!("sing-box template json: {e}"),
        })
    } else {
        let v: serde_json::Value =
            serde_yaml::from_str(trimmed).map_err(|e| Error::ParseError {
                detail: format!("sing-box template yaml: {e}"),
            })?;
        serde_json::from_value(v).map_err(|e| Error::ParseError {
            detail: format!("sing-box template yaml->json: {e}"),
        })
    }
}

/// Build a Template from provided text.
/// - If `text` is None, returns default config for the target kind
/// - If provided, tries to parse as-is; if it fails, tries base64-decoding then parse
pub fn parse_template_text(kind: TargetKind, text: Option<&str>) -> Result<Template> {
    match (kind, text) {
        (TargetKind::Clash, None) => Ok(Template::Clash(ClashConfig::default())),
        (TargetKind::Singbox, None) => Ok(Template::SingBox(SingBoxConfig::default())),
        (TargetKind::Clash, Some(s)) => {
            // try raw first
            match parse_clash_from_str(s) {
                Ok(cfg) => Ok(Template::Clash(cfg)),
                Err(_) => {
                    // try base64 -> utf8 -> parse
                    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s.trim())
                        && let Ok(decoded_str) = String::from_utf8(decoded)
                    {
                        Ok(Template::Clash(parse_clash_from_str(&decoded_str)?))
                    } else {
                        // return original error context
                        Ok(Template::Clash(parse_clash_from_str(s)?))
                    }
                }
            }
        }
        (TargetKind::Singbox, Some(s)) => match parse_singbox_from_str(s) {
            Ok(cfg) => Ok(Template::SingBox(cfg)),
            Err(_) => {
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s.trim())
                    && let Ok(decoded_str) = String::from_utf8(decoded)
                {
                    Ok(Template::SingBox(parse_singbox_from_str(&decoded_str)?))
                } else {
                    Ok(Template::SingBox(parse_singbox_from_str(s)?))
                }
            }
        },
    }
}
