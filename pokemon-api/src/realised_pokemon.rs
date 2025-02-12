use crate::nature::Nature;
use crate::pkm_move::PkmMove;
use crate::primitive_types::RealisedId;
use crate::species::Species;
use async_graphql::{Context, SimpleObject};
use rand::prelude::*;
use rand::Rng;
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct RealisedPokemon {
    pub id: RealisedId,
    pub species: Species,
    pub move_1: PkmMove,
    pub move_2: PkmMove,
    pub move_3: PkmMove,
    pub move_4: PkmMove,
    pub nature: Nature,
}

impl RealisedPokemon {
    pub async fn random(ctx: &Context<'_>) -> async_graphql::Result<Self> {
        let x = Species::max_id(ctx).await?;
        let species_id = thread_rng().gen_range(1..=x);
        let species = Species::get(ctx, species_id).await?;
        let move_pool = PkmMove::by_pkm(ctx, species_id).await?;
        let selection: Vec<_> = move_pool
            .into_iter()
            .choose_multiple(&mut thread_rng(), 4)
            .into_iter()
            .collect();

        let x = Nature::max_id(ctx).await?;
        let nature_id = thread_rng().gen_range(1..=x);
        let nature = Nature::get(ctx, nature_id).await?;
        let id = Self::get_id(ctx).await?;
        Ok(
            Self::build(id, species, selection, nature).ok_or(async_graphql::Error::new(
                "Couldn't find 4 moves for species.",
            ))?,
        )
    }

    async fn get_id(ctx: &Context<'_>) -> async_graphql::Result<RealisedId> {
        struct Result {
            id: RealisedId,
        }
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let id = sqlx::query_as!(
            Result,
            "SELECT COALESCE(MAX(id), 0) id FROM realised_pokemon"
        )
        .fetch_one(pool)
        .await?;
        Ok(id.id + 1)
    }

    fn build(id: RealisedId, species: Species, mut moves: Vec<PkmMove>, nature: Nature) -> Option<Self> {
        Some(Self {
            id,
            species,
            move_1: moves.pop()?,
            move_2: moves.pop()?,
            move_3: moves.pop()?,
            move_4: moves.pop()?,
            nature,
        })
    }
}
