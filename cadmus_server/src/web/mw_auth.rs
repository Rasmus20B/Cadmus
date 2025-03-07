use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::web::{AUTH_TOKEN, Error, Result};
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::{Request, request::Parts};
use axum::middleware::Next;
use axum::response::Response;
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;


pub async fn mw_ctx_require(
    ctx: Result<Ctx>,
    req: Request<Body>,
    next: Next) -> Result<Response> {

    debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");
    ctx?;
    Ok(next.run(req).await)

}

pub async fn mw_ctx_resolve(
    _mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_resolve ", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let result_ctx =
        Ctx::new(100).map_err(|e| CtxExtError::CtxCreateFail(e.to_string()));

    if result_ctx.as_ref().is_err_and(|r| !matches!(result_ctx, Err(CtxExtError::TokenNotInCookie))) {
        cookies.remove(Cookie::build(AUTH_TOKEN).into())
    }

    req.extensions_mut().insert(result_ctx);
    Ok(next.run(req).await)

}

impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;
    async fn from_request_parts<'a>(parts: &mut Parts, _state: &'a S) -> Result<Self> {
        debug!("{:<12} - Ctx", "EXTRACTOR");
        parts.extensions
            .get::<CtxExtResult>()
            .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            .map_err(Error::CtxExt)
    }
}

type CtxExtResult = core::result::Result<Ctx, CtxExtError>;

#[derive(Debug, Serialize, Clone)]
pub enum CtxExtError {
    TokenNotInCookie,
    CtxNotInRequestExt,
    CtxCreateFail(String),
}
