use crate::primitive_types::NatureId;
use async_graphql::{Context, SimpleObject};
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct Nature {
    id: NatureId,
    name: String,
    decreased: i64,
    increased: i64,
}

impl Nature {
    pub async fn get(ctx: &Context<'_>, id: NatureId) -> async_graphql::Result<Self> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let nature = sqlx::query_as!(
            Nature,
            r#"SELECT id, identifier as name, decreased_stat_id as decreased,
        increased_stat_id as increased FROM natures WHERE id = $1"#,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(nature)
    }

    pub async fn max_id(ctx: &Context<'_>) -> async_graphql::Result<NatureId> {
        struct Result {
            id: Option<NatureId>,
        }

        let pool = ctx.data::<Pool<Sqlite>>()?;
        let result = sqlx::query_as!(Result, r#"SELECT MAX(id) id FROM natures"#)
            .fetch_one(pool)
            .await?;
        match result.id {
            None => Err(async_graphql::Error::new("Nature table empty")),
            Some(id) => Ok(id),
        }
    }
}
