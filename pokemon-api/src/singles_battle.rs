use crate::pokemon_in_battle::PokemonInBattle;
use crate::primitive_types::{BattleId, RealisedId};
use async_graphql::{Context, SimpleObject};
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct SinglesBattle {
    id: BattleId,
    team_a: Vec<PokemonInBattle>,
    team_b: Vec<PokemonInBattle>,
}

impl SinglesBattle {
    pub async fn get(ctx: &Context<'_>, id: BattleId) -> async_graphql::Result<Self> {
        let team_a = PokemonInBattle::get_team(ctx, id, 0).await?;
        let team_b = PokemonInBattle::get_team(ctx, id, 1).await?;

        Ok(Self { id, team_a, team_b })
    }

    pub async fn insert(
        ctx: &Context<'_>,
        team_a: Vec<RealisedId>,
        team_b: Vec<RealisedId>,
    ) -> async_graphql::Result<Self> {
        struct Result {
            id: BattleId,
        }
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let id = sqlx::query_as!(
            Result,
            "SELECT COALESCE(MAX(battle_id), -1) + 1 id FROM pokemon_in_battle"
        )
        .fetch_one(pool)
        .await?
        .id;

        let team_a = PokemonInBattle::insert_new_team(ctx, team_a, id, 0).await?;
        let team_b = PokemonInBattle::insert_new_team(ctx, team_b, id, 0).await?;
        Ok(Self { id, team_a, team_b })
    }

    // pub async fn play_turn(battle_id: BattleId, turn_a: TurnChoice, turn_b: TurnChoice) -> Self {}
}
