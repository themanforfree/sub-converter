use worker::*;

mod route;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/profile", route::profile::handler)
        .get_async("/template/:name", route::template::get)
        .run(req, env)
        .await
}
