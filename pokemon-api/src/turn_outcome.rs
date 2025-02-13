use async_graphql::{Enum, SimpleObject};

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum TurnStepType {
    Damage,
    Missed,
    FaintedTarget,
    Immune,
    Protected,
    Failed,
}

#[derive(SimpleObject)]
pub struct TurnStep {
    pub damage_dealt: i64,
    pub type_: TurnStepType,
    pub effect_triggered: bool,
}

#[derive(SimpleObject)]
pub struct TurnOutcome {
    pub order: Vec<TurnStep>
}