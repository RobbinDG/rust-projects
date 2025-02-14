use async_graphql::Enum;

#[derive(PartialEq, Eq, Clone, Copy, Enum)]
pub enum SimpleTurnChoice {
    SelectMove1,
    SelectMove2,
    SelectMove3,
    SelectMove4,
    Switch1,
    Switch2,
    Switch3,
    Switch4,
}