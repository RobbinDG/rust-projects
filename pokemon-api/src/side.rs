use async_graphql::Enum;

/// The fanciest `bool` wrapper known to man. Used to avoid breaking bugs regarding targeting
/// of sides, even though this can be represented by a single bit.
#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum Side {
    TeamA,
    TeamB,
}

impl Side {
    pub fn target(&self, targets_opposing: bool) -> Self {
        if targets_opposing {
            self.opposing()
        } else {
            self.clone()
        }
    }

    pub fn opposing(&self) -> Self {
        match self {
            Side::TeamA => Side::TeamB,
            Side::TeamB => Side::TeamA,
        }
    }
}