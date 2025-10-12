use crate::error::{Error, Result};
use crate::formats::{ClashConfig, SingBoxConfig};
use crate::ir::Subscription;
use crate::parse::uri::parse_uri_line;
use crate::template::{OutputEncoding, Template};
use base64::Engine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginKind {
    Auto,
    Clash,
    SingBox,
    UriList,
}

#[derive(Debug, Clone)]
pub enum OriginConfig {
    Clash(ClashConfig),
    SingBox(SingBoxConfig),
    UriList(Vec<crate::ir::Node>),
}

pub fn detect_origin_kind(bytes: &[u8]) -> Result<OriginKind> {
    let s = match std::str::from_utf8(bytes) {
        Ok(v) => v.trim(),
        Err(_) => {
            return Err(Error::ParseError {
                detail: "input is not valid utf-8".into(),
            });
        }
    };

    // try base64 decode -> uri list
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s)
        && let Ok(decoded_str) = String::from_utf8(decoded)
        && is_uri_list_format(decoded_str.trim())
    {
        return Ok(OriginKind::UriList);
    }

    if is_uri_list_format(s) {
        return Ok(OriginKind::UriList);
    }

    // json
    let is_json = s.trim_start().starts_with('{');
    if is_json {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
            if v.get("proxies").is_some() {
                return Ok(OriginKind::Clash);
            }
            if v.get("outbounds").is_some() {
                return Ok(OriginKind::SingBox);
            }
        }
    } else {
        // yaml heuristics
        if s.contains("proxies:") {
            return Ok(OriginKind::Clash);
        }
        if s.contains("outbounds:") {
            return Ok(OriginKind::SingBox);
        }
    }

    Err(Error::ValidationError {
        reason: "无法自动检测输入类型，请显式指定".into(),
    })
}

fn is_uri_list_format(content: &str) -> bool {
    let lines: Vec<&str> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    !lines.is_empty() && lines.iter().all(|line| line.contains("://"))
}

fn parse_inner(kind: OriginKind, bytes: &[u8]) -> Result<OriginConfig> {
    match kind {
        OriginKind::Clash => parse_clash(bytes).map(OriginConfig::Clash),
        OriginKind::SingBox => parse_singbox(bytes).map(OriginConfig::SingBox),
        OriginKind::UriList => parse_urilist(bytes).map(OriginConfig::UriList),
        OriginKind::Auto => unreachable!(),
    }
}

pub fn parse_origin(kind: OriginKind, bytes: &[u8]) -> Result<OriginConfig> {
    let kind = match kind {
        OriginKind::Auto => detect_origin_kind(bytes)?,
        _ => kind,
    };
    parse_inner(kind, bytes)
}

fn parse_clash(bytes: &[u8]) -> Result<ClashConfig> {
    let s = std::str::from_utf8(bytes).map_err(|e| Error::ParseError {
        detail: format!("utf8: {e}"),
    })?;
    let input_trim = s.trim_start();
    let cfg: ClashConfig = if input_trim.starts_with('{') {
        // json -> yaml value -> ClashConfig
        let v: serde_yaml::Value = serde_json::from_str(s).map_err(|e| Error::ParseError {
            detail: format!("clash json: {e}"),
        })?;
        serde_yaml::from_value(v).map_err(|e| Error::ParseError {
            detail: format!("clash json->yaml: {e}"),
        })?
    } else {
        serde_yaml::from_str(s).map_err(|e| Error::ParseError {
            detail: format!("clash yaml: {e}"),
        })?
    };
    Ok(cfg)
}

fn parse_singbox(bytes: &[u8]) -> Result<SingBoxConfig> {
    let s = std::str::from_utf8(bytes).map_err(|e| Error::ParseError {
        detail: format!("utf8: {e}"),
    })?;
    let input_trim = s.trim_start();
    let cfg: SingBoxConfig = if input_trim.starts_with('{') {
        serde_json::from_str(s).map_err(|e| Error::ParseError {
            detail: format!("sing-box json: {e}"),
        })?
    } else {
        let v: serde_json::Value = serde_yaml::from_str(s).map_err(|e| Error::ParseError {
            detail: format!("sing-box yaml: {e}"),
        })?;
        serde_json::from_value(v).map_err(|e| Error::ParseError {
            detail: format!("sing-box yaml->json: {e}"),
        })?
    };
    Ok(cfg)
}

fn parse_urilist(bytes: &[u8]) -> Result<Vec<crate::ir::Node>> {
    // try base64 first
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => String::new(),
    };
    let candidate = if let Ok(decoded) =
        base64::engine::general_purpose::STANDARD.decode(text.trim())
        && let Ok(decoded_str) = String::from_utf8(decoded)
    {
        decoded_str
    } else {
        text
    };

    let mut out = Vec::new();
    for line in candidate.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(node) = parse_uri_line(line)? {
            out.push(node);
        }
    }
    Ok(out)
}

pub fn convert(
    kind: OriginKind,
    input: impl AsRef<[u8]>,
    target: Template,
    enc: OutputEncoding,
) -> Result<String> {
    let origin = parse_origin(kind, input.as_ref())?;
    convert_origin(origin, target, enc)
}

pub fn convert_origin(
    origin: OriginConfig,
    target: Template,
    enc: OutputEncoding,
) -> Result<String> {
    let sub: Subscription = match origin {
        OriginConfig::Clash(cfg) => Subscription::from(cfg),
        OriginConfig::SingBox(cfg) => Subscription::from(cfg),
        OriginConfig::UriList(nodes) => Subscription::from(nodes),
    };
    convert_subscription(&sub, target, enc)
}

pub fn convert_subscription(
    sub: &Subscription,
    target: Template,
    enc: OutputEncoding,
) -> Result<String> {
    match target {
        Template::Clash(tpl) => {
            let out = build_clash_output(sub, &tpl);
            serialize_clash(&out, enc)
        }
        Template::SingBox(tpl) => {
            let out = build_singbox_output(sub, &tpl);
            serialize_singbox(&out, enc)
        }
    }
}

fn build_clash_output(sub: &Subscription, tpl: &ClashConfig) -> ClashConfig {
    use crate::formats::ClashProxy;
    // nodes -> proxies
    let proxies: Vec<ClashProxy> = sub
        .nodes
        .clone()
        .into_iter()
        .map(|n| n.try_into())
        .filter_map(Result::ok)
        .collect();
    let proxy_names: Vec<String> = proxies
        .iter()
        .map(|p| match p {
            ClashProxy::Ss { name, .. } | ClashProxy::Trojan { name, .. } => name.clone(),
        })
        .collect();

    let processed_proxy_groups = tpl
        .proxy_groups
        .as_ref()
        .map(|groups| process_proxy_groups(groups, &proxy_names));

    ClashConfig {
        general: tpl.general.clone(),
        proxies,
        proxy_groups: processed_proxy_groups,
        rules: tpl.rules.clone(),
        dns: tpl.dns.clone(),
    }
}

fn process_proxy_groups(
    groups: &[crate::formats::ProxyGroup],
    proxy_names: &[String],
) -> Vec<crate::formats::ProxyGroup> {
    const ALL_PROXIES_PLACEHOLDER: &str = "{{all_proxies}}";
    groups
        .iter()
        .map(|group| {
            let mut group = group.clone();
            if let Some(proxies) = &mut group.proxies {
                let new_proxies: Vec<String> = proxies
                    .iter()
                    .flat_map(|proxy| {
                        if proxy == ALL_PROXIES_PLACEHOLDER {
                            proxy_names.to_vec()
                        } else {
                            vec![proxy.clone()]
                        }
                    })
                    .collect();
                *proxies = new_proxies;
            }
            group
        })
        .collect()
}

fn build_singbox_output(sub: &Subscription, tpl: &SingBoxConfig) -> SingBoxConfig {
    use crate::formats::SingBoxOutbound;

    let mut outbounds_from_ir: Vec<SingBoxOutbound> = sub
        .nodes
        .clone()
        .into_iter()
        .map(|n| n.try_into())
        .filter_map(Result::ok)
        .collect();

    let proxy_names: Vec<String> = outbounds_from_ir
        .iter()
        .filter_map(|o| match o {
            SingBoxOutbound::Shadowsocks { tag, .. } => tag.clone(),
            SingBoxOutbound::Trojan { tag, .. } => tag.clone(),
            _ => None,
        })
        .collect();

    let processed_template_outbounds = process_outbounds(&tpl.outbounds, &proxy_names);

    let mut final_outbounds = processed_template_outbounds;
    final_outbounds.append(&mut outbounds_from_ir);

    SingBoxConfig {
        general: tpl.general.clone(),
        inbounds: tpl.inbounds.clone(),
        outbounds: final_outbounds,
        route: tpl.route.clone(),
        dns: tpl.dns.clone(),
    }
}

fn expand_outbound_names(list: &[String], proxy_names: &[String]) -> Vec<String> {
    const PH: &str = "{{all_proxies}}";
    let mut v = Vec::with_capacity(list.len().saturating_mul(2));
    for name in list {
        if name == PH {
            v.extend(proxy_names.iter().cloned());
        } else {
            v.push(name.clone());
        }
    }
    v
}

fn process_outbounds(
    outbounds: &[crate::formats::SingBoxOutbound],
    proxy_names: &[String],
) -> Vec<crate::formats::SingBoxOutbound> {
    use crate::formats::SingBoxOutbound;
    outbounds
        .iter()
        .map(|outbound| match outbound {
            SingBoxOutbound::Selector { tag, outbounds } => SingBoxOutbound::Selector {
                tag: tag.clone(),
                outbounds: expand_outbound_names(outbounds, proxy_names),
            },
            SingBoxOutbound::Urltest {
                tag,
                outbounds,
                url,
                interval,
                tolerance,
            } => SingBoxOutbound::Urltest {
                tag: tag.clone(),
                outbounds: expand_outbound_names(outbounds, proxy_names),
                url: url.clone(),
                interval: interval.clone(),
                tolerance: *tolerance,
            },
            _ => outbound.clone(),
        })
        .collect()
}

fn serialize_clash(cfg: &ClashConfig, enc: OutputEncoding) -> Result<String> {
    match enc {
        OutputEncoding::Json => serde_json::to_string_pretty(cfg).map_err(|e| Error::EmitError {
            detail: format!("clash json emit: {e}"),
        }),
        OutputEncoding::Yaml => serde_yaml::to_string(cfg).map_err(|e| Error::EmitError {
            detail: format!("clash yaml emit: {e}"),
        }),
    }
}

fn serialize_singbox(cfg: &SingBoxConfig, enc: OutputEncoding) -> Result<String> {
    match enc {
        OutputEncoding::Json => serde_json::to_string_pretty(cfg).map_err(|e| Error::EmitError {
            detail: format!("sing-box json emit: {e}"),
        }),
        OutputEncoding::Yaml => {
            let v = serde_json::to_value(cfg).map_err(|e| Error::EmitError {
                detail: format!("sing-box yaml to_value: {e}"),
            })?;
            serde_yaml::to_string(&v).map_err(|e| Error::EmitError {
                detail: format!("sing-box yaml emit: {e}"),
            })
        }
    }
}
