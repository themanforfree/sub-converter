use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClashTemplate {
    pub general: Option<serde_yaml::Value>,
    pub proxy_groups: Option<serde_yaml::Value>,
    pub rules: Option<serde_yaml::Value>,
    pub dns: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SingBoxTemplate {
    pub general: Option<serde_json::Value>,
    pub inbounds: Option<serde_json::Value>,
    pub route: Option<serde_json::Value>,
    pub dns: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum Template {
    Clash(ClashTemplate),
    SingBox(SingBoxTemplate),
}
