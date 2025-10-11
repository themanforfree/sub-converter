mod api;
pub mod emit;
mod error;
pub mod formats;
pub mod ir;
mod merge;
pub mod parse;
pub mod template;

pub use api::{InputFormat, InputItem, OutputFormat, convert, detect_format};
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    use crate::{
        formats::{ClashConfig, SingBoxConfig},
        template::Template,
    };

    use super::*;

    #[test]
    fn smoke_uri_to_clash_and_singbox() {
        let uris = "trojan://pwd@a.com:443#A\nss://YWVzLTI1Ni1nY206cGFzczpQM0UjQCM=@b.com:123#B";
        let inputs = vec![InputItem {
            format: InputFormat::UriList,
            content: uris.to_string(),
        }];
        let template = Template::Clash(ClashConfig::default());
        let clash = convert(inputs.clone(), template).expect("clash output");
        assert!(clash.contains("proxies"));

        let template = Template::SingBox(SingBoxConfig::default());
        let sb = convert(inputs, template).expect("sb output");
        assert!(sb.contains("outbounds"));
    }
}
