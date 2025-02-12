use async_graphql::Enum;

#[derive(PartialEq, Eq, Clone, Copy, Enum)]
pub enum TurnChoice {
    SelectMove1,
    SelectMove2,
    SelectMove3,
    SelectMove4,
}