use crate::primitive_types::{BattleId, RealisedId};
use async_graphql::{Context, SimpleObject};
use sqlx::{Pool, Sqlite};
use std::cmp::{max, min};

#[derive(SimpleObject)]
pub struct StatModifier {
    stages: i64,
    stat: i64,
}

impl StatModifier {
    pub fn raise(&mut self, change: i64) {
        self.stages = min(max(-6, self.stages + change), 6);
    }

    pub fn apply(&self, base_stat: i64) -> i64 {
        let factor = if self.stages >= 0 {
            (self.stages + 2) as f64 / 2.0
        } else {
            2.0 / (-self.stages + 2) as f64
        };
        (base_stat as f64 * factor).round() as i64
    }

    pub async fn get(
        ctx: &Context<'_>,
        battle_id: BattleId,
        pkm_id: RealisedId,
    ) -> async_graphql::Result<StatModifier> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let result = sqlx::query_as!(
            Self,
            "SELECT COALESCE(stages, 0) stages, stat \
            FROM pokemon_modifiers \
            WHERE battle_id = $1 AND realised_pokemon_id = $2",
            battle_id,
            pkm_id
        )
        .fetch_one(pool)
        .await?;
        Ok(result)
    }
}