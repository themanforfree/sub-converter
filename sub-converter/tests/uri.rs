use sub_converter::formats::{ClashConfig, SingBoxConfig};
use sub_converter::template::{OutputEncoding, Template};
use sub_converter::{OriginKind, convert};

#[test]
fn ss_uri_plain_and_base64() {
    let inputs = "ss://aes-256-gcm:pass@a.com:123#A\nss://YWVzLTI1Ni1nY206cGFzcw==@b.com:456#B\nss://YWVzLTI1Ni1nY206cGFzcw@c.com:789#C%20Name";

    let out = convert(
        OriginKind::UriList,
        inputs,
        Template::Clash(ClashConfig::default()),
        OutputEncoding::Yaml,
    )
    .expect("clash from uri");
    assert!(out.contains("A"));
    assert!(out.contains("B"));
    assert!(out.contains("C Name"));

    let out = convert(
        OriginKind::UriList,
        inputs,
        Template::SingBox(SingBoxConfig::default()),
        OutputEncoding::Json,
    )
    .expect("sb from uri");
    assert!(out.contains("A"));
    assert!(out.contains("B"));
    assert!(out.contains("C Name"));
}

#[test]
fn trojan_uri_with_tls() {
    let inputs = "trojan://pwd@exa.mple:443?sni=exa.mple&alpn=h2,http/1.1&insecure=1#T";
    let out = convert(
        OriginKind::UriList,
        inputs,
        Template::SingBox(SingBoxConfig::default()),
        OutputEncoding::Json,
    )
    .expect("trojan from uri");
    assert!(out.contains("T"));
    assert!(out.contains("exa.mple"));
}

#[test]
fn trojan_uri_with_extended_params() {
    let inputs = "trojan://password@applehk1.zymnode.cc:443?allowInsecure=0&peer=applehk1.zymnode.cc&sni=applehk1.zymnode.cc&type=tcp#HK-Node";
    let out = convert(
        OriginKind::UriList,
        inputs,
        Template::SingBox(SingBoxConfig::default()),
        OutputEncoding::Json,
    )
    .expect("trojan ext");
    assert!(out.contains("HK-Node"));
    assert!(out.contains("applehk1.zymnode.cc"));
}
