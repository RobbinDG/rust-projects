use crate::pkm_move::PkmMove;
use crate::realised_pokemon::RealisedPokemon;
use crate::simple_turn_choice::SimpleTurnChoice;
use async_graphql::Context;

pub enum TurnChoice {
    Move(PkmMove),
    Switch(i64),
}

impl TurnChoice {
    pub async fn from_simple(
        ctx: &Context<'_>,
        turn: SimpleTurnChoice,
        pkm: &RealisedPokemon,
    ) -> async_graphql::Result<Self> {
        match turn {
            SimpleTurnChoice::SelectMove1 => Ok(TurnChoice::Move(pkm.move_1(ctx).await?)),
            SimpleTurnChoice::SelectMove2 => Ok(TurnChoice::Move(pkm.move_2(ctx).await?)),
            SimpleTurnChoice::SelectMove3 => Ok(TurnChoice::Move(pkm.move_3(ctx).await?)),
            SimpleTurnChoice::SelectMove4 => Ok(TurnChoice::Move(pkm.move_4(ctx).await?)),
            SimpleTurnChoice::Switch1 => Ok(TurnChoice::Switch(0)),
            SimpleTurnChoice::Switch2 => Ok(TurnChoice::Switch(1)),
            SimpleTurnChoice::Switch3 => Ok(TurnChoice::Switch(2)),
            SimpleTurnChoice::Switch4 => Ok(TurnChoice::Switch(3)),
        }
    }
}
