use crate::error::{Error, Result};
use crate::ir::{Node, Protocol, Subscription};
use crate::template::{SingBoxTemplate, Template};
use serde_json::{Value, json};

pub struct SingBoxEmitter;

impl super::Emitter for SingBoxEmitter {
    fn emit(&self, sub: &Subscription, tpl: &Template) -> Result<String> {
        let Template::SingBox(SingBoxTemplate {
            general,
            inbounds,
            route,
            dns,
        }) = tpl
        else {
            return Err(Error::EmitError {
                detail: "expect sing-box template".into(),
            });
        };
        let outbounds: Vec<Value> = sub
            .nodes
            .iter()
            .map(node_to_singbox_outbound)
            .collect::<Result<Vec<_>>>()?;

        let mut root = serde_json::Map::new();
        if let Some(g) = general
            && let Some(obj) = g.as_object()
        {
            for (k, v) in obj {
                root.insert(k.clone(), v.clone());
            }
        }

        if let Some(ib) = inbounds {
            root.insert("inbounds".into(), ib.clone());
        }
        root.insert("outbounds".into(), Value::Array(outbounds));
        if let Some(r) = route {
            root.insert("route".into(), r.clone());
        }
        if let Some(d) = dns {
            root.insert("dns".into(), d.clone());
        }

        let s =
            serde_json::to_string_pretty(&Value::Object(root)).map_err(|e| Error::EmitError {
                detail: format!("sing-box json emit: {e}"),
            })?;
        Ok(s)
    }
}

fn node_to_singbox_outbound(n: &Node) -> Result<Value> {
    match &n.protocol {
        Protocol::Shadowsocks { method, password } => Ok(json!({
            "type": "shadowsocks",
            "tag": n.name,
            "server": n.server,
            "server_port": n.port,
            "method": method,
            "password": password,
        })),
        Protocol::Trojan { password } => {
            let mut m = serde_json::Map::from_iter(vec![
                ("type".into(), Value::from("trojan")),
                ("tag".into(), Value::from(n.name.clone())),
                ("server".into(), Value::from(n.server.clone())),
                ("server_port".into(), Value::from(n.port)),
                ("password".into(), Value::from(password.clone())),
            ]);
            if let Some(tls) = &n.tls {
                let mut tls_obj = serde_json::Map::new();
                tls_obj.insert("enabled".into(), Value::from(tls.enabled));
                if let Some(sni) = &tls.server_name {
                    tls_obj.insert("server_name".into(), Value::from(sni.clone()));
                }
                if let Some(alpn) = &tls.alpn {
                    tls_obj.insert("alpn".into(), serde_json::to_value(alpn).unwrap());
                }
                if let Some(insecure) = tls.insecure {
                    tls_obj.insert("insecure".into(), Value::from(insecure));
                }
                m.insert("tls".into(), Value::from(tls_obj));
            }
            Ok(Value::Object(m))
        }
        _ => Err(Error::Unsupported {
            what: "protocol for sing-box".into(),
        }),
    }
}
