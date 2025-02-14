pub trait AgentFactory<Agent, Reward> {
    fn generate_initial(&self) -> Vec<Agent>;

    fn generate_new_agents(&mut self, previous: Vec<(Agent, Reward)>) -> Vec<Agent>;
}