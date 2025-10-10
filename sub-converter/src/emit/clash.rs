use crate::error::{Error, Result};
use crate::ir::{Node, Protocol, Subscription};
use crate::template::{ClashTemplate, Template};
use serde::Serialize;
use serde_yaml::Value;

pub struct ClashEmitter;

#[derive(Serialize)]
struct ClashOut {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    general: Option<Value>,
    proxies: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy_groups: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rules: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dns: Option<Value>,
}

impl super::Emitter for ClashEmitter {
    fn emit(&self, sub: &Subscription, tpl: &Template) -> Result<String> {
        let Template::Clash(ClashTemplate {
            general,
            proxy_groups,
            rules,
            dns,
        }) = tpl
        else {
            return Err(Error::EmitError {
                detail: "expect clash template".into(),
            });
        };
        let proxies: Vec<Value> = sub
            .nodes
            .iter()
            .map(node_to_clash_proxy)
            .collect::<Result<Vec<_>>>()?;
        let out = ClashOut {
            general: general.clone(),
            proxies,
            proxy_groups: proxy_groups.clone(),
            rules: rules.clone(),
            dns: dns.clone(),
        };
        let s = serde_yaml::to_string(&out).map_err(|e| Error::EmitError {
            detail: format!("clash yaml emit: {e}"),
        })?;
        Ok(s)
    }
}

fn node_to_clash_proxy(n: &Node) -> Result<Value> {
    match &n.protocol {
        Protocol::Shadowsocks { method, password } => {
            let mut m = serde_yaml::Mapping::new();
            m.insert(Value::from("name"), Value::from(n.name.clone()));
            m.insert(Value::from("type"), Value::from("ss"));
            m.insert(Value::from("server"), Value::from(n.server.clone()));
            m.insert(Value::from("port"), Value::from(n.port));
            m.insert(Value::from("cipher"), Value::from(method.clone()));
            m.insert(Value::from("password"), Value::from(password.clone()));
            Ok(Value::Mapping(m))
        }
        Protocol::Trojan { password } => {
            let mut m = serde_yaml::Mapping::new();
            m.insert(Value::from("name"), Value::from(n.name.clone()));
            m.insert(Value::from("type"), Value::from("trojan"));
            m.insert(Value::from("server"), Value::from(n.server.clone()));
            m.insert(Value::from("port"), Value::from(n.port));
            m.insert(Value::from("password"), Value::from(password.clone()));
            if let Some(tls) = &n.tls {
                if let Some(sni) = &tls.server_name {
                    m.insert(Value::from("sni"), Value::from(sni.clone()));
                }
                if let Some(alpn) = &tls.alpn {
                    m.insert(Value::from("alpn"), serde_yaml::to_value(alpn).unwrap());
                }
                if let Some(insecure) = tls.insecure {
                    m.insert(Value::from("skip-cert-verify"), Value::from(insecure));
                }
            }
            Ok(Value::Mapping(m))
        }
        _ => Err(Error::Unsupported {
            what: "protocol for clash".into(),
        }),
    }
}
