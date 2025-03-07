
use crate::web;
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode
};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, Clone)]
pub enum Error {
    LoginFail,
    CtxExt(web::mw_auth::CtxExtError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - model::Error {self:?}", "INTO_RES");
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        response.extensions_mut().insert(self);
        response
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use web::Error::*;

        match self {
            CtxExt(..) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            )
        }
    }
}

#[derive(Debug, Clone, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    SERVICE_ERROR
}
