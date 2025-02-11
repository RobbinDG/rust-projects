use crate::primitive_types::PkmEffectId;
use async_graphql::{Context, SimpleObject};
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct MoveEffect {
    id: PkmEffectId,
    short: String,
    long: String,
}

impl MoveEffect {
    pub async fn get(ctx: &Context<'_>, id: PkmEffectId) -> async_graphql::Result<Self> {
        let pool = ctx.data::<Pool<Sqlite>>().unwrap();

        let result = sqlx::query_as!(
            MoveEffect,
            r#"SELECT move_effect_id as id, short_effect as short, effect as long FROM move_effects WHERE move_effect_id = $1"#,
            id,
        ).fetch_one(pool).await?;
        Ok(result)
    }
}

#[derive(SimpleObject)]
pub struct BoundMoveEffect {
    effect: MoveEffect,
    chance: Option<i64>,
}

impl BoundMoveEffect {
    pub fn new(effect: MoveEffect, chance: Option<i64>) -> Self {
        Self {
            effect,
            chance,
        }
    }
}
