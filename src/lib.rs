pub mod api;
pub mod emit;
pub mod error;
pub mod ir;
pub mod merge;
pub mod parse;
pub mod template;

pub use api::{BuildOptions, Builder, InputFormat, OutputFormat, convert};
pub use error::{Error, Result};
pub use ir::*;
pub use parse::Parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_uri_to_clash_and_singbox() {
        let uris = "trojan://pwd@a.com:443#A\nss://YWVzLTI1Ni1nY206cGFzczpQM0UjQCM=@b.com:123#B";
        let clash = Builder::new()
            .add_input(InputFormat::UriList, uris, "uris")
            .target(OutputFormat::Clash)
            .build()
            .expect("clash output");
        assert!(clash.contains("proxies"));

        let sb = Builder::new()
            .add_input(InputFormat::UriList, uris, "uris")
            .target(OutputFormat::SingBox)
            .build()
            .expect("sb output");
        assert!(sb.contains("outbounds"));
    }
}
