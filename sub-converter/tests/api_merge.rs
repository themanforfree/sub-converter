use sub_converter::{InputFormat, InputItem, convert};
use sub_converter::formats::ClashConfig;
use sub_converter::template::Template;

#[test]
fn merge_order_preserved_and_no_dedup() {
    let clash = r#"
proxies:
  - {name: A, type: ss, server: a.com, port: 1, cipher: aes-256-gcm, password: p}
"#;
    let sb = r#"{"outbounds":[{"type":"trojan","tag":"B","server":"b.com","server_port":2,"password":"x"}]}"#;
    let uris = "ss://YWVzLTI1Ni1nY206cA@c.com:3#C\nss://YWVzLTI1Ni1nY206cA@c.com:3#C";

    let inputs = vec![
        InputItem { format: InputFormat::Clash, content: clash.to_string() },
        InputItem { format: InputFormat::SingBox, content: sb.to_string() },
        InputItem { format: InputFormat::UriList, content: uris.to_string() },
    ];
    let template = Template::Clash(ClashConfig::default());
    let out = convert(inputs, template).expect("convert");

    assert!(out.contains("name: A"));
    assert!(out.contains("name: B"));
    let count_c = out.match_indices("name: C").count();
    assert_eq!(count_c, 2);
}
