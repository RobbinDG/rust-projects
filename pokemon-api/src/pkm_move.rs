use crate::damage_class::DamageClass;
use crate::move_effect::MoveEffect;
use crate::move_stat_changes::MoveStatChange;
use crate::move_target::MoveTarget;
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
    pub id: PkmMoveId,
    name: String,
    #[graphql(skip)]
    type_id: PkmTypeId,
    pub power: Option<i64>,
    pub pp: Option<i64>,
    pub accuracy: Option<i64>,
    #[graphql(skip)]
    target_id: i64,
    #[graphql(skip)]
    damage_class: i64,
    #[graphql(skip)]
    effect_id: i64,
    pub effect_chance: Option<i64>,
}

impl PkmMove {
    pub async fn get(ctx: &Context<'_>, id: PkmMoveId) -> async_graphql::Result<Self> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let result = sqlx::query_as!(
            Self,
            "\
            SELECT id, identifier as name, type_id, power, pp, accuracy, target_id, \
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
            SELECT COALESCE(m.id, -1) id, COALESCE(m.identifier, "Error") name, COALESCE(type_id, -1) type_id, power, pp, accuracy, COALESCE(target_id, -1) target_id,
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

    pub async fn effect(&self, ctx: &Context<'_>) -> async_graphql::Result<MoveEffect> {
        Ok(MoveEffect::get(ctx, self.effect_id).await?)
    }

    pub async fn target(&self) -> async_graphql::Result<MoveTarget> {
        Ok(MoveTarget::try_from(self.target_id)?)
    }

    pub async fn stat_changes(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<MoveStatChange>> {
        MoveStatChange::get(ctx, self.id).await
    }
}
