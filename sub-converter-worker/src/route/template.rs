use worker::{Bucket, Request, Response, Result, RouteContext};

pub async fn get_template(bucket: &Bucket, name: &str) -> std::result::Result<String, String> {
    let Some(objects) = bucket
        .get(name)
        .execute()
        .await
        .map_err(|e| e.to_string())?
    else {
        return Err("template not found".to_string());
    };
    let Some(body) = objects.body() else {
        return Err("template not found".to_string());
    };
    let res = body.text().await.map_err(|e| e.to_string())?;
    Ok(res)
}

pub async fn get(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("template name not found")?;
    let bucket = ctx.bucket("TEMPLATE")?;
    let res = get_template(&bucket, name).await?;

    Response::ok(res)
}

pub async fn put(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("template name not found")?;

    // Parse token from query parameter
    let url = req.url()?;
    let token = url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.to_string());

    // Validate token against TEMPLATE_TOKEN environment variable
    let provided = match token.as_deref() {
        Some(t) if !t.is_empty() => t,
        _ => return Response::error("unauthorized: missing token", 401),
    };

    let expected = match ctx.var("TEMPLATE_TOKEN") {
        Ok(v) => v.to_string(),
        Err(_) => return Response::error("server misconfigured: TEMPLATE_TOKEN missing", 500),
    };

    if provided != expected {
        return Response::error("unauthorized: invalid token", 401);
    }

    // Read template content from request body
    let body = match req.text().await {
        Ok(text) => text,
        Err(e) => return Response::error(format!("failed to read request body: {}", e), 400),
    };

    if body.is_empty() {
        return Response::error("template content cannot be empty", 400);
    }

    // Upload template to R2 bucket
    let bucket = ctx.bucket("TEMPLATE")?;
    bucket
        .put(name, body)
        .execute()
        .await
        .map_err(|e| format!("failed to upload template '{}': {}", name, e))?;

    Response::ok("template uploaded successfully")
}

pub async fn list(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let bucket = ctx.bucket("TEMPLATE")?;

    // List all objects in the bucket
    let list_result = bucket
        .list()
        .execute()
        .await
        .map_err(|e| worker::Error::RustError(format!("Failed to list templates: {}", e)))?;

    // Extract template names from the list
    let templates: Vec<String> = list_result
        .objects()
        .iter()
        .map(|obj| obj.key().to_string())
        .collect();

    // Return as JSON
    Response::from_json(&serde_json::json!({
        "templates": templates
    }))
}
