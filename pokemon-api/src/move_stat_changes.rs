use crate::primitive_types::{PkmMoveId, StatId};
use async_graphql::{Context, SimpleObject};
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct MoveStatChange {
    stat: StatId,
    change: i64,
}

impl MoveStatChange {
    pub async fn get(
        ctx: &Context<'_>,
        move_id: PkmMoveId,
    ) -> async_graphql::Result<Vec<MoveStatChange>> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let result = sqlx::query_as!(
            MoveStatChange,
            "SELECT stat_id as stat, change FROM move_stat_changes WHERE move_id = $1",
            move_id,
        )
        .fetch_all(pool)
        .await?;
        Ok(result)
    }
}
