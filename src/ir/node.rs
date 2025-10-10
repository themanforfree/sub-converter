use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Protocol {
    Shadowsocks {
        method: String,
        password: String,
    },
    Trojan {
        password: String,
    },
    // Reserved for future protocols
    Vmess {
        id: String,
        security: Option<String>,
    },
    Vless {
        id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "transport", rename_all = "kebab-case")]
pub enum Transport {
    Tcp,
    Ws {
        path: Option<String>,
        headers: Option<Vec<(String, String)>>,
    },
    H2,
    Grpc {
        service_name: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Tls {
    pub enabled: bool,
    pub server_name: Option<String>,
    pub alpn: Option<Vec<String>>,
    pub insecure: Option<bool>,
    pub utls_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub protocol: Protocol,
    pub transport: Option<Transport>,
    pub tls: Option<Tls>,
    pub tags: Vec<String>,
}
