use crate::error::Result;
use crate::formats::{ClashConfig, ClashProxy, ProxyGroup};
use crate::ir::Subscription;
use crate::template::Template;

pub struct ClashEmitter;

impl super::Emitter for ClashEmitter {
    fn emit(&self, sub: Subscription, tpl: Template) -> Result<String> {
        let Template::Clash(clash_tpl) = tpl else {
            return Err(crate::error::Error::EmitError {
                detail: "expect clash template".into(),
            });
        };

        let proxies: Vec<ClashProxy> = sub
            .nodes
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        let proxy_names: Vec<String> = proxies
            .iter()
            .map(|p| match p {
                ClashProxy::Ss { name, .. } | ClashProxy::Trojan { name, .. } => name.clone(),
            })
            .collect();

        let processed_proxy_groups = clash_tpl
            .proxy_groups
            .as_ref()
            .map(|groups| process_proxy_groups(groups, &proxy_names));

        let out = ClashConfig {
            general: clash_tpl.general,
            proxies,
            proxy_groups: processed_proxy_groups,
            rules: clash_tpl.rules,
            dns: clash_tpl.dns,
        };

        let s = serde_yaml::to_string(&out).map_err(|e| crate::error::Error::EmitError {
            detail: format!("clash yaml emit: {e}"),
        })?;
        Ok(s)
    }
}

/// Process proxy groups to replace {{all_proxies}} placeholder
fn process_proxy_groups(groups: &[ProxyGroup], proxy_names: &[String]) -> Vec<ProxyGroup> {
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
                            // Expand placeholder to all proxy names
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
