use crate::primitive_types::{GenerationId, PkmTypeId};
use async_graphql::{Context, SimpleObject};
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct PkmType {
    id: PkmTypeId,
    name: String,
    generation_id: GenerationId,
    damage_class: Option<f64>,
}

impl PkmType {
    pub async fn get(id: PkmTypeId, ctx: &Context<'_>) -> async_graphql::Result<Self> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let result = sqlx::query_as!(
            PkmType,
            "SELECT \
                id, identifier as name, generation_id, damage_class_id as damage_class \
                FROM types WHERE id = $1",
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(result)
    }
}
