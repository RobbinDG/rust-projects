use crate::nature::Nature;
use crate::pkm_move::PkmMove;
use crate::primitive_types::{NatureId, PkmId, PkmMoveId, RealisedId};
use crate::species::Species;
use async_graphql::{ComplexObject, Context, SimpleObject};
use rand::prelude::*;
use rand::Rng;
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct RealisedPokemon {
    id: RealisedId,
    #[graphql(skip)]
    species_id: PkmId,
    #[graphql(skip)]
    move_1_id: PkmMoveId,
    #[graphql(skip)]
    move_2_id: PkmMoveId,
    #[graphql(skip)]
    move_3_id: PkmMoveId,
    #[graphql(skip)]
    move_4_id: PkmMoveId,
    #[graphql(skip)]
    nature_id: NatureId,
}

impl RealisedPokemon {
    pub async fn random(ctx: &Context<'_>) -> async_graphql::Result<Self> {
        let x = Species::max_id(ctx).await?;
        let species_id = thread_rng().gen_range(1..=x);
        let move_pool = PkmMove::by_pkm(ctx, species_id).await?;
        let selection: Vec<_> = move_pool
            .into_iter()
            .choose_multiple(&mut thread_rng(), 4)
            .into_iter()
            .map(|mv| mv.id)
            .collect();

        let x = Nature::max_id(ctx).await?;
        let nature_id = thread_rng().gen_range(1..=x);
        let id = Self::get_id(ctx).await?;
        let (move_1_id, move_2_id, move_3_id, move_4_id) = Self::get_moves_from_pool(selection)
            .ok_or(async_graphql::Error::new(
                "Couldn't find 4 moves for species.",
            ))?;
        Ok(Self {
            id,
            species_id,
            move_1_id,
            move_2_id,
            move_3_id,
            move_4_id,
            nature_id,
        })
    }

    pub async fn get(ctx: &Context<'_>, id: RealisedId) -> async_graphql::Result<Self> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let result = sqlx::query_as!(
            Self,
            "SELECT id, pokemon_id as species_id, move_1 as move_1_id, move_2 as move_2_id, \
                move_3 as move_3_id, move_4 as move_4_id, nature as nature_id \
            FROM realised_pokemon WHERE id = $1",
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(result)
    }

    pub async fn insert(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let x = sqlx::query!(
            "INSERT INTO realised_pokemon VALUES (?, ?, ?, ?, ?, ?, ?)",
            self.id,
            self.species_id,
            self.move_1_id,
            self.move_2_id,
            self.move_3_id,
            self.move_4_id,
            self.nature_id,
        )
        .execute(pool)
        .await?;
        Ok(())
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

    fn get_moves_from_pool(
        mut moves: Vec<PkmMoveId>,
    ) -> Option<(PkmMoveId, PkmMoveId, PkmMoveId, PkmMoveId)> {
        Some((moves.pop()?, moves.pop()?, moves.pop()?, moves.pop()?))
    }
}

#[ComplexObject]
impl RealisedPokemon {
    pub async fn species(&self, ctx: &Context<'_>) -> async_graphql::Result<Species> {
        Species::get(ctx, self.species_id).await
    }

    pub async fn move_1(&self, ctx: &Context<'_>) -> async_graphql::Result<PkmMove> {
        PkmMove::get(ctx, self.move_1_id).await
    }
    pub async fn move_2(&self, ctx: &Context<'_>) -> async_graphql::Result<PkmMove> {
        PkmMove::get(ctx, self.move_2_id).await
    }
    pub async fn move_3(&self, ctx: &Context<'_>) -> async_graphql::Result<PkmMove> {
        PkmMove::get(ctx, self.move_3_id).await
    }
    pub async fn move_4(&self, ctx: &Context<'_>) -> async_graphql::Result<PkmMove> {
        PkmMove::get(ctx, self.move_4_id).await
    }

    pub async fn nature(&self, ctx: &Context<'_>) -> async_graphql::Result<Nature> {
        Nature::get(ctx, self.nature_id).await
    }
}
