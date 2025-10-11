use sub_converter::formats::{ClashConfig, SingBoxConfig};
use sub_converter::template::Template;
use sub_converter::{InputFormat, InputItem, convert};

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
    let inputs = vec![InputItem {
        format: InputFormat::ClashYaml,
        content: yaml.into(),
    }];
    let out = convert(inputs, Template::ClashYaml(ClashConfig::default())).expect("clash parse");
    assert!(out.contains("A"));
    assert!(out.contains("B"));
}

#[test]
fn parse_singbox_minimal() {
    let json = r#"{
  "outbounds": [
    {"type":"shadowsocks","tag":"A","server":"a.com","server_port":123,"method":"aes-256-gcm","password":"pass"},
    {"type":"trojan","tag":"B","server":"b.com","server_port":443,"password":"pwd","tls":{"enabled":true,"server_name":"b.com"}}
  ]
}"#;
    let inputs = vec![InputItem {
        format: InputFormat::SingBoxJson,
        content: json.into(),
    }];
    let out = convert(inputs, Template::SingBoxJson(SingBoxConfig::default())).expect("sb parse");
    assert!(out.contains("A"));
    assert!(out.contains("B"));
}
