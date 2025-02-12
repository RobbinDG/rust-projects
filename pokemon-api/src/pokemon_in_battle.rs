use crate::primitive_types::{BattleId, RealisedId};
use crate::realised_pokemon::RealisedPokemon;
use async_graphql::{ComplexObject, Context, SimpleObject};
use sqlx::{Pool, Sqlite};
use std::cmp::max;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PokemonInBattle {
    realised_id: RealisedId,
    remaining_hp: i64,
}

impl PokemonInBattle {
    pub async fn get(
        ctx: &Context<'_>,
        realised_id: RealisedId,
        battle_id: BattleId,
    ) -> async_graphql::Result<Self> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let pkm = sqlx::query_as!(
            PokemonInBattle,
            "SELECT remaining_hp, realised_pokemon_id as realised_id \
            FROM pokemon_in_battle \
            WHERE battle_id = $1 AND realised_pokemon_id = $2 \
            ORDER BY position ASC",
            battle_id,
            realised_id,
        )
        .fetch_one(pool)
        .await?;
        Ok(pkm)
    }

    pub async fn get_team(
        ctx: &Context<'_>,
        battle_id: BattleId,
        team_id: i64,
    ) -> async_graphql::Result<Vec<Self>> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let team: Vec<_> = sqlx::query_as!(
            Self,
            "SELECT realised_pokemon_id as realised_id, remaining_hp \
            FROM pokemon_in_battle \
            WHERE battle_id = $1 AND team = $2 \
            ORDER BY position ASC",
            battle_id,
            team_id,
        )
        .fetch_all(pool)
        .await?;
        Ok(team)
    }

    pub async fn insert_new_team(
        ctx: &Context<'_>,
        team: Vec<RealisedId>,
        battle_id: BattleId,
        team_id: i64,
    ) -> async_graphql::Result<Vec<Self>> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let mut team_done = Vec::new();
        for (i, member) in team.iter().enumerate() {
            let max_hp = RealisedPokemon::get(ctx, member.clone())
                .await?
                .species(ctx)
                .await?
                .pkm_stats(ctx)
                .await?
                .hp
                .base_stat;
            let idx = i as i64;
            sqlx::query!(
                "INSERT INTO pokemon_in_battle (battle_id, realised_pokemon_id, team, position, remaining_hp) \
                VALUES ($1, $2, $3, $4, $5)",
                battle_id, member, team_id, idx, max_hp
            ).execute(pool).await?;
            team_done.push(PokemonInBattle {
                realised_id: member.clone(),
                remaining_hp: max_hp,
            });
        }
        Ok(team_done)
    }

    pub async fn update_for_battle(&self, ctx: &Context<'_>, battle_id: BattleId) -> async_graphql::Result<()> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let _ = sqlx::query!(
            "UPDATE pokemon_in_battle SET remaining_hp = $1 WHERE battle_id = $2 AND realised_pokemon_id = $3",
            self.remaining_hp,
            battle_id,
            self.realised_id,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub fn apply_damage(&mut self, damage: u32) {
        self.remaining_hp = max(self.remaining_hp - damage as i64, 0);
    }

    pub fn fainted(&self) -> bool {
        self.remaining_hp <= 0
    }
}

#[ComplexObject]
impl PokemonInBattle {
    pub async fn pokemon(&self, ctx: &Context<'_>) -> async_graphql::Result<RealisedPokemon> {
        RealisedPokemon::get(ctx, self.realised_id).await
    }
}
