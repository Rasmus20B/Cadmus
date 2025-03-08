use crate::crypt::token::{validate_web_token, Token};
use crate::ctx::Ctx;
use crate::model::user::{UserBMC, UserForAuth};
use crate::model::ModelManager;
use crate::web::{set_token_cookie, Error, Result, AUTH_TOKEN};
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
    mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next
) -> Result<Response> {
    async fn _ctx_resolve(mm: State<ModelManager>, cookies: &Cookies) -> CtxExtResult {
        let token = cookies
            .get(AUTH_TOKEN)
            .map(|c| c.value().to_string())
            .ok_or(CtxExtError::TokenNotInCookie)?;

        let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

        let user: UserForAuth = 
            UserBMC::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
            .await
            .map_err(|e| CtxExtError::ModelAccessError(e.to_string()))?
            .ok_or(CtxExtError::UserNotFound)?;

        validate_web_token(&token, &user.token_salt.to_string())
            .map_err(|_| CtxExtError::FailValidate)?;

        set_token_cookie(cookies, &user.username, &user.token_salt.to_string())
            .map_err(|_| CtxExtError::CannotSetTokenCookie)?;

        Ctx::new(user.id).map_err(|e| CtxExtError::CtxCreateFail(e.to_string()))
    }
    debug!("{:<12} - mw_ctx_resolve ", "MIDDLEWARE");

    let ctx_ext_result = _ctx_resolve(mm, &cookies).await;

    if ctx_ext_result.is_err() 
    && !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN))
    }

    req.extensions_mut().insert(ctx_ext_result);

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
    TokenWrongFormat,

    UserNotFound,
    ModelAccessError(String),
    FailValidate,
    CannotSetTokenCookie,

    CtxNotInRequestExt,
    CtxCreateFail(String),
}
