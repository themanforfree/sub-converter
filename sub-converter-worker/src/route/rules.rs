use worker::{kv::KvStore, Request, Response, Result, RouteContext};

const SET_MARKER_PREFIX: &str = "sets:";
const RULE_PREFIX: &str = "rp:"; // rp:{set_id}:{ulid}

fn set_marker_key(set_id: &str) -> String { format!("{}{}", SET_MARKER_PREFIX, set_id) }
fn rule_key(set_id: &str, suffix: &str) -> String { format!("{}{}:{}", RULE_PREFIX, set_id, suffix) }
fn rule_prefix(set_id: &str) -> String { format!("{}{}:", RULE_PREFIX, set_id) }

fn yaml_response(body: String) -> Result<Response> {
    let mut resp = Response::ok(body)?;
    resp.headers_mut().set("content-type", "text/yaml; charset=utf-8")?;
    Ok(resp)
}

async fn kv(ctx: &RouteContext<()>) -> Result<KvStore> { ctx.kv("RULES") }

pub async fn list_sets(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = kv(&ctx).await?;
    let list = kv.list().prefix(SET_MARKER_PREFIX.to_string()).execute().await?;
    let mut sets = Vec::new();
    for k in list.keys {
        if let Some(name) = k.name.strip_prefix(SET_MARKER_PREFIX) {
            sets.push(name.to_string());
        }
    }
    Response::from_json(&serde_json::json!({ "sets": sets }))
}

pub async fn delete_set(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let set_id = ctx.param("set_id").ok_or("missing set_id")?;
    let kv = kv(&ctx).await?;

    // delete marker
    let _ = kv.delete(&set_marker_key(set_id)).await;

    // list and delete all rules under this set
    let prefix = rule_prefix(set_id);
    let list = kv.list().prefix(prefix).execute().await?;
    for k in list.keys {
        let _ = kv.delete(&k.name).await;
    }
    Response::ok("deleted")
}

pub async fn list_rules(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let set_id = ctx.param("set_id").ok_or("missing set_id")?;
    let kv = kv(&ctx).await?;
    let prefix = rule_prefix(set_id);
    let list = kv.list().prefix(prefix).execute().await?;

    // Collect keys and sort by suffix (ULID) lexicographically
    let mut keys: Vec<String> = list.keys.into_iter().map(|k| k.name).collect();
    keys.sort();

    let mut out = Vec::new();
    for key in keys.into_iter() {
        if let Some(v) = kv.get(&key).text().await? {
            out.push(serde_json::json!({ "key": key, "value": v }));
        }
    }
    Response::from_json(&serde_json::json!({ "rules": out }))
}

pub async fn export_yaml(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Try router params: some router impls bind param name literally as "set_id.yaml"
    let set_id_from_router = ctx
        .param("set_id")
        .or_else(|| ctx.param("set_id.yaml"))
        .map(|s| s.to_string());

    // If router param includes suffix, strip it; else parse from path
    let set_id = match set_id_from_router {
        Some(mut id) => {
            if let Some(stripped) = id.strip_suffix(".yaml") {
                id = stripped.to_string();
            }
            id
        }
        None => {
            let path = req.url()?.path().to_string();
            // Expected prefix/suffix
            let prefix = "/sets/";
            let suffix = ".yaml";
            if path.starts_with(prefix) && path.ends_with(suffix) && path.len() > prefix.len() + suffix.len() {
                path[prefix.len()..path.len() - suffix.len()].to_string()
            } else {
                return Response::error("missing set_id", 400);
            }
        }
    };

    let kv = kv(&ctx).await?;
    let prefix = rule_prefix(&set_id);
    let list = kv.list().prefix(prefix).execute().await?;

    let mut keys: Vec<String> = list.keys.into_iter().map(|k| k.name).collect();
    keys.sort();

    // Read values into Vec<String>
    let mut vals: Vec<String> = Vec::new();
    for key in keys.into_iter() {
        if let Some(v) = kv.get(&key).text().await? {
            vals.push(v);
        }
    }

    // Build YAML: payload: [lines]
    let yaml = match serde_yaml::to_string(&serde_json::json!({"payload": vals})) {
        Ok(s) => s,
        Err(_) => "payload: []\n".to_string(),
    };
    yaml_response(yaml)
}

pub async fn add_rule(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let set_id = ctx.param("set_id").ok_or("missing set_id")?;
    let kv = kv(&ctx).await?;

    let value = req.text().await.unwrap_or_default();

    // Ensure set marker exists
    if let Ok(builder) = kv.put(&set_marker_key(set_id), "1") { let _ = builder.execute().await; }

    // Generate lexicographically sortable suffix: 13-digit ms timestamp + 6-digit random
    fn new_suffix() -> String {
        let now_ms = js_sys::Date::now() as u64;
        let rand = (js_sys::Math::random() * 1_000_000.0) as u64; // 0..999999
        format!("{:013}-{:06}", now_ms, rand)
    }
    let key = rule_key(set_id, &new_suffix());
    kv.put(&key, value)?.execute().await?;

    Response::from_json(&serde_json::json!({ "key": key }))
}

pub async fn delete_rule(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let set_id = ctx.param("set_id").ok_or("missing set_id")?;
    let suffix = ctx.param("suffix").ok_or("missing suffix")?;
    let kv = kv(&ctx).await?;
    // Allow deleting by full key or by suffix
    let key = if suffix.contains(':') {
        // if client passed full key like rp:{set_id}:{suffix}
        suffix.to_string()
    } else {
        rule_key(set_id, suffix)
    };
    let _ = kv.delete(&key).await; // best-effort
    Response::ok("deleted")
}
