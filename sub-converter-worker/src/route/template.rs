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
