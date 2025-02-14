pub trait Interpreter<State, Reward> {
    fn calculate_reward(&self, state: &State) -> Reward;
}