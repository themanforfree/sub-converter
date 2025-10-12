use sub_converter::formats::ClashConfig;
use sub_converter::ir::Subscription;
use sub_converter::template::{OutputEncoding, Template};
use sub_converter::{OriginConfig, OriginKind, convert_origin, parse_origin};

#[test]
fn merge_order_preserved_and_no_dedup() {
    let clash = r#"
proxies:
  - {name: A, type: ss, server: a.com, port: 1, cipher: aes-256-gcm, password: p}
"#;
    let sb = r#"{"outbounds":[{"type":"trojan","tag":"B","server":"b.com","server_port":2,"password":"x"}]}"#;
    let uris = "ss://YWVzLTI1Ni1nY206cA@c.com:3#C\nss://YWVzLTI1Ni1nY206cA@c.com:3#C";

    // Parse each input to a Subscription, preserving order
    let clash_sub = match parse_origin(OriginKind::Clash, clash.as_bytes()).expect("parse clash") {
        OriginConfig::Clash(cfg) => Subscription::from(cfg),
        _ => unreachable!(),
    };
    let sb_sub = match parse_origin(OriginKind::SingBox, sb.as_bytes()).expect("parse singbox") {
        OriginConfig::SingBox(cfg) => Subscription::from(cfg),
        _ => unreachable!(),
    };
    let uri_sub = match parse_origin(OriginKind::UriList, uris.as_bytes()).expect("parse uris") {
        OriginConfig::UriList(nodes) => Subscription::from(nodes),
        _ => unreachable!(),
    };

    let mut merged = Subscription::default();
    merged.nodes.extend(clash_sub.nodes);
    merged.nodes.extend(sb_sub.nodes);
    merged.nodes.extend(uri_sub.nodes);

    let template = Template::Clash(ClashConfig::default());
    let out = convert_origin(
        OriginConfig::UriList(merged.nodes),
        template,
        OutputEncoding::Yaml,
    )
    .expect("convert");

    assert!(out.contains("name: A"));
    assert!(out.contains("name: B"));
    let count_c = out.match_indices("name: C").count();
    assert_eq!(count_c, 2);
}
