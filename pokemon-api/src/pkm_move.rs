use crate::damage_class::DamageClass;
use crate::move_effect::{BoundMoveEffect, MoveEffect};
use crate::pkm_type::PkmType;
use crate::primitive_types::{PkmEffectId, PkmId, PkmMoveId, PkmTypeId};
use async_graphql::{ComplexObject, Context, SimpleObject};
use sqlx;
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct PkmEffect {
    id: PkmEffectId,
    chance: i64,
}

#[derive(SimpleObject, sqlx::FromRow)]
#[graphql(complex)]
pub struct PkmMove {
    id: PkmMoveId,
    name: String,
    #[graphql(skip)]
    type_id: PkmTypeId,
    pub power: Option<i64>,
    pub pp: Option<i64>,
    pub accuracy: Option<i64>,
    target: i64,
    #[graphql(skip)]
    damage_class: i64,
    #[graphql(skip)]
    effect_id: i64,
    #[graphql(skip)]
    effect_chance: Option<i64>,
}

impl PkmMove {
    pub async fn get(ctx: &Context<'_>, id: PkmMoveId) -> async_graphql::Result<Self> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let result = sqlx::query_as!(
            Self,
            "\
            SELECT id, identifier as name, type_id, power, pp, accuracy, target_id as target, \
            damage_class_id as damage_class, effect_id, effect_chance \
            FROM moves WHERE id = $1
            ",
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(result)
    }

    pub async fn by_pkm(ctx: &Context<'_>, id: PkmId) -> async_graphql::Result<Vec<Self>> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let result = sqlx::query_as!(
            Self,
            r#"
            SELECT COALESCE(m.id, -1) id, COALESCE(m.identifier, "Error") name, COALESCE(type_id, -1) type_id, power, pp, accuracy, COALESCE(target_id, -1) target,
            COALESCE(damage_class_id, -1) damage_class, COALESCE(effect_id, -1) effect_id, effect_chance
            FROM moves m
            JOIN pokemon_moves pm ON m.id = pm.move_id AND pm.pokemon_id = $1
            WHERE pm.pokemon_id = $1
            "#,
            id
        )
        .fetch_all(pool)
        .await?;
        Ok(result)
    }

    pub async fn damage_class(&self) -> async_graphql::Result<DamageClass> {
        Ok(DamageClass::try_from(self.damage_class)?)
    }

    pub async fn pkm_type(&self, ctx: &Context<'_>) -> async_graphql::Result<PkmType> {
        Ok(PkmType::get(self.type_id, ctx).await?)
    }
}

#[ComplexObject]
impl PkmMove {
    pub async fn move_type(&self, ctx: &Context<'_>) -> async_graphql::Result<PkmType> {
        PkmType::get(self.type_id, ctx).await
    }

    pub async fn effect(&self, ctx: &Context<'_>) -> async_graphql::Result<BoundMoveEffect> {
        Ok(BoundMoveEffect::new(
            MoveEffect::get(ctx, self.effect_id).await?,
            self.effect_chance,
        ))
    }
}
