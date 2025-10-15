use worker::{Request, Response, Result, RouteContext, kv::KvStore};

/// Get rules from KV storage as Vec<String>
async fn get_rules_vec(kv: &KvStore, name: &str) -> std::result::Result<Vec<String>, String> {
    let value = kv.get(name).text().await.map_err(|e| e.to_string())?;

    match value {
        Some(v) => {
            // Parse stored JSON array
            serde_json::from_str::<Vec<String>>(&v).map_err(|e| format!("parse error: {}", e))
        }
        None => Err("rule set not found".to_string()),
    }
}

/// Store rules to KV storage as JSON array
async fn put_rules_vec(
    kv: &KvStore,
    name: &str,
    rules: Vec<String>,
) -> std::result::Result<(), String> {
    let json_content =
        serde_json::to_string(&rules).map_err(|e| format!("serialize error: {}", e))?;

    kv.put(name, json_content)
        .map_err(|e| format!("failed to create KV put operation: {}", e))?
        .execute()
        .await
        .map_err(|e| format!("failed to store rule set '{}': {}", name, e))?;

    Ok(())
}

/// GET /rules/:name - Retrieve entire rule set as Clash rule-provider format
pub async fn get(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("rule set name not found")?;
    let kv = ctx.kv("RULES")?;
    let rules = get_rules_vec(&kv, name).await?;

    // Format as Clash rule-provider YAML
    let yaml_output = serde_yaml::to_string(&serde_json::json!({
        "payload": rules
    }))
    .map_err(|e| format!("failed to generate YAML: {}", e))?;

    Response::ok(yaml_output)
}

/// Validate token from request
fn validate_token(ctx: &RouteContext<()>, req: &Request) -> std::result::Result<(), worker::Error> {
    let url = req.url()?;
    let token = url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.to_string());

    let provided = match token.as_deref() {
        Some(t) if !t.is_empty() => t,
        _ => {
            return Err(worker::Error::RustError(
                "unauthorized: missing token".to_string(),
            ));
        }
    };

    let expected = match ctx.var("RULES_TOKEN") {
        Ok(v) => v.to_string(),
        Err(_) => {
            return Err(worker::Error::RustError(
                "server misconfigured: RULES_TOKEN missing".to_string(),
            ));
        }
    };

    if provided != expected {
        return Err(worker::Error::RustError(
            "unauthorized: invalid token".to_string(),
        ));
    }

    Ok(())
}

/// PUT /rules/:name - Replace entire rule set
pub async fn put(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("rule set name not found")?;
    if let Err(e) = validate_token(&ctx, &req) {
        return Response::error(e.to_string(), 401);
    }

    // Read request body
    let body = match req.text().await {
        Ok(text) => text,
        Err(e) => return Response::error(format!("failed to read request body: {}", e), 400),
    };

    if body.is_empty() {
        return Response::error("rule content cannot be empty", 400);
    }

    // Parse input as JSON array or line-separated text
    let rules: Vec<String> = if body.trim_start().starts_with('[') {
        // Parse as JSON array
        match serde_json::from_str::<Vec<String>>(&body) {
            Ok(arr) => arr,
            Err(e) => {
                return Response::error(format!("invalid JSON array: {}", e), 400);
            }
        }
    } else {
        // Parse as line-separated text
        body.lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(String::from)
            .collect()
    };

    if rules.is_empty() {
        return Response::error("rule set cannot be empty", 400);
    }

    // Store rules
    let kv = ctx.kv("RULES")?;
    put_rules_vec(&kv, name, rules)
        .await
        .map_err(worker::Error::RustError)?;

    Response::ok("rule set replaced successfully")
}

/// POST /rules/:name/rules - Add rules to existing rule set
pub async fn add_rules(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("rule set name not found")?;
    if let Err(e) = validate_token(&ctx, &req) {
        return Response::error(e.to_string(), 401);
    }

    // Read request body
    let body = match req.text().await {
        Ok(text) => text,
        Err(e) => return Response::error(format!("failed to read request body: {}", e), 400),
    };

    if body.is_empty() {
        return Response::error("rules to add cannot be empty", 400);
    }

    // Parse new rules
    let new_rules: Vec<String> = if body.trim_start().starts_with('[') {
        match serde_json::from_str::<Vec<String>>(&body) {
            Ok(arr) => arr,
            Err(e) => return Response::error(format!("invalid JSON array: {}", e), 400),
        }
    } else {
        body.lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(String::from)
            .collect()
    };

    if new_rules.is_empty() {
        return Response::error("no valid rules to add", 400);
    }

    // Get existing rules or create new set
    let kv = ctx.kv("RULES")?;
    let mut rules = get_rules_vec(&kv, name).await.unwrap_or_default();

    // Add new rules
    rules.extend(new_rules);

    // Store updated rules
    put_rules_vec(&kv, name, rules)
        .await
        .map_err(worker::Error::RustError)?;

    Response::ok("rules added successfully")
}

/// DELETE /rules/:name/rules/:index - Delete a specific rule by index
pub async fn delete_rule(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("rule set name not found")?;
    let index_str = ctx.param("index").ok_or("rule index not found")?;
    if let Err(e) = validate_token(&ctx, &req) {
        return Response::error(e.to_string(), 401);
    }

    let index: usize = match index_str.parse() {
        Ok(i) => i,
        Err(_) => return Response::error("invalid index", 400),
    };

    // Get existing rules
    let kv = ctx.kv("RULES")?;
    let mut rules = get_rules_vec(&kv, name).await?;

    if index >= rules.len() {
        return Response::error(
            format!("index {} out of range (0-{})", index, rules.len() - 1),
            400,
        );
    }

    // Remove rule at index
    rules.remove(index);

    if rules.is_empty() {
        return Response::error(
            "cannot delete last rule; use DELETE /rules/:name to delete entire rule set",
            400,
        );
    }

    // Store updated rules
    put_rules_vec(&kv, name, rules)
        .await
        .map_err(worker::Error::RustError)?;

    Response::ok(format!("rule at index {} deleted successfully", index))
}

/// PUT /rules/:name/rules/:index - Update a specific rule by index
pub async fn update_rule(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("rule set name not found")?;
    let index_str = ctx.param("index").ok_or("rule index not found")?;
    if let Err(e) = validate_token(&ctx, &req) {
        return Response::error(e.to_string(), 401);
    }

    let index: usize = match index_str.parse() {
        Ok(i) => i,
        Err(_) => return Response::error("invalid index", 400),
    };

    // Read new rule content
    let body = match req.text().await {
        Ok(text) => text.trim().to_string(),
        Err(e) => return Response::error(format!("failed to read request body: {}", e), 400),
    };

    if body.is_empty() {
        return Response::error("rule content cannot be empty", 400);
    }

    // Get existing rules
    let kv = ctx.kv("RULES")?;
    let mut rules = get_rules_vec(&kv, name).await?;

    if index >= rules.len() {
        return Response::error(
            format!("index {} out of range (0-{})", index, rules.len() - 1),
            400,
        );
    }

    // Update rule at index
    rules[index] = body;

    // Store updated rules
    put_rules_vec(&kv, name, rules)
        .await
        .map_err(worker::Error::RustError)?;

    Response::ok(format!("rule at index {} updated successfully", index))
}

/// DELETE /rules/:name - Delete entire rule set
pub async fn delete(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("rule set name not found")?;
    if let Err(e) = validate_token(&ctx, &req) {
        return Response::error(e.to_string(), 401);
    }

    let kv = ctx.kv("RULES")?;
    kv.delete(name)
        .await
        .map_err(|e| worker::Error::RustError(format!("failed to delete rule set: {}", e)))?;

    Response::ok("rule set deleted successfully")
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
