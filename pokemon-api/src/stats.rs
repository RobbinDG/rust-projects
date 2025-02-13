use async_graphql::Enum;
use crate::primitive_types::StatId;

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum Stats {
    Hp,
    Atk,
    Def,
    SAtk,
    SDef,
    Spd,
    Acc,
    Eva,
}

impl Stats {
    pub fn id(&self) -> StatId {
        match self {
            Stats::Hp => 1,
            Stats::Atk => 2,
            Stats::Def => 3,
            Stats::SAtk => 4,
            Stats::SDef => 5,
            Stats::Spd => 6,
            Stats::Acc => 7,
            Stats::Eva => 8,
        }
    }
}

impl TryFrom<StatId> for Stats {
    type Error = ();

    fn try_from(value: StatId) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Stats::Hp),
            2 => Ok(Stats::Atk),
            3 => Ok(Stats::Def),
            4 => Ok(Stats::SAtk),
            5 => Ok(Stats::SDef),
            6 => Ok(Stats::Spd),
            7 => Ok(Stats::Acc),
            8 => Ok(Stats::Eva),
            _ => Err(()),
        }
    }
}