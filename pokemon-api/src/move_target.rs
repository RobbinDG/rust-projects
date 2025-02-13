use async_graphql::Enum;

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum MoveTarget {
    SpecificMove,
    /// For "Me First" only
    SelectedMeFirst,
    /// e.g. Ally Swap
    Ally,
    /// e.g. Light Screen, Tailwind
    UsersField,
    UserOrAlly,
    /// e.g. Stealth Rock, Spikes
    OpponentsField,
    /// e.g. Recover
    User,
    /// e.g. Struggle
    RandomOpponent,
    /// e.g Earthquake, Discharge
    AllOtherPokemon,
    /// Most single-target moves
    SelectedPokemon,
    /// Most multi-target moves
    AllOpponents,
    /// Weather & Terrain moves, Room Moves (like Trick Room).
    EntireField,
    /// e.g. Life Dew
    UserAndAllies,
    /// e.g. Perish Song
    AllPokemon,
}

impl TryFrom<i64> for MoveTarget {
    type Error = &'static str;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::SpecificMove),
            2 => Ok(Self::SelectedMeFirst),
            3 => Ok(Self::Ally),
            4 => Ok(Self::UsersField),
            5 => Ok(Self::UserOrAlly),
            6 => Ok(Self::OpponentsField),
            7 => Ok(Self::User),
            8 => Ok(Self::RandomOpponent),
            9 => Ok(Self::AllOtherPokemon),
            10 => Ok(Self::SelectedPokemon),
            11 => Ok(Self::AllOpponents),
            12 => Ok(Self::EntireField),
            13 => Ok(Self::UserAndAllies),
            14 => Ok(Self::AllPokemon),
            _ => Err("Invalid move target"),
        }
    }
}