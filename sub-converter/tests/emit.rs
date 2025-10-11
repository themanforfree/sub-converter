use sub_converter::emit::{Emitter, clash::ClashEmitter, sing_box::SingBoxEmitter};
use sub_converter::ir::{Node, Protocol, Subscription, Tls};
use sub_converter::template::{ClashTemplate, SingBoxTemplate, Template};

fn build_sub() -> Subscription {
    Subscription {
        nodes: vec![
            Node {
                name: "A".into(),
                server: "a.com".into(),
                port: 123,
                protocol: Protocol::Shadowsocks {
                    method: "aes-256-gcm".into(),
                    password: "pass".into(),
                },
                transport: None,
                tls: None,
                tags: vec![],
            },
            Node {
                name: "B".into(),
                server: "b.com".into(),
                port: 443,
                protocol: Protocol::Trojan {
                    password: "pwd".into(),
                },
                transport: None,
                tls: Some(Tls {
                    enabled: true,
                    server_name: Some("b.com".into()),
                    alpn: None,
                    insecure: None,
                    utls_fingerprint: None,
                }),
                tags: vec![],
            },
        ],
    }
}

#[test]
fn emit_clash_yaml() {
    let sub = build_sub();
    let tpl = Template::Clash(ClashTemplate::default());
    let s = ClashEmitter.emit(&sub, &tpl).expect("emit clash");
    assert!(s.contains("proxies"));
    assert!(s.contains("type: ss"));
    assert!(s.contains("type: trojan"));
}

#[test]
fn emit_singbox_json() {
    let sub = build_sub();
    let tpl = Template::SingBox(SingBoxTemplate::default());
    let s = SingBoxEmitter.emit(&sub, &tpl).expect("emit sb");
    assert!(s.contains("outbounds"));
    assert!(s.contains("\"type\": \"shadowsocks\""));
    assert!(s.contains("\"type\": \"trojan\""));
}

#[test]
fn emit_clash_with_all_proxies_placeholder() {
    let sub = build_sub();

    // Create a template with {{all_proxies}} placeholder
    let template_yaml = r#"
mode: rule
proxy-groups:
  - name: Proxies
    type: select
    proxies:
      - Auto
      - DIRECT
      - "{{all_proxies}}"
  - name: Auto
    type: url-test
    url: http://www.gstatic.com/generate_204
    interval: 300
    proxies:
      - "{{all_proxies}}"
rules:
  - GEOIP,CN,DIRECT
  - MATCH,Proxies
"#;

    let tpl: ClashTemplate = serde_yaml::from_str(template_yaml).expect("parse template");
    let s = ClashEmitter
        .emit(&sub, &Template::Clash(tpl))
        .expect("emit clash");

    // Verify that {{all_proxies}} was replaced with actual proxy names
    assert!(s.contains("- A"));
    assert!(s.contains("- B"));

    // Verify the placeholder is gone
    assert!(!s.contains("{{all_proxies}}"));

    // Both proxy groups should have the expanded proxies
    let yaml: serde_yaml::Value = serde_yaml::from_str(&s).expect("parse output");
    let groups = yaml["proxy-groups"].as_sequence().expect("proxy-groups");

    // Check Proxies group
    let proxies_group = &groups[0];
    let proxies_list = proxies_group["proxies"].as_sequence().expect("proxies");
    assert!(proxies_list.iter().any(|v| v.as_str() == Some("A")));
    assert!(proxies_list.iter().any(|v| v.as_str() == Some("B")));

    // Check Auto group
    let auto_group = &groups[1];
    let auto_list = auto_group["proxies"].as_sequence().expect("proxies");
    assert!(auto_list.iter().any(|v| v.as_str() == Some("A")));
    assert!(auto_list.iter().any(|v| v.as_str() == Some("B")));
}
