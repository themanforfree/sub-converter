use sub_converter::formats::{ClashConfig, SingBoxConfig};
use sub_converter::template::Template;
use sub_converter::{InputFormat, InputItem, convert};

fn build_inputs() -> Vec<InputItem> {
    // A: ss, B: trojan
    let uris = "ss://YWVzLTI1Ni1nY206cGFzcw@a.com:123#A\n\
                trojan://pwd@b.com:443#B";
    vec![InputItem {
        format: InputFormat::UriList,
        content: uris.to_string(),
    }]
}

#[test]
fn emit_clash_yaml() {
    let inputs = build_inputs();
    let tpl = Template::ClashYaml(ClashConfig::default());
    let s = convert(inputs, tpl).expect("emit clash");
    assert!(s.contains("proxies"));
    assert!(s.contains("type: ss"));
    assert!(s.contains("type: trojan"));
}

#[test]
fn emit_singbox_json() {
    let inputs = build_inputs();
    let tpl = Template::SingBoxJson(SingBoxConfig::default());
    let s = convert(inputs, tpl).expect("emit sb");
    assert!(s.contains("outbounds"));
    assert!(s.contains("\"type\": \"shadowsocks\""));
    assert!(s.contains("\"type\": \"trojan\""));
}

#[test]
fn emit_clash_with_all_proxies_placeholder() {
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

    let tpl: ClashConfig = serde_yaml::from_str(template_yaml).expect("parse template");

    let inputs = build_inputs();
    let s = convert(inputs, Template::ClashYaml(tpl)).expect("emit clash");

    assert!(s.contains("- A"));
    assert!(s.contains("- B"));
    assert!(!s.contains("{{all_proxies}}"));

    let yaml: serde_yaml::Value = serde_yaml::from_str(&s).expect("parse output");
    let groups = yaml["proxy-groups"].as_sequence().expect("proxy-groups");

    let proxies_group = &groups[0];
    let proxies_list = proxies_group["proxies"].as_sequence().expect("proxies");
    assert!(proxies_list.iter().any(|v| v.as_str() == Some("A")));
    assert!(proxies_list.iter().any(|v| v.as_str() == Some("B")));

    let auto_group = &groups[1];
    let auto_list = auto_group["proxies"].as_sequence().expect("proxies");
    assert!(auto_list.iter().any(|v| v.as_str() == Some("A")));
    assert!(auto_list.iter().any(|v| v.as_str() == Some("B")));
}
