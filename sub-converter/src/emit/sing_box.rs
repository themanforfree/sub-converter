use crate::error::Result;
use crate::formats::{SingBoxConfig, SingBoxOutbound};
use crate::ir::Subscription;
use crate::template::Template;

pub struct SingBoxEmitter;

impl super::Emitter for SingBoxEmitter {
    fn emit(&self, sub: &Subscription, tpl: &Template) -> Result<String> {
        let Template::SingBox(sb_tpl) = tpl else {
            return Err(crate::error::Error::EmitError {
                detail: "expect sing-box template".into(),
            });
        };

        let mut outbounds: Vec<SingBoxOutbound> = sub
            .nodes
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        let proxy_names: Vec<String> = sub.nodes.iter().map(|n| n.name.clone()).collect();

        let processed_template_outbounds = process_outbounds(&sb_tpl.outbounds, &proxy_names);

        let mut final_outbounds = processed_template_outbounds;
        final_outbounds.append(&mut outbounds);

        let dns = sb_tpl.dns.clone();

        let out = SingBoxConfig {
            general: sb_tpl.general.clone(),
            inbounds: sb_tpl.inbounds.clone(),
            outbounds: final_outbounds,
            route: sb_tpl.route.clone(),
            dns,
        };

        let s = serde_json::to_string_pretty(&out).map_err(|e| crate::error::Error::EmitError {
            detail: format!("sing-box json emit: {e}"),
        })?;
        Ok(s)
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
    outbounds: &[SingBoxOutbound],
    proxy_names: &[String],
) -> Vec<SingBoxOutbound> {
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
