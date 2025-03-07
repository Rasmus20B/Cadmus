
mod config;
mod error;
mod ctx;
mod model;
mod monitor_service;
mod store;
mod web;

pub mod _dev_utils;

pub use self::error::{Error, Result};
pub use config::config;

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

    let routes_all = Router::new()
        .merge(routes_login::routes())
        .layer(middleware::map_response(mw_response_map))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("{:<12} - {addr}\n", "LISTENING");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, routes_all)
        .await
        .unwrap();
    Ok(())
}
