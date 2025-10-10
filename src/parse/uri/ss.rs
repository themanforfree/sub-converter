use crate::error::{Error, Result};
use crate::ir::{Node, Protocol};
use base64::Engine;
use percent_encoding::percent_decode_str;
use url::Url;

// Minimal SIP002: ss://method:password@host:port#name
// Also support ss://BASE64(method:password)@host:port#name
pub fn parse_ss_uri(s: &str) -> Result<Node> {
    let url = Url::parse(s).map_err(|e| Error::ParseError {
        detail: format!("ss uri: {e}"),
    })?;
    let host = url.host_str().ok_or_else(|| Error::ParseError {
        detail: "ss uri: missing host".into(),
    })?;
    let port = url.port().ok_or_else(|| Error::ParseError {
        detail: "ss uri: missing port".into(),
    })?;
    let name = url
        .fragment()
        .map(|f| percent_decode_str(f).decode_utf8_lossy().to_string())
        .unwrap_or_else(|| format!("ss:{}:{}", host, port));

    // authority userinfo may be either method:password or base64(method:password)
    let userinfo = url.username();
    let password_opt = url.password();
    let (method, password) = if !userinfo.is_empty()
        && let Some(password) = password_opt
    {
        (userinfo.to_string(), password.to_string())
    } else if !userinfo.is_empty() && password_opt.is_none() {
        // try base64 (STANDARD / STANDARD_NO_PAD / URL_SAFE / URL_SAFE_NO_PAD)
        let ui_decoded = percent_decode_str(userinfo).decode_utf8_lossy();
        let bytes = ui_decoded.as_bytes();
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(bytes)
            .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(bytes))
            .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(bytes))
            .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(bytes))
            .map_err(|e| Error::ParseError {
                detail: format!("ss uri base64: {e}"),
            })?;
        let decoded = String::from_utf8(decoded).map_err(|e| Error::ParseError {
            detail: format!("ss uri base64 utf8: {e}"),
        })?;
        let mut parts = decoded.splitn(2, ':');
        let method = parts.next().ok_or_else(|| Error::ParseError {
            detail: "ss uri: missing method".into(),
        })?;
        let password = parts.next().ok_or_else(|| Error::ParseError {
            detail: "ss uri: missing password".into(),
        })?;
        (method.to_string(), password.to_string())
    } else {
        return Err(Error::ParseError {
            detail: "ss uri: missing credentials".into(),
        });
    };

    Ok(Node {
        name,
        server: host.to_string(),
        port,
        protocol: Protocol::Shadowsocks { method, password },
        transport: None,
        tls: None,
        tags: Vec::new(),
    })
}
