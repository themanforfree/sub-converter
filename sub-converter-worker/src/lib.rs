use worker::*;

mod route;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/profile", route::profile::handler)
        .get_async("/template", route::template::list)
        .get_async("/template/:name", route::template::get)
        .put_async("/template/:name", route::template::put)
        .get_async("/rules", route::rules::list)
        .get_async("/rules/:name", route::rules::get)
        .put_async("/rules/:name", route::rules::put)
        .run(req, env)
        .await
}
