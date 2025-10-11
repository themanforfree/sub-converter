use crate::error::{Error, Result};
use crate::ir::{Node, Protocol, Tls};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::BTreeMap;

/// Clash proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ClashProxy {
    #[serde(rename = "ss")]
    Ss {
        name: String,
        server: String,
        port: u16,
        cipher: String,
        password: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        udp: Option<bool>,
    },
    Trojan {
        name: String,
        server: String,
        port: u16,
        password: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sni: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alpn: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "skip-cert-verify")]
        skip_cert_verify: Option<bool>,
    },
}

/// Clash proxy group configuration
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

/// Top-level Clash configuration that can be used for both parsing and templates
#[derive(Debug, Clone, Default)]
pub struct ClashConfig {
    /// General configuration options (e.g., port, socks-port, allow-lan, mode, log-level, etc.)
    pub general: Option<Value>,
    /// List of proxies
    pub proxies: Vec<ClashProxy>,
    /// Proxy groups
    pub proxy_groups: Option<Vec<ProxyGroup>>,
    /// Rules
    pub rules: Option<Value>,
    /// DNS configuration
    pub dns: Option<Value>,
}

impl<'de> Deserialize<'de> for ClashConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = BTreeMap::<String, Value>::deserialize(deserializer)?;

        let proxies = map
            .remove("proxies")
            .map(serde_yaml::from_value)
            .transpose()
            .map_err(serde::de::Error::custom)?
            .unwrap_or_default();

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

        Ok(ClashConfig {
            general,
            proxies,
            proxy_groups,
            rules,
            dns,
        })
    }
}

impl Serialize for ClashConfig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
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

        if !self.proxies.is_empty() {
            map.serialize_entry("proxies", &self.proxies)?;
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

// Conversion from ClashProxy to Node
impl From<ClashProxy> for Node {
    fn from(proxy: ClashProxy) -> Self {
        match proxy {
            ClashProxy::Ss {
                name,
                server,
                port,
                cipher,
                password,
                ..
            } => Node {
                name,
                server,
                port,
                protocol: Protocol::Shadowsocks {
                    method: cipher,
                    password,
                },
                transport: None,
                tls: None,
                tags: Vec::new(),
            },
            ClashProxy::Trojan {
                name,
                server,
                port,
                password,
                sni,
                alpn,
                skip_cert_verify,
            } => Node {
                name,
                server,
                port,
                protocol: Protocol::Trojan { password },
                transport: None,
                tls: Some(Tls {
                    enabled: true,
                    server_name: sni,
                    alpn,
                    insecure: skip_cert_verify,
                    utls_fingerprint: None,
                }),
                tags: Vec::new(),
            },
        }
    }
}

// Conversion from Node to ClashProxy (fallible)
impl TryFrom<Node> for ClashProxy {
    type Error = Error;

    fn try_from(node: Node) -> Result<Self> {
        let Node {
            name,
            server,
            port,
            protocol,
            tls,
            ..
        } = node;

        match protocol {
            Protocol::Shadowsocks { method, password } => {
                if method.is_empty() || password.is_empty() {
                    return Err(Error::ValidationError {
                        reason: format!("Empty method or password for Shadowsocks node: {}", name),
                    });
                }

                Ok(ClashProxy::Ss {
                    name,
                    server,
                    port,
                    cipher: method,
                    password,
                    udp: None,
                })
            }
            Protocol::Trojan { password } => {
                if password.is_empty() {
                    return Err(Error::ValidationError {
                        reason: format!("Empty password for Trojan node: {}", name),
                    });
                }

                let (sni, alpn, skip_cert_verify) = tls
                    .map(|tls| (tls.server_name, tls.alpn, tls.insecure))
                    .unwrap_or((None, None, None));

                Ok(ClashProxy::Trojan {
                    name,
                    server,
                    port,
                    password,
                    sni,
                    alpn,
                    skip_cert_verify,
                })
            }
            _ => Err(Error::Unsupported {
                what: format!("protocol '{}' for clash", protocol.name()),
            }),
        }
    }
}
