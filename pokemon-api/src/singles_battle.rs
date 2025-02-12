use crate::damage_calc::calculate;
use crate::move_target::MoveTarget;
use crate::pkm_move::PkmMove;
use crate::pokemon_in_battle::PokemonInBattle;
use crate::primitive_types::{BattleId, RealisedId};
use crate::realised_pokemon::RealisedPokemon;
use crate::turn_choice::TurnChoice;
use crate::turn_outcome::{TurnOutcome, TurnStep, TurnStepType};
use async_graphql::{Context, SimpleObject};
use rand::Rng;
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

        let a_first = if speed_a > speed_b {
            true
        } else if speed_b > speed_a {
            false
        } else {
            rand::random::<bool>()
        };

        let outcomes = if a_first {
            Self::perform_turn(ctx, &mut active_a, &mut active_b, &move_a, &move_b).await?
        } else {
            Self::perform_turn(ctx, &mut active_b, &mut active_a, &move_b, &move_a).await?
        };
        active_a.update_for_battle(ctx, battle_id).await?;
        active_b.update_for_battle(ctx, battle_id).await?;
        Ok(TurnOutcome { order: outcomes })
    }

    async fn perform_turn(
        ctx: &Context<'_>,
        first: &mut PokemonInBattle,
        second: &mut PokemonInBattle,
        first_move: &PkmMove,
        second_move: &PkmMove,
    ) -> async_graphql::Result<Vec<TurnStep>> {
        let mut outcomes = Vec::new();
        outcomes.push(Self::execute_move(ctx, first, second, first_move).await?);

        if !second.fainted() {
            outcomes.push(Self::execute_move(ctx, second, first, second_move).await?);
        }
        Ok(outcomes)
    }

    async fn execute_move(
        ctx: &Context<'_>,
        user: &mut PokemonInBattle,
        target: &mut PokemonInBattle,
        move_used: &PkmMove,
    ) -> async_graphql::Result<TurnStep> {
        let (damage, turn) = calculate(ctx, user, move_used, target).await?;

        let move_target: MoveTarget = move_used.target(ctx).await?;
        let targeted = match move_target {
            MoveTarget::UserOrAlly => Some(user),
            MoveTarget::User => Some(user),
            MoveTarget::RandomOpponent => Some(target),
            MoveTarget::AllOtherPokemon => Some(target),
            MoveTarget::SelectedPokemon => Some(target),
            MoveTarget::AllOpponents => Some(target),
            MoveTarget::UserAndAllies => Some(user),
            _ => None,
        };

        let effect_trigger = match move_used.effect_chance {
            None => true,
            Some(c) => rand::thread_rng().gen_range(0..100) < c,
        };

        if let Some(targeted) = targeted {
            let true_damage = targeted.apply_damage(damage);
            if effect_trigger {
                let stat_changes = move_used.stat_changes(ctx).await?;
                for stat_change in stat_changes {
                    targeted
                        .stat_modifier(ctx, &stat_change.stat()?)
                        .await?
                        .raise(ctx, stat_change.change)
                        .await?;
                }
            }
            Ok(TurnStep {
                damage_dealt: true_damage,
                type_: if targeted.fainted() {
                    TurnStepType::FaintedTarget
                } else {
                    turn
                },
                effect_triggered: effect_trigger,
            })
        } else {
            Err(async_graphql::Error::new("Move target type not supported."))
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
