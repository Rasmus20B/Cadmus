
use crate::{ctx::Ctx, model::{ModelManager, Error, Result}};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlb::Fields;

use super::base::{self, DbBMC};

#[derive(Fields, Debug, Clone, FromRow, Serialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

#[derive(Fields, Deserialize)]
pub struct ProjectForCreate {
    pub name: String,
}

#[derive(Fields, Deserialize)]
pub struct ProjectForUpdate {
    pub id: i64,
    pub name: Option<String>,
}

pub struct ProjectBMC;

impl DbBMC for ProjectBMC {
    const TABLE: &'static str = "projects";
}

impl ProjectBMC {
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        project_c: ProjectForCreate,
    ) -> Result<i64> {
        base::create::<Self, _>(ctx, mm, project_c).await
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Project> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Project>> {
        base::list::<Self, _>(ctx, mm).await
    }

    pub async fn update(ctx: &Ctx, mm: &ModelManager, id: i64, project_u: ProjectForUpdate) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, project_u).await

    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use crate::_dev_utils;
    use super::*;

    #[serial]
    #[tokio::test]
    async fn test_create_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_name = "test_create_ok name";

        let project_c = ProjectForCreate {
            name: fx_name.to_string(),
        };

        let id = ProjectBMC::create(&ctx, &mm, project_c).await?;

        let (name,) : (String,) = 
            sqlx::query_as("SELECT name FROM projects WHERE id == $1")
            .bind(id)
            .fetch_one(mm.db())
            .await?;

        assert_eq!(name, fx_name);

        let count = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(mm.db())
            .await?
            .rows_affected();

        assert_eq!(count, 1);
        Ok(())
    }
}
