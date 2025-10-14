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
    // Get template name from URL parameter
    let name = ctx.param("name").ok_or("template name not found")?;

    // Token validation: must provide and match environment variable TEMPLATE_TOKEN
    let auth_header = req
        .headers()
        .get("Authorization")
        .unwrap_or(None)
        .ok_or("unauthorized: missing authorization header")?;

    let provided = auth_header
        .strip_prefix("Bearer ")
        .ok_or("unauthorized: invalid authorization format")?;

    let expected = match ctx.var("TEMPLATE_TOKEN") {
        Ok(v) => v.to_string(),
        Err(_) => return Response::error("server misconfigured: TEMPLATE_TOKEN missing", 500),
    };

    if provided != expected {
        return Response::error("unauthorized: invalid token", 401);
    }

    // Get template content from request body
    let body = match req.text().await {
        Ok(text) => text,
        Err(e) => return Response::error(format!("failed to read request body: {}", e), 400),
    };

    if body.is_empty() {
        return Response::error("template content cannot be empty", 400);
    }

    // Store template in R2 bucket
    let bucket = ctx.bucket("TEMPLATE")?;
    bucket
        .put(name, body)
        .execute()
        .await
        .map_err(|e| format!("failed to upload template: {}", e))?;

    Response::ok("template uploaded successfully")
}
