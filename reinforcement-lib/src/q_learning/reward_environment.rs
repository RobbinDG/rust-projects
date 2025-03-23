pub trait RewardEnvironment<S, A> {
    fn reward(&self, state: &S, action: &A) -> f64;
}