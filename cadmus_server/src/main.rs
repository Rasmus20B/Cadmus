
mod config;
mod crypt;
mod error;
mod ctx;
mod model;
mod monitor_service;
mod utils;
mod web;

pub mod _dev_utils;

pub use self::error::{Error, Result};
use axum::response::Html;
pub use config::config;
use web::mw_auth::mw_ctx_require;

use crate::model::ModelManager;
use crate::web::mw_auth::mw_ctx_resolve;
use crate::web::mw_res_map::mw_response_map;
use crate::web::{routes_login, routes_static};
use axum::{middleware, Router};
use tracing::info;
use tracing_subscriber::EnvFilter;
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    //_dev_utils::init_dev().await;

    let mm = ModelManager::new().await?;

    let routes_hello = Router::new()
        .route("/hello", axum::routing::get(|| async { Html("Hello World") }))
        .route_layer(middleware::from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(routes_login::routes(mm.clone()))
        .merge(routes_hello)
        .layer(middleware::map_response(mw_response_map))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("{:<12} - {addr}\n", "LISTENING");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, routes_all)
        .await
        .unwrap();
    Ok(())
}
