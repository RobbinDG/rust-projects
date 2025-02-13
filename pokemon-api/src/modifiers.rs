use crate::primitive_types::{BattleId, RealisedId};
use async_graphql::{Context, SimpleObject};
use sqlx::{Pool, Sqlite};
use std::cmp::{max, min};
use crate::stats::Stats;

#[derive(SimpleObject)]
pub struct StatModifier {
    stages: i64,
    stat: i64,
    #[graphql(skip)]
    pkm_id: RealisedId,
    #[graphql(skip)]
    battle_id: BattleId,
}

impl StatModifier {
    pub async fn raise(&mut self, ctx: &Context<'_>, change: i64) -> async_graphql::Result<()> {
        self.stages = min(max(-6, self.stages + change), 6);
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let result = sqlx::query!(
            "INSERT OR REPLACE INTO pokemon_modifiers(battle_id, realised_pokemon_id, stat, stages)\
            VALUES ($1, $2, $3, $4)",
            self.battle_id,
            self.pkm_id,
            self.stat,
            self.stages,
        )
        .execute(pool)
        .await?;
        Ok(())
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
        stat: &Stats,
    ) -> async_graphql::Result<StatModifier> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let stat_id = stat.id();
        let result = sqlx::query_as!(
            Self,
            "SELECT COALESCE(stages, 0) stages, stat, realised_pokemon_id as pkm_id, battle_id \
            FROM pokemon_modifiers \
            WHERE battle_id = $1 AND realised_pokemon_id = $2 AND stat = $3",
            battle_id,
            pkm_id,
            stat_id,
        )
        .fetch_optional(pool)
        .await?;

        match result {
            Some(result) => Ok(result),
            None => Ok(StatModifier {
                stages: 0,
                stat: stat_id,
                battle_id,
                pkm_id,
            })
        }

    }
}