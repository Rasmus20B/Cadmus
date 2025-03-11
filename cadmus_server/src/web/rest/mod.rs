use axum::{routing::post, Router};
use project_rest::create_project;

use crate::model::ModelManager;


mod project_rest;
mod error;

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/project", post(create_project))
        .with_state(mm)
}
