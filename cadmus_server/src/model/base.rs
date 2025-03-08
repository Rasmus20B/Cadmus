
use sqlb::{HasFields, SqlBuilder};
use sqlx::postgres::PgRow;
use sqlx::FromRow;

use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::{Error, Result};

use super::project::Project;

pub trait DbBMC {
    const TABLE: &'static str;
}

pub async fn update<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64, data: E) -> Result<()>
where 
    MC: DbBMC,
    E: HasFields,
{
    let db = mm.db();
    let fields = data.not_none_fields();
    let count = sqlb::update()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .data(fields)
        .exec(db)
        .await?;

    if count == 0 {
        return Err(Error::EntityNotFound { entity: MC::TABLE, id })
    }

    Ok(())
}

pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where 
    MC: DbBMC,
    E: HasFields,
{
    let db = mm.db();
    let fields = data.not_none_fields();
    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one(db)
        .await?;

    Ok(id)

}
pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where 
    MC: DbBMC,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasFields,
{
    let db = mm.db();

    let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);
    let entity: E = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .and_where("id", "=", id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound { entity: MC::TABLE, id })?;

    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>> 
where 
    MC: DbBMC,
    E: for <'r> FromRow<'r, PgRow> + Unpin + Send + HasFields, 
{
    let db = mm.db();
    let entities: Vec<E> = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .order_by("id")
        .fetch_all(db)
        .await?;

    Ok(entities)
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> 
where 
    MC: DbBMC,
{
    let db = mm.db();
    let count = sqlb::delete()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .exec(db)
        .await?;

    if count == 0 {
        return Err(Error::EntityNotFound { entity: MC::TABLE, id })
    }
    Ok(())
}







