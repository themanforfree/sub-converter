use sub_converter::parse::{Parser, clash::ClashParser, sing_box::SingBoxParser};

#[test]
fn parse_clash_minimal() {
    let yaml = r#"
proxies:
  - name: A
    type: ss
    server: a.com
    port: 123
    cipher: aes-256-gcm
    password: pass
  - name: B
    type: trojan
    server: b.com
    port: 443
    password: pwd
    sni: b.com
"#;
    let nodes = ClashParser.parse(yaml).expect("clash parse");
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].name, "A");
    assert_eq!(nodes[1].name, "B");
}

#[test]
fn parse_singbox_minimal() {
    let json = r#"{
  "outbounds": [
    {"type":"shadowsocks","tag":"A","server":"a.com","server_port":123,"method":"aes-256-gcm","password":"pass"},
    {"type":"trojan","tag":"B","server":"b.com","server_port":443,"password":"pwd","tls":{"enabled":true,"server_name":"b.com"}}
  ]
}"#;
    let nodes = SingBoxParser.parse(json).expect("sb parse");
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].name, "A");
    assert_eq!(nodes[1].name, "B");
}
