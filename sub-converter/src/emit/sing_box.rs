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

        let outbounds: Vec<SingBoxOutbound> = sub
            .nodes
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        let out = SingBoxConfig {
            general: sb_tpl.general.clone(),
            inbounds: sb_tpl.inbounds.clone(),
            outbounds,
            route: sb_tpl.route.clone(),
            dns: sb_tpl.dns.clone(),
        };

        let s = serde_json::to_string_pretty(&out).map_err(|e| crate::error::Error::EmitError {
            detail: format!("sing-box json emit: {e}"),
        })?;
        Ok(s)
    }
}
