use worker::*;

mod route;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/profile", route::profile::handler)
        .get_async("/template", route::template::list)
        .get_async("/template/:name", route::template::get)
        .put_async("/template/:name", route::template::put)
        // rules endpoints (KV-based, raw lines)
        .get_async("/sets", route::rules::list_sets)
        .delete_async("/sets/:set_id", route::rules::delete_set)
        .get_async("/sets/:set_id/rules", route::rules::list_rules)
        .put_async("/sets/:set_id/rules", route::rules::add_rule)
        .delete_async("/sets/:set_id/rules/:suffix", route::rules::delete_rule)
        .get_async("/sets/:set_id.yaml", route::rules::export_yaml)
        .run(req, env)
        .await
}
