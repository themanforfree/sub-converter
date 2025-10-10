use sub_converter::parse::uri::{ss::parse_ss_uri, trojan::parse_trojan_uri};

#[test]
fn ss_uri_plain_and_base64() {
    let n1 = parse_ss_uri("ss://aes-256-gcm:pass@a.com:123#A").expect("plain ss uri");
    assert_eq!(n1.name, "A");
    assert_eq!(n1.server, "a.com");
    assert_eq!(n1.port, 123);

    // base64("aes-256-gcm:pass") = YWVzLTI1Ni1nY206cGFzcw==
    let n2 = parse_ss_uri("ss://YWVzLTI1Ni1nY206cGFzcw==@b.com:456#B").expect("b64 ss uri");
    assert_eq!(n2.name, "B");
    assert_eq!(n2.server, "b.com");
    assert_eq!(n2.port, 456);

    // url-safe no pad variant with urlencoded fragment
    let n3 =
        parse_ss_uri("ss://YWVzLTI1Ni1nY206cGFzcw@c.com:789#C%20Name").expect("b64 urlsafe ss uri");
    assert_eq!(n3.name, "C Name");
}

#[test]
fn trojan_uri_with_tls() {
    let n =
        parse_trojan_uri("trojan://pwd@exa.mple:443?sni=exa.mple&alpn=h2,http/1.1&insecure=1#T")
            .expect("trojan uri");
    assert_eq!(n.name, "T");
    assert_eq!(n.server, "exa.mple");
    assert_eq!(n.port, 443);
    let tls = n.tls.expect("tls");
    assert_eq!(tls.server_name.as_deref(), Some("exa.mple"));
    assert!(tls.alpn.unwrap().contains(&"h2".to_string()))
}

#[test]
fn trojan_uri_with_extended_params() {
    // Test allowInsecure, peer, and type parameters
    let n = parse_trojan_uri(
        "trojan://password@applehk1.zymnode.cc:443?allowInsecure=0&peer=applehk1.zymnode.cc&sni=applehk1.zymnode.cc&type=tcp#HK-Node"
    )
    .expect("trojan uri with extended params");

    assert_eq!(n.name, "HK-Node");
    assert_eq!(n.server, "applehk1.zymnode.cc");
    assert_eq!(n.port, 443);

    // Check TLS config
    let tls = n.tls.expect("tls should be present");
    assert_eq!(tls.enabled, true);
    assert_eq!(tls.server_name.as_deref(), Some("applehk1.zymnode.cc"));
    assert_eq!(tls.insecure, Some(false)); // allowInsecure=0 means insecure=false

    // Check transport
    assert!(matches!(
        n.transport,
        Some(sub_converter::ir::Transport::Tcp)
    ));
}
