use crate::error::{Error, Result};
use crate::ir::{Node, Protocol};
use serde::Deserialize;

pub struct SingBoxParser;

#[derive(Debug, Deserialize)]
struct SingBoxConfig {
    #[serde(default)]
    outbounds: Vec<Outbound>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
enum Outbound {
    Shadowsocks {
        tag: Option<String>,
        server: String,
        server_port: u16,
        method: String,
        password: String,
    },
    Trojan {
        tag: Option<String>,
        server: String,
        server_port: u16,
        password: String,
        #[serde(default)]
        tls: Option<SbTls>,
    },
    // Ignore other outbounds for now
}

#[derive(Debug, Deserialize)]
struct SbTls {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    server_name: Option<String>,
    #[serde(default)]
    alpn: Option<Vec<String>>,
    #[serde(default)]
    insecure: Option<bool>,
}

impl super::Parser for SingBoxParser {
    fn parse(&self, input: &str) -> Result<Vec<Node>> {
        let cfg: SingBoxConfig = serde_json::from_str(input).map_err(|e| Error::ParseError {
            detail: format!("sing-box json: {e}"),
        })?;
        let mut out = Vec::new();
        for ob in cfg.outbounds.into_iter() {
            match ob {
                Outbound::Shadowsocks {
                    tag,
                    server,
                    server_port,
                    method,
                    password,
                } => {
                    out.push(Node {
                        name: tag.unwrap_or_else(|| format!("ss:{}:{}", server, server_port)),
                        server,
                        port: server_port,
                        protocol: Protocol::Shadowsocks { method, password },
                        transport: None,
                        tls: None,
                        tags: Vec::new(),
                    });
                }
                Outbound::Trojan {
                    tag,
                    server,
                    server_port,
                    password,
                    tls,
                } => {
                    let t = tls.map(|t| crate::ir::Tls {
                        enabled: t.enabled.unwrap_or(true),
                        server_name: t.server_name,
                        alpn: t.alpn,
                        insecure: t.insecure,
                        utls_fingerprint: None,
                    });
                    out.push(Node {
                        name: tag.unwrap_or_else(|| format!("trojan:{}:{}", server, server_port)),
                        server,
                        port: server_port,
                        protocol: Protocol::Trojan { password },
                        transport: None,
                        tls: t,
                        tags: Vec::new(),
                    });
                }
            }
        }
        Ok(out)
    }
}
