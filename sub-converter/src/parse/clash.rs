use crate::error::{Error, Result};
use crate::ir::{Node, Protocol, Tls};
use serde::Deserialize;

pub struct ClashParser;

#[derive(Debug, Deserialize)]
struct ClashConfig {
    #[serde(default)]
    proxies: Vec<ClashProxy>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum ClashProxy {
    Ss {
        name: String,
        server: String,
        port: u16,
        cipher: String,
        password: String,
        #[allow(dead_code)]
        #[serde(default)]
        udp: Option<bool>,
    },
    Trojan {
        name: String,
        server: String,
        port: u16,
        password: String,
        #[serde(default)]
        sni: Option<String>,
        #[serde(default)]
        alpn: Option<Vec<String>>,
        #[serde(default)]
        skip_cert_verify: Option<bool>,
    },
    // Ignore other types for now
}

impl super::Parser for ClashParser {
    fn parse(&self, input: &str) -> Result<Vec<Node>> {
        let cfg: ClashConfig = serde_yaml::from_str(input).map_err(|e| Error::ParseError {
            detail: format!("clash yaml: {e}"),
        })?;
        let mut out = Vec::new();
        for p in cfg.proxies.into_iter() {
            match p {
                ClashProxy::Ss {
                    name,
                    server,
                    port,
                    cipher,
                    password,
                    ..
                } => {
                    out.push(Node {
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
                    });
                }
                ClashProxy::Trojan {
                    name,
                    server,
                    port,
                    password,
                    sni,
                    alpn,
                    skip_cert_verify,
                } => {
                    out.push(Node {
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
                    });
                }
            }
        }
        Ok(out)
    }
}
