use crate::primitive_types::{PkmMoveId, StatId};
use async_graphql::{Context, SimpleObject};
use sqlx::{Pool, Sqlite};
use crate::stats::Stats;

#[derive(SimpleObject)]
pub struct MoveStatChange {
    #[graphql(skip)]
    stat: StatId,
    pub change: i64,
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

    pub fn stat(&self) -> async_graphql::Result<Stats> {
        Ok(Stats::try_from(self.stat).map_err(|_| "Stat Id not recognised")?)
    }
}
