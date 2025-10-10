use crate::error::{Error, Result};
use crate::ir::{Node, Protocol, Tls};
use percent_encoding::percent_decode_str;
use url::Url;

// trojan://password@host:port?security=tls&sni=example.com#name
// optionally ws params in query are ignored for now
pub fn parse_trojan_uri(s: &str) -> Result<Node> {
    let url = Url::parse(s).map_err(|e| Error::ParseError {
        detail: format!("trojan uri: {e}"),
    })?;
    let host = url.host_str().ok_or_else(|| Error::ParseError {
        detail: "trojan uri: missing host".into(),
    })?;
    let port = url.port().ok_or_else(|| Error::ParseError {
        detail: "trojan uri: missing port".into(),
    })?;
    let name = url
        .fragment()
        .map(|f| percent_decode_str(f).decode_utf8_lossy().to_string())
        .unwrap_or_else(|| format!("trojan:{}:{}", host, port));
    let password = url.username();
    if password.is_empty() {
        return Err(Error::ParseError {
            detail: "trojan uri: missing password".into(),
        });
    }

    // tls params
    let mut tls = Tls {
        enabled: true,
        ..Default::default()
    };
    let qp = url.query_pairs();
    for (k, v) in qp {
        match k.as_ref() {
            "sni" | "server_name" => tls.server_name = Some(v.into_owned()),
            "alpn" => tls.alpn = Some(v.split(',').map(|s| s.to_string()).collect()),
            "insecure" => tls.insecure = Some(v == "1" || v.eq_ignore_ascii_case("true")),
            _ => {}
        }
    }

    Ok(Node {
        name,
        server: host.to_string(),
        port,
        protocol: Protocol::Trojan {
            password: password.to_string(),
        },
        transport: None,
        tls: Some(tls),
        tags: Vec::new(),
    })
}
