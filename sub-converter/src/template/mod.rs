use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lazy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "use")]
    pub use_provider: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(flatten)]
    pub other: serde_yaml::Mapping,
}

#[derive(Debug, Clone, Default)]
pub struct ClashTemplate {
    pub general: Option<Value>,
    pub proxy_groups: Option<Vec<ProxyGroup>>,
    pub rules: Option<Value>,
    pub dns: Option<Value>,
}

impl<'de> Deserialize<'de> for ClashTemplate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = BTreeMap::<String, Value>::deserialize(deserializer)?;

        let proxy_groups = map
            .remove("proxy-groups")
            .map(serde_yaml::from_value)
            .transpose()
            .map_err(serde::de::Error::custom)?;
        let rules = map.remove("rules");
        let dns = map.remove("dns");

        // Everything else goes into general
        let general = if map.is_empty() {
            None
        } else {
            Some(Value::Mapping(
                map.into_iter()
                    .map(|(k, v)| (Value::String(k), v))
                    .collect(),
            ))
        };

        Ok(ClashTemplate {
            general,
            proxy_groups,
            rules,
            dns,
        })
    }
}

impl Serialize for ClashTemplate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        if let Some(Value::Mapping(m)) = &self.general {
            for (k, v) in m {
                map.serialize_entry(k, v)?;
            }
        }

        if let Some(pg) = &self.proxy_groups {
            map.serialize_entry("proxy-groups", pg)?;
        }

        if let Some(r) = &self.rules {
            map.serialize_entry("rules", r)?;
        }

        if let Some(d) = &self.dns {
            map.serialize_entry("dns", d)?;
        }

        map.end()
    }
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
