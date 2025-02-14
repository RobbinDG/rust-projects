use async_graphql::{Enum, SimpleObject};
use crate::side::Side;

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum AttackPhaseType {
    Damage,
    Missed,
    FaintedTarget,
    Immune,
    Protected,
    Failed,
}

#[derive(SimpleObject)]
pub struct SwitchPhaseStep {
    pub side: Side,
    pub into: i64,
}

#[derive(SimpleObject)]
pub struct AttackPhaseStep {
    pub damage_dealt: i64,
    pub type_: AttackPhaseType,
    pub effect_triggered: bool,
}

#[derive(SimpleObject)]
pub struct TurnOutcome {
    pub switch_phase: Vec<SwitchPhaseStep>,
    pub attack_phase: Vec<AttackPhaseStep>
}