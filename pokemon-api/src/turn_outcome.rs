use async_graphql::{Enum, SimpleObject};

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum TurnStepType {
    Damage,
    Fainted,
    Immune,
    Protected,
    Failed,
}

#[derive(SimpleObject)]
pub struct TurnStep {
    pub damage_dealt: i64,
    pub type_: TurnStepType,
}

#[derive(SimpleObject)]
pub struct TurnOutcome {
    pub order: Vec<TurnStep>
}