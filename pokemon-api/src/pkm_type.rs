use crate::pkm_move::PkmMove;
use crate::primitive_types::{GenerationId, PkmTypeId};
use async_graphql::{Context, SimpleObject};
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct PkmType {
    id: PkmTypeId,
    name: String,
    generation_id: GenerationId,
    damage_class: Option<i64>,
}

impl PkmType {
    pub async fn get(id: PkmTypeId, ctx: &Context<'_>) -> async_graphql::Result<Self> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let result = sqlx::query_as!(
            PkmType,
            "SELECT \
                id, identifier as name, generation_id, damage_class_id as damage_class \
                FROM types WHERE id = $1",
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(result)
    }

    pub async fn get_type_efficacy(
        &self,
        ctx: &Context<'_>,
        defender_types: &Vec<PkmType>,
    ) -> async_graphql::Result<f64> {
        struct Result {
            damage_factor: i64,
        }

        let pool = ctx.data::<Pool<Sqlite>>()?;

        let mut factor = 1.0;
        for def_type in defender_types {
            let result = sqlx::query_as!(
                Result,
                r#"SELECT damage_factor
                FROM type_efficacy
                WHERE damage_type_id = $1 AND target_type_id = $2"#,
                self.id,
                def_type.id
            )
            .fetch_one(pool)
            .await?;
            factor *= result.damage_factor as f64 / 100.0;
        }
        Ok(factor)
    }
}

impl PartialEq for &PkmType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
