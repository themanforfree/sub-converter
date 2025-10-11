use crate::error::{Error, Result};
use crate::ir::{Node, Protocol, Tls};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Sing-Box outbound configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum SingBoxOutbound {
    Shadowsocks {
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        server: String,
        server_port: u16,
        method: String,
        password: String,
    },
    Trojan {
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        server: String,
        server_port: u16,
        password: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tls: Option<SingBoxTls>,
    },
}

/// Sing-Box TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingBoxTls {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure: Option<bool>,
}

/// Top-level Sing-Box configuration that can be used for both parsing and templates
#[derive(Debug, Clone, Default)]
pub struct SingBoxConfig {
    /// General configuration options (e.g., log level, experimental features, etc.)
    pub general: Option<Value>,
    /// Inbound configurations
    pub inbounds: Option<Value>,
    /// Outbound configurations
    pub outbounds: Vec<SingBoxOutbound>,
    /// Route configuration
    pub route: Option<Value>,
    /// DNS configuration
    pub dns: Option<Value>,
}

impl<'de> Deserialize<'de> for SingBoxConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = BTreeMap::<String, Value>::deserialize(deserializer)?;

        let outbounds = map
            .remove("outbounds")
            .map(serde_json::from_value)
            .transpose()
            .map_err(serde::de::Error::custom)?
            .unwrap_or_default();

        let inbounds = map.remove("inbounds");
        let route = map.remove("route");
        let dns = map.remove("dns");

        // Everything else goes into general
        let general = if map.is_empty() {
            None
        } else {
            Some(Value::Object(map.into_iter().collect()))
        };

        Ok(SingBoxConfig {
            general,
            inbounds,
            outbounds,
            route,
            dns,
        })
    }
}

impl Serialize for SingBoxConfig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        if let Some(Value::Object(m)) = &self.general {
            for (k, v) in m {
                map.serialize_entry(k, v)?;
            }
        }

        if let Some(ib) = &self.inbounds {
            map.serialize_entry("inbounds", ib)?;
        }

        if !self.outbounds.is_empty() {
            map.serialize_entry("outbounds", &self.outbounds)?;
        }

        if let Some(r) = &self.route {
            map.serialize_entry("route", r)?;
        }

        if let Some(d) = &self.dns {
            map.serialize_entry("dns", d)?;
        }

        map.end()
    }
}

// Conversion from SingBoxTls to Tls
impl From<SingBoxTls> for Tls {
    fn from(tls: SingBoxTls) -> Self {
        Tls {
            enabled: tls.enabled.unwrap_or(true),
            server_name: tls.server_name,
            alpn: tls.alpn,
            insecure: tls.insecure,
            utls_fingerprint: None,
        }
    }
}

// Conversion from Tls to SingBoxTls
impl From<&Tls> for SingBoxTls {
    fn from(tls: &Tls) -> Self {
        SingBoxTls {
            enabled: Some(tls.enabled),
            server_name: tls.server_name.clone(),
            alpn: tls.alpn.clone(),
            insecure: tls.insecure,
        }
    }
}

// Conversion from SingBoxOutbound to Node
impl From<SingBoxOutbound> for Node {
    fn from(outbound: SingBoxOutbound) -> Self {
        match outbound {
            SingBoxOutbound::Shadowsocks {
                tag,
                server,
                server_port,
                method,
                password,
            } => Node {
                name: tag.unwrap_or_else(|| format!("ss:{}:{}", server, server_port)),
                server,
                port: server_port,
                protocol: Protocol::Shadowsocks { method, password },
                transport: None,
                tls: None,
                tags: Vec::new(),
            },
            SingBoxOutbound::Trojan {
                tag,
                server,
                server_port,
                password,
                tls,
            } => Node {
                name: tag.unwrap_or_else(|| format!("trojan:{}:{}", server, server_port)),
                server,
                port: server_port,
                protocol: Protocol::Trojan { password },
                transport: None,
                tls: tls.map(Into::into),
                tags: Vec::new(),
            },
        }
    }
}

// Conversion from Node to SingBoxOutbound (fallible)
impl TryFrom<&Node> for SingBoxOutbound {
    type Error = Error;

    fn try_from(node: &Node) -> Result<Self> {
        match &node.protocol {
            Protocol::Shadowsocks { method, password } => {
                if method.is_empty() || password.is_empty() {
                    return Err(Error::ValidationError {
                        reason: format!(
                            "Empty method or password for Shadowsocks node: {}",
                            node.name
                        ),
                    });
                }

                Ok(SingBoxOutbound::Shadowsocks {
                    tag: Some(node.name.clone()),
                    server: node.server.clone(),
                    server_port: node.port,
                    method: method.clone(),
                    password: password.clone(),
                })
            }
            Protocol::Trojan { password } => {
                if password.is_empty() {
                    return Err(Error::ValidationError {
                        reason: format!("Empty password for Trojan node: {}", node.name),
                    });
                }

                let tls = node.tls.as_ref().map(Into::into);
                Ok(SingBoxOutbound::Trojan {
                    tag: Some(node.name.clone()),
                    server: node.server.clone(),
                    server_port: node.port,
                    password: password.clone(),
                    tls,
                })
            }
            _ => Err(Error::Unsupported {
                what: format!("protocol '{}' for sing-box", node.protocol.name()),
            }),
        }
    }
}
