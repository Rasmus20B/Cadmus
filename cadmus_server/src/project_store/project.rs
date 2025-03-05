use serde::{Deserialize, Serialize};
use crate::PgPool;
use axum::extract::State;
use sqlx::{query_as, types::JsonValue};

use sqlx::prelude::FromRow;


#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ProjectInfo {
    id: i32,
    name: String,
    owner_id: i32,
    project_path: String,
    metadata: JsonValue,
}
unsafe impl Send for ProjectInfo {}
unsafe impl Sync for ProjectInfo {}

pub async fn get_projects(State(pool): State<PgPool>) -> Result<axum::Json<Vec<ProjectInfo>>, String> {
    Ok(query_as!(ProjectInfo, "SELECT id, name, owner_id, project_path, metadata FROM projects")
        .fetch_all(&pool)
        .await
        .map(|projects| axum::Json(projects))
        .unwrap())
}

pub async fn create_project(State(pool): State<PgPool>) {
}
