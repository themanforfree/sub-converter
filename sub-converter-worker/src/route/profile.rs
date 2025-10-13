use serde::Deserialize;
use sub_converter::{
    OriginKind,
    api::convert_full,
    template::{OutputEncoding, TargetKind},
};
use worker::{Fetch, Request, Response, Result, RouteContext, Url};

#[derive(Debug, Deserialize)]
pub struct ProfileQuery {
    /// 源订阅链接（必填）
    origin_url: String,
    /// 可选：源类型，auto|clash|singbox|uri
    #[serde(default)]
    origin_kind: Option<OriginKind>,
    /// 目标订阅格式（必填）：clash|singbox
    target_kind: TargetKind,
    /// 编码（可选）：json|yaml
    #[serde(default)]
    encoding: Option<OutputEncoding>,
    /// 模版（可选）：base64 编码内容
    #[serde(default)]
    template_b64: Option<String>,
    /// 模版（可选）：模板地址（http/https）
    #[serde(default)]
    template_url: Option<String>,
    /// 授权令牌（必填）：通过查询参数 ?token= 传入
    #[serde(default)]
    token: Option<String>,
}

async fn fetch_text(url: &str) -> std::result::Result<String, String> {
    let url = Url::parse(url).map_err(|e| e.to_string())?;
    let mut res = Fetch::Url(url).send().await.map_err(|e| e.to_string())?;
    let body = res.text().await.map_err(|e| e.to_string())?;
    Ok(body)
}

async fn resolve_template(q: &ProfileQuery) -> std::result::Result<Option<String>, String> {
    if let Some(ref url) = q.template_url {
        return fetch_text(url).await.map(Some);
    }
    Ok(q.template_b64.clone())
}

pub async fn handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // 解析查询参数
    let url = req.url()?;
    let q: ProfileQuery = match url.query() {
        Some(qs) => match serde_urlencoded::from_str(qs) {
            Ok(v) => v,
            Err(e) => return Response::error(format!("invalid query: {e}"), 400),
        },
        None => return Response::error("missing query", 400),
    };

    // token 校验：必须提供且匹配环境变量 PROFILE_TOKEN
    let provided = match q.token.as_deref() {
        Some(t) if !t.is_empty() => t,
        _ => return Response::error("unauthorized: missing token", 401),
    };
    let expected = match ctx.var("PROFILE_TOKEN") {
        Ok(v) => v.to_string(),
        Err(_) => return Response::error("server misconfigured: PROFILE_TOKEN missing", 500),
    };
    if provided != expected {
        return Response::error("unauthorized: invalid token", 401);
    }

    if q.origin_url.is_empty() {
        return Response::error("missing origin", 400);
    }

    // 获取源内容
    let origin_text = match fetch_text(&q.origin_url).await {
        Ok(v) => v,
        Err(e) => return Response::error(format!("fetch origin failed: {e}"), 502),
    };

    // 解析模板（可选）
    let template_raw = match resolve_template(&q).await {
        Ok(v) => v,
        Err(e) => return Response::error(format!("template error: {e}"), 400),
    };

    let (body, resolved_enc) = match convert_full(
        q.origin_kind.unwrap_or_default(),
        origin_text,
        q.target_kind,
        template_raw.as_deref(),
        q.encoding,
    ) {
        Ok(v) => v,
        Err(e) => return Response::error(format!("convert error: {e:?}"), 400),
    };

    // 返回
    let content_type = match resolved_enc {
        OutputEncoding::Json => "application/json; charset=utf-8",
        OutputEncoding::Yaml => "application/yaml; charset=utf-8",
    };
    let mut resp = Response::ok(body)?;
    let _ = resp.headers_mut().set("Content-Type", content_type);
    Ok(resp)
}
