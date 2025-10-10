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
        meta: vec![],
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
