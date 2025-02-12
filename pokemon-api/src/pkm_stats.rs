use crate::primitive_types::{PkmTypeId, StatId};
use crate::stats::Stats;
use async_graphql::futures_util::StreamExt;
use async_graphql::{Context, SimpleObject};
use poem::EndpointExt;
use sqlx::{Pool, Sqlite};

#[derive(SimpleObject)]
pub struct PkmStats {
    pub hp: PkmStat,
    pub atk: PkmStat,
    pub def: PkmStat,
    pub s_atk: PkmStat,
    pub s_def: PkmStat,
    pub spd: PkmStat,
}

impl PkmStats {
    pub async fn get(id: PkmTypeId, ctx: &Context<'_>) -> async_graphql::Result<Self> {
        let pool = ctx.data::<Pool<Sqlite>>()?;

        let results = sqlx::query_as!(
            PkmStat,
            "SELECT stat_id, base_stat, effort \
                FROM pokemon_stats \
                WHERE pokemon_id = $1",
            id
        )
        .fetch_all(pool)
        .await?;

        let mut maybe_stats: [Option<PkmStat>; 6] = Default::default();
        for result in results.into_iter() {
            let idx = (result.stat_id.clone() - 1) as usize;
            maybe_stats[idx] = Some(result);
        }

        Self::build_self(maybe_stats).ok_or(async_graphql::Error::new("Not all stats available for pokemon."))
    }

    pub fn base_stat(&self, stat: Stats) -> i64 {
        match stat {
            Stats::Hp => self.hp.base_stat.clone(),
            Stats::Atk => self.atk.base_stat.clone(),
            Stats::Def => self.def.base_stat.clone(),
            Stats::SAtk => self.s_atk.base_stat.clone(),
            Stats::SDef => self.s_def.base_stat.clone(),
            Stats::Spd => self.spd.base_stat.clone(),
            Stats::Acc => 1,
            Stats::Eva => 1,
        }
    }

    fn build_self(stats: [Option<PkmStat>; 6]) -> Option<Self> {
        let [hp, atk, def, s_atk, s_def, spd] = stats;
        Some(Self {
            hp: hp?,
            atk: atk?,
            def: def?,
            s_atk: s_atk?,
            s_def: s_def?,
            spd: spd?,
        })
    }
}

#[derive(SimpleObject)]
pub struct PkmStat {
    pub stat_id: StatId,
    pub base_stat: i64,
    pub effort: i64,
}

impl PkmStat {}
