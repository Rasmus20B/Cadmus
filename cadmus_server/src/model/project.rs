
use crate::{ctx::Ctx, model::{ModelManager, Error, Result}};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize)]
pub struct ProjectForCreate {
    pub name: String,
}

#[derive(Deserialize)]
pub struct ProjectForUpdate {
    pub name: Option<String>,
}

pub struct ProjectBMC;

impl ProjectBMC {
    pub async fn create(
        _ctx: &Ctx,
        mm: &ModelManager,
        project_c: ProjectForCreate,
    ) -> Result<i64> {
        let db = mm.db();
        let (id, ) = sqlx::query_as::<_, (i64,)>(
            "INSERT INTO project (name) VALUES ($1) returning id"
        )
        .bind(project_c.name)
        .fetch_one(db).await?;

        Ok(id)
    }

    pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Project> {
        let db = mm.db();

        let project: Project = sqlx::query_as("SELECT * FROM projects WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::EntityNotFound { entity: "project", id })?;

        Ok(project)
    }

    pub async fn delete(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        let db = mm.db();
        let count = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?
            .rows_affected();

        if count == 0 {
            return Err(Error::EntityNotFound { entity: "project", id  })
        }

        Ok(())
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
