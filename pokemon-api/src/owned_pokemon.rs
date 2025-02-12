use crate::pkm_move::PkmMove;
use crate::species::Species;
use async_graphql::{Context, SimpleObject};
use rand::prelude::*;
use rand::Rng;
use crate::nature::Nature;

#[derive(SimpleObject)]
pub struct OwnedPokemon {
    pub species: Species,
    pub move_1: PkmMove,
    pub move_2: PkmMove,
    pub move_3: PkmMove,
    pub move_4: PkmMove,
    pub nature: Nature,
}

impl OwnedPokemon {
    pub async fn random(ctx: &Context<'_>) -> async_graphql::Result<Self> {
        let x = Species::max_id(ctx).await?;
        let species_id = thread_rng().gen_range(1..=x);
        let species = Species::get(ctx, species_id).await?;
        let move_pool = PkmMove::by_pkm(ctx, species_id).await?;
        let selection: Vec<_> = move_pool.into_iter().choose_multiple(&mut thread_rng(), 4).into_iter().collect();

        let x = Nature::max_id(ctx).await?;
        let nature_id = thread_rng().gen_range(1..=x);
        let nature = Nature::get(ctx, nature_id).await?;
        Ok(Self::build(species, selection, nature).ok_or(async_graphql::Error::new("Couldn't find 4 moves for species."))?)
    }

    fn build(species: Species, mut moves: Vec<PkmMove>, nature: Nature) -> Option<Self> {
        Some(Self {
            species,
            move_1: moves.pop()?,
            move_2: moves.pop()?,
            move_3: moves.pop()?,
            move_4: moves.pop()?,
            nature,
        })
    }
}