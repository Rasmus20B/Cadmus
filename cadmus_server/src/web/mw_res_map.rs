
use crate::ctx::Ctx;
use crate::web;
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

pub async fn mw_response_map(
    ctx: Ctx,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    debug!("{:<12} - mw_response_map", "RES_MAPPER");

    let uuid = Uuid::new_v4();
    let web_error = res.extensions().get::<web::Error>();
    let client_status_error = web_error.map(|se| se.client_status_and_error());

    let error_response =
        client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });
            debug!("CLIENT ERROR BODY:\n{client_error_body}");
            (*status_code, Json(client_error_body)).into_response()
        });

    let client_error = client_status_error.unzip().1;

    println!("\n");
    error_response.unwrap_or(res)
}
