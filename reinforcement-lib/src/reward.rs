pub trait Reward: Ord {
    fn minimal() -> Self;
}