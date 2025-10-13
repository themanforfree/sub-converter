use worker::{Request, Response, Result, RouteContext};

pub async fn get(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = ctx.param("name").ok_or("template name not found")?;
    let bucket = ctx.bucket("TEMPLATE")?;
    let Some(objects) = bucket.get(name).execute().await? else {
        return Response::error("template not found", 404);
    };
    let Some(body) = objects.body() else {
        return Response::error("template not found", 404);
    };
    let res = body.text().await?;

    Response::ok(res)
}
