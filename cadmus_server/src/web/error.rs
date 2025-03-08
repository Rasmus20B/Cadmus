
use crate::{crypt, model, web};
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode
};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag ="type", content="data")]
pub enum Error {
    LoginFailUsernameNotFound,
    LoginFailUserHasNoPwd { user_id: i64 },
    LoginFailPwdNotMatching { user_id: i64 },

    Crypt(crypt::Error),
    Model(model::Error),
    CtxExt(web::mw_auth::CtxExtError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - model::Error {self:?}", "INTO_RES");
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        response.extensions_mut().insert(self.to_string());
        response
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<model::Error> for Error {
    fn from(value: model::Error) -> Self {
        Self::Model(value)
    }
}

impl From<crypt::Error> for Error {
    fn from(value: crypt::Error) -> Self {
        Self::Crypt(value)
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use web::Error::*;

        match self {
            LoginFailUsernameNotFound
            | LoginFailUserHasNoPwd { .. }
            | LoginFailPwdNotMatching { .. } => {
                (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL)
            },
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
