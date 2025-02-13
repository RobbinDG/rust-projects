use crate::damage_calc::calculate;
use crate::pkm_move::PkmMove;
use crate::pokemon_in_battle::PokemonInBattle;
use crate::primitive_types::{BattleId, RealisedId};
use crate::realised_pokemon::RealisedPokemon;
use crate::turn_choice::TurnChoice;
use crate::turn_outcome::{TurnOutcome, TurnStep, TurnStepType};
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
        let team_b = PokemonInBattle::insert_new_team(ctx, team_b, id, 1).await?;
        Ok(Self { id, team_a, team_b })
    }

    pub async fn play_turn(
        ctx: &Context<'_>,
        battle_id: BattleId,
        turn_a: TurnChoice,
        turn_b: TurnChoice,
    ) -> async_graphql::Result<TurnOutcome> {
        let mut battle = Self::get(ctx, battle_id).await?;

        let mut active_a = battle
            .team_a
            .get_mut(0)
            .ok_or(async_graphql::Error::new("Team empty"))?;
        let mut active_b = battle
            .team_b
            .get_mut(0)
            .ok_or(async_graphql::Error::new("Team empty"))?;
        let mut pkm_a = active_a.pokemon(ctx).await?;
        let mut pkm_b = active_b.pokemon(ctx).await?;
        let move_a = Self::get_selected_move(ctx, turn_a, &mut pkm_a).await?;
        let move_b = Self::get_selected_move(ctx, turn_b, &mut pkm_b).await?;
        let speed_a = pkm_a
            .species(ctx)
            .await?
            .pkm_stats(ctx)
            .await?
            .spd
            .base_stat;
        let speed_b = pkm_b
            .species(ctx)
            .await?
            .pkm_stats(ctx)
            .await?
            .spd
            .base_stat;

        let (damage_a_b, outcome_a_b) = calculate(ctx, &active_a, &move_a, &active_b).await?;
        let (damage_b_a, outcome_b_a) = calculate(ctx, &active_b, &move_b, &active_a).await?;

        let a_first = if speed_a > speed_b {
            true
        } else if speed_b > speed_a {
            false
        } else {
            rand::random::<bool>()
        };

        let outcomes = if a_first {
            Self::perform_turn(
                &mut active_a,
                &mut active_b,
                damage_a_b,
                outcome_a_b,
                damage_b_a,
                outcome_b_a,
            )
        } else {
            Self::perform_turn(
                &mut active_b,
                &mut active_a,
                damage_b_a,
                outcome_b_a,
                damage_a_b,
                outcome_a_b,
            )
        };
        active_a.update_for_battle(ctx, battle_id).await?;
        active_b.update_for_battle(ctx, battle_id).await?;
        Ok(TurnOutcome { order: outcomes })
    }

    fn perform_turn(
        first: &mut &mut PokemonInBattle,
        second: &mut &mut PokemonInBattle,
        first_damage: u32,
        first_type: TurnStepType,
        second_damage: u32,
        second_type: TurnStepType,
    ) -> Vec<TurnStep> {
        let mut outcomes = Vec::new();
        outcomes.push(Self::attack_target(second, first_damage, first_type));

        if !second.fainted() {
            outcomes.push(Self::attack_target(first, second_damage, second_type));
        }
        outcomes
    }

    fn attack_target(
        target: &mut PokemonInBattle,
        damage: u32,
        second_type: TurnStepType,
    ) -> TurnStep {
        let true_damage = target.apply_damage(damage);
        if target.fainted() {
            TurnStep {
                damage_dealt: true_damage,
                type_: TurnStepType::FaintedTarget,
            }
        } else {
            TurnStep {
                damage_dealt: true_damage,
                type_: second_type,
            }
        }
    }

    async fn get_selected_move(
        ctx: &Context<'_>,
        turn: TurnChoice,
        pkm: &mut RealisedPokemon,
    ) -> async_graphql::Result<PkmMove> {
        match turn {
            TurnChoice::SelectMove1 => pkm.move_1(ctx).await,
            TurnChoice::SelectMove2 => pkm.move_2(ctx).await,
            TurnChoice::SelectMove3 => pkm.move_3(ctx).await,
            TurnChoice::SelectMove4 => pkm.move_4(ctx).await,
        }
    }
}
