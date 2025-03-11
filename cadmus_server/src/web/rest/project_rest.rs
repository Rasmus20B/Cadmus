
use axum::{extract::State, response::{IntoResponse, Response}, Json};
use serde_json::Value;

use crate::{ctx::Ctx, model::{self, project::{Project, ProjectBMC, ProjectForCreate, ProjectForUpdate}, ModelManager}};

use super::error::{Error, Result};

#[axum::debug_handler]
pub async fn create_project(
    ctx: Ctx,
    State(mm): State<ModelManager>,
    Json(project_c): Json<ProjectForCreate>,
) -> Result<Json<Result<Project>>> {
    let id = ProjectBMC::create(&ctx, &mm, project_c)
        .await
        .map_err(|e| Error::Model(e))?;
    let project = ProjectBMC::get(&ctx, &mm, id).await
        .map_err(|e| Error::Model(e));
    Ok(axum::Json(project))
}

pub async fn list_projects(mm: ModelManager, ctx: Ctx) -> Result<Vec<Project>> {
    let projects = ProjectBMC::list(&ctx, &mm).await?;
    Ok(projects)
}

pub async fn update_project(
    ctx: Ctx,
    mm: ModelManager,
    Json(project_u): Json<ProjectForUpdate>,
) -> Result<Project> {
    let id = project_u.id.clone();
    ProjectBMC::update(&ctx, &mm, id, project_u).await?;
    let project = ProjectBMC::get(&ctx, &mm, id).await?;
    Ok(project)
}
