use worker::*;

mod route;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/profile", route::profile::handler)
        .run(req, env)
        .await
}
