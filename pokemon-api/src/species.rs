use crate::pkm_move::PkmMove;
use crate::pkm_stats::PkmStats;
use crate::pkm_type::PkmType;
use crate::primitive_types::PkmId;
use async_graphql::{ComplexObject, Context, SimpleObject};
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Species {
    id: PkmId,
    identifier: String,
    evolves_from: Option<PkmId>,
}

impl Species {
    pub async fn get(ctx: &Context<'_>, id: i64) -> async_graphql::Result<Species> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let result = sqlx::query_as!(
            Species,
            "\
            SELECT id, identifier, evolves_from_species_id as evolves_from \
            FROM pokemon_species s WHERE id = $1
            ",
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(result)
    }

    pub async fn max_id(ctx: &Context<'_>) -> async_graphql::Result<i64> {
        struct Result {
            id: Option<i64>,
        }
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let m = sqlx::query_as!(Result, r#"SELECT MAX(id) id FROM pokemon_species"#)
            .fetch_one(pool)
            .await?;
        match m.id {
            None => Err(async_graphql::Error::new("Species table empty")),
            Some(id) => Ok(id),
        }
    }
}

#[ComplexObject]
impl Species {
    async fn pkm_type(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<PkmType>> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let result: Vec<(i64,)> = sqlx::query_as(
            "SELECT type_id FROM pokemon_types WHERE pokemon_id = $1 ORDER BY slot ASC",
        )
        .bind(self.id)
        .fetch_all(pool)
        .await?;

        let type_1 = result
            .get(0)
            .ok_or(async_graphql::Error::new("Pokemon has no first type."))?;
        let type_2 = result.get(1).map(|x| x.0);
        Ok(match type_2 {
            None => vec![PkmType::get(type_1.0, ctx).await?],
            Some(type_2) => vec![
                PkmType::get(type_1.0, ctx).await?,
                PkmType::get(type_2, ctx).await?,
            ],
        })
    }

    async fn pkm_stats(&self, ctx: &Context<'_>) -> async_graphql::Result<PkmStats> {
        PkmStats::get(self.id, ctx).await
    }

    async fn moves(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<PkmMove>> {
        PkmMove::by_pkm(ctx, self.id).await
    }
}
