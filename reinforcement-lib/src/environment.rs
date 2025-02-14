pub trait Environment<State, Action> {
    fn reset(&mut self) -> State;

    fn update(&mut self, action: Action) -> Option<State>;
}