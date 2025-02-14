use crate::damage_calc::calculate;
use crate::move_target::MoveTarget;
use crate::pkm_move::PkmMove;
use crate::pokemon_in_battle::PokemonInBattle;
use crate::primitive_types::{BattleId, RealisedId};
use crate::side::Side;
use crate::simple_turn_choice::SimpleTurnChoice;
use crate::stats::Stats;
use crate::turn_choice::TurnChoice;
use crate::turn_outcome::{AttackPhaseStep, SwitchPhaseStep, TurnOutcome, AttackPhaseType};
use async_graphql::{Context, SimpleObject};
use rand::Rng;
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct SinglesBattle {
    id: BattleId,
    active_a: i64,
    active_b: i64,
    team_a: Vec<PokemonInBattle>,
    team_b: Vec<PokemonInBattle>,
}

impl SinglesBattle {
    pub async fn get(ctx: &Context<'_>, id: BattleId) -> async_graphql::Result<Self> {
        let team_a = PokemonInBattle::get_team(ctx, id, 0).await?;
        let team_b = PokemonInBattle::get_team(ctx, id, 1).await?;

        struct Partial {
            id: BattleId,
            active_a: i64,
            active_b: i64,
        }

        let pool = ctx.data::<Pool<Sqlite>>()?;
        let partial = sqlx::query_as!(
            Partial,
            "SELECT id, active_a, active_b FROM singles_battle WHERE id = $1",
            id
        )
            .fetch_one(pool)
            .await?;

        Ok(Self {
            id,
            active_a: partial.active_a,
            active_b: partial.active_b,
            team_a,
            team_b,
        })
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

        sqlx::query!("INSERT INTO singles_battle (id) VALUES ($1)", id)
            .execute(pool)
            .await?;

        let team_a = PokemonInBattle::insert_new_team(ctx, team_a, id, 0).await?;
        let team_b = PokemonInBattle::insert_new_team(ctx, team_b, id, 1).await?;
        Ok(Self {
            id,
            team_a,
            team_b,
            active_a: 0,
            active_b: 0,
        })
    }

    pub async fn play_turn(
        &mut self,
        ctx: &Context<'_>,
        turn_a: SimpleTurnChoice,
        turn_b: SimpleTurnChoice,
    ) -> async_graphql::Result<TurnOutcome> {
        let turn_order = self.resolve_turn_order(ctx, turn_a, turn_b).await?;

        // TODO validate choices (switch candidate exists & is not fainted + move exists)

        // Switch phase (if any)
        let mut switch_phase = Vec::new();
        for (is_team_a, cur_choice) in &turn_order {
            if let TurnChoice::Switch(i) = cur_choice {
                let step = self.switch(*is_team_a, i.clone())?;
                switch_phase.push(step);
            }
        }

        // Attack phase
        let mut attack_phase = Vec::new();
        for (user_side, cur_choice) in &turn_order {
            if let TurnChoice::Move(mv) = cur_choice {
                let cur_pkm = self.get_active(*user_side)?;
                if !cur_pkm.fainted() {
                    attack_phase.push(self.execute_move(ctx, *user_side, mv).await?);
                }
            }
        }

        self.update(ctx).await?;
        Ok(TurnOutcome {
            switch_phase,
            attack_phase,
        })
    }

    async fn resolve_turn_order(
        &self,
        ctx: &Context<'_>,
        turn_a: SimpleTurnChoice,
        turn_b: SimpleTurnChoice,
    ) -> async_graphql::Result<[(Side, TurnChoice); 2]> {
        let active_a = self
            .team_a
            .get(self.active_a as usize)
            .ok_or(async_graphql::Error::new("Team empty"))?;
        let active_b = self
            .team_b
            .get(self.active_b as usize)
            .ok_or(async_graphql::Error::new("Team empty"))?;
        let pkm_a = active_a.pokemon(ctx).await?;
        let pkm_b = active_b.pokemon(ctx).await?;
        let choice_a = TurnChoice::from_simple(ctx, turn_a, &pkm_a).await?;
        let choice_b = TurnChoice::from_simple(ctx, turn_b, &pkm_b).await?;
        let speed_a = active_a.stat(ctx, Stats::Spd).await?;
        let speed_b = active_b.stat(ctx, Stats::Spd).await?;

        let a_first = if speed_a > speed_b {
            true
        } else if speed_b > speed_a {
            false
        } else {
            rand::random::<bool>()
        };
        let turn_order = if a_first {
            [(Side::TeamA, choice_a), (Side::TeamB, choice_b)]
        } else {
            [(Side::TeamB, choice_b), (Side::TeamA, choice_a)]
        };
        Ok(turn_order)
    }

    async fn execute_move(
        &mut self,
        ctx: &Context<'_>,
        user_side: Side,
        move_used: &PkmMove,
    ) -> async_graphql::Result<AttackPhaseStep> {
        let move_target: MoveTarget = move_used.target(ctx).await?;
        let targeted_opponent = Self::resolve_target_side(user_side, move_target);

        let effect_trigger = match move_used.effect_chance {
            None => true,
            Some(c) => rand::thread_rng().gen_range(0..100) < c,
        };

        if let Some(targeted) = targeted_opponent {
            // A block is required here to release the immutable reference before immediately
            // after calculating the damage such that we can mutably reference the target after.
            let (damage, turn) = {
                let user = self.get_active(user_side)?;
                let targeted = self.get_active(targeted)?;

                calculate(ctx, user, move_used, targeted).await?
            };

            let targeted = self.get_active_mut(targeted)?;
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
            Ok(AttackPhaseStep {
                damage_dealt: true_damage,
                type_: if targeted.fainted() {
                    AttackPhaseType::FaintedTarget
                } else {
                    turn
                },
                effect_triggered: effect_trigger,
            })
        } else {
            Err(async_graphql::Error::new("Move target type not supported."))
        }
    }

    fn resolve_target_side(user_side: Side, move_target: MoveTarget) -> Option<Side> {
        match move_target {
            MoveTarget::UserOrAlly => Some(user_side),
            MoveTarget::User => Some(user_side),
            MoveTarget::RandomOpponent => Some(user_side.opposing()),
            MoveTarget::AllOtherPokemon => Some(user_side.opposing()),
            MoveTarget::SelectedPokemon => Some(user_side.opposing()),
            MoveTarget::AllOpponents => Some(user_side.opposing()),
            MoveTarget::UserAndAllies => Some(user_side),
            _ => None,
        }
    }

    fn get_active_mut(&mut self, side: Side) -> async_graphql::Result<&mut PokemonInBattle> {
        match side {
            Side::TeamA => self.team_a.get_mut(self.active_a as usize),
            Side::TeamB => self.team_b.get_mut(self.active_b as usize),
        }
            .ok_or(async_graphql::Error::new("Current target invalid"))
    }

    fn switch(&mut self, side: Side, switch_target: i64) -> async_graphql::Result<SwitchPhaseStep> {
        let target: Option<&mut PokemonInBattle> = match side {
            Side::TeamA => self.team_a.get_mut(switch_target as usize),
            Side::TeamB => self.team_b.get_mut(switch_target as usize),
        };

        match target {
            None => Err(async_graphql::Error::new("Switch target does not exist.")),
            Some(target) => {
                if !target.fainted() {
                    match side {
                        Side::TeamA => self.active_a = switch_target,
                        Side::TeamB => self.active_b = switch_target,
                    }
                    Ok(SwitchPhaseStep {
                        side,
                        into: switch_target,
                    })
                } else {
                    Err(async_graphql::Error::new("Switch target does not exist."))
                }
            }
        }
    }

    async fn update(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
        self.get_active(Side::TeamA)?
            .update_for_battle(ctx, self.id)
            .await?;
        self.get_active(Side::TeamB)?
            .update_for_battle(ctx, self.id)
            .await?;

        let pool = ctx.data::<Pool<Sqlite>>()?;
        sqlx::query!(
            "UPDATE singles_battle SET active_a = $1, active_b = $2 WHERE id = $3",
            self.active_a,
            self.active_b,
            self.id
        )
            .execute(pool)
            .await?;
        Ok(())
    }

    fn get_active(&self, side: Side) -> async_graphql::Result<&PokemonInBattle> {
        match side {
            Side::TeamA => self.team_a.get(self.active_a as usize),
            Side::TeamB => self.team_b.get(self.active_b as usize),
        }.ok_or(async_graphql::Error::new("Current target invalid"))
    }
}
