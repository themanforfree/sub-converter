use worker::{Request, Response, Result, RouteContext, kv::KvStore};

/// Get a rule from KV storage
pub async fn get_rule(kv: &KvStore, name: &str) -> std::result::Result<String, String> {
    let value = kv.get(name).text().await.map_err(|e| e.to_string())?;

    match value {
        Some(v) => Ok(v),
        None => Err("rule not found".to_string()),
    }
}

/// GET /rules/:name - Retrieve a rule from KV storage
pub async fn get(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("rule name not found")?;
    let kv = ctx.kv("RULES")?;
    let res = get_rule(&kv, name).await?;

    Response::ok(res)
}

/// PUT /rules/:name - Store or update a rule in KV storage
pub async fn put(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("rule name not found")?;

    // Parse token from query parameter
    let url = req.url()?;
    let token = url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.to_string());

    // Validate token against RULES_TOKEN environment variable
    let provided = match token.as_deref() {
        Some(t) if !t.is_empty() => t,
        _ => return Response::error("unauthorized: missing token", 401),
    };

    let expected = match ctx.var("RULES_TOKEN") {
        Ok(v) => v.to_string(),
        Err(_) => return Response::error("server misconfigured: RULES_TOKEN missing", 500),
    };

    if provided != expected {
        return Response::error("unauthorized: invalid token", 401);
    }

    // Read rule content from request body
    let body = match req.text().await {
        Ok(text) => text,
        Err(e) => return Response::error(format!("failed to read request body: {}", e), 400),
    };

    if body.is_empty() {
        return Response::error("rule content cannot be empty", 400);
    }

    // Store rule in KV
    let kv = ctx.kv("RULES")?;
    kv.put(name, body)
        .map_err(|e| format!("failed to create KV put operation: {}", e))?
        .execute()
        .await
        .map_err(|e| format!("failed to store rule '{}': {}", name, e))?;

    Response::ok("rule stored successfully")
}

/// GET /rules - List all available rules in KV storage
pub async fn list(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.kv("RULES")?;

    // List all keys in the KV namespace
    let list_result = kv
        .list()
        .execute()
        .await
        .map_err(|e| worker::Error::RustError(format!("Failed to list rules: {}", e)))?;

    // Extract rule names from the list
    let rules: Vec<String> = list_result
        .keys
        .iter()
        .map(|key| key.name.clone())
        .collect();

    // Return as JSON
    Response::from_json(&serde_json::json!({
        "rules": rules
    }))
}
