use sub_converter::{InputFormat, InputItem, convert};
use sub_converter::template::Template;
use sub_converter::formats::{ClashConfig, SingBoxConfig};

#[test]
fn ss_uri_plain_and_base64() {
    let inputs = vec![InputItem { format: InputFormat::UriList, content:
        "ss://aes-256-gcm:pass@a.com:123#A\nss://YWVzLTI1Ni1nY206cGFzcw==@b.com:456#B\nss://YWVzLTI1Ni1nY206cGFzcw@c.com:789#C%20Name".into() }];

    let out = convert(inputs.clone(), Template::Clash(ClashConfig::default())).expect("clash from uri");
    assert!(out.contains("A"));
    assert!(out.contains("B"));
    assert!(out.contains("C Name"));

    let out = convert(inputs, Template::SingBox(SingBoxConfig::default())).expect("sb from uri");
    assert!(out.contains("A"));
    assert!(out.contains("B"));
    assert!(out.contains("C Name"));
}

#[test]
fn trojan_uri_with_tls() {
    let inputs = vec![InputItem { format: InputFormat::UriList, content:
        "trojan://pwd@exa.mple:443?sni=exa.mple&alpn=h2,http/1.1&insecure=1#T".into() }];
    let out = convert(inputs, Template::SingBox(SingBoxConfig::default())).expect("trojan from uri");
    assert!(out.contains("T"));
    assert!(out.contains("exa.mple"));
}

#[test]
fn trojan_uri_with_extended_params() {
    let inputs = vec![InputItem { format: InputFormat::UriList, content:
        "trojan://password@applehk1.zymnode.cc:443?allowInsecure=0&peer=applehk1.zymnode.cc&sni=applehk1.zymnode.cc&type=tcp#HK-Node".into() }];
    let out = convert(inputs, Template::SingBox(SingBoxConfig::default())).expect("trojan ext");
    assert!(out.contains("HK-Node"));
    assert!(out.contains("applehk1.zymnode.cc"));
}
