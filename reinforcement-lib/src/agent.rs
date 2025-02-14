pub trait Agent<State, Action> {
    fn operate(&mut self, state: &State) -> Option<Action>;
}