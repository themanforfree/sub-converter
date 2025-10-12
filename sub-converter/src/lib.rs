mod api;
mod error;
pub mod formats;
pub mod ir;
pub mod parse;
pub mod template;

pub use api::{
    OriginConfig, OriginKind, convert, convert_origin, detect_origin_kind, parse_origin,
};
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    use crate::{
        formats::{ClashConfig, SingBoxConfig},
        template::{OutputEncoding, Template},
    };

    use super::*;

    #[test]
    fn smoke_uri_to_clash_and_singbox() {
        let uris = "trojan://pwd@a.com:443#A\nss://YWVzLTI1Ni1nY206cGFzczpQM0UjQCM=@b.com:123#B";

        // Clash YAML output
        let clash = convert(
            OriginKind::Auto,
            uris,
            Template::Clash(ClashConfig::default()),
            OutputEncoding::Yaml,
        )
        .expect("clash output");
        assert!(clash.contains("proxies"));

        // SingBox JSON output
        let sb = convert(
            OriginKind::Auto,
            uris,
            Template::SingBox(SingBoxConfig::default()),
            OutputEncoding::Json,
        )
        .expect("sb output");
        assert!(sb.contains("outbounds"));
    }
}
