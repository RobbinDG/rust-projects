use crate::primitive_types::{AbilityId, PkmId};
use async_graphql::{Context, SimpleObject};
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct Ability {
    id: AbilityId,
    name: String,
    effect: String,
}

impl Ability {
    pub async fn get(ctx: &Context<'_>, id: AbilityId) -> async_graphql::Result<Ability> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let ability = sqlx::query_as!(
            Ability,
            r#"SELECT id, identifier as name, COALESCE(ae.effect, "") effect
            FROM abilities a
                JOIN ability_effects ae ON a.id = ae.ability_id
            WHERE a.id = $1"#,
            id
        )
            .fetch_one(pool)
            .await?;
        Ok(ability)
    }
}

#[derive(SimpleObject)]
pub struct AbilityPool {
    ability_1: Ability,
    ability_2: Option<Ability>,
    hidden: Option<Ability>,
}

struct AbilityInSlot {
    id: AbilityId,
    slot: i64,
    is_hidden: i64,
}

impl AbilityPool {
    pub async fn get(ctx: &Context<'_>, id: PkmId) -> async_graphql::Result<AbilityPool> {
        let pool = ctx.data::<Pool<Sqlite>>()?;
        let abilities = sqlx::query_as!(
            AbilityInSlot,
            r#"SELECT ability_id as id, slot, is_hidden FROM pokemon_abilities where pokemon_id = $1 ORDER BY slot ASC"#,
            id
        ).fetch_all(pool).await?;
        let id_1 = abilities
            .get(0)
            .ok_or(async_graphql::Error::new("Species has no first ability"))?;
        let id_2 = abilities.get(1);
        let id_3 = abilities.get(2);

        let (id_2, hidden) = match (id_2, id_3) {
            (
                Some(AbilityInSlot {
                         id,
                         is_hidden,
                         ..
                     }),
                _,
            ) if is_hidden.clone() != 0 => (None, Some(id.clone())),
            (a, b) => {
                (a.map(|a| a.id), b.map(|a| a.id))
            }
        };

        Ok(Self {
            ability_1: Ability::get(ctx, id_1.id).await?,
            ability_2: match id_2 {
                None => None,
                Some(a) => Some(Ability::get(ctx, a).await?),
            },
            hidden: match hidden {
                None => None,
                Some(a) => Some(Ability::get(ctx, a).await?),
            },
        })
    }
}
