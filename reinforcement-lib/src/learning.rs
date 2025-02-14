use crate::agent::Agent;
use crate::agent_factory::AgentFactory;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::reward::Reward;
use std::cmp::max;
use std::marker::PhantomData;

pub struct Learning<Ag, R, S, Ac, Factory, Env, Int>
where
    R: Reward,
    Ag: Agent<S, Ac>,
    Factory: AgentFactory<Ag, R>,
    Env: Environment<S, Ac>,
    Int: Interpreter<S, R>,
{
    factory: Factory,
    environment: Env,
    interpreter: Int,
    reward_marker: PhantomData<R>,
    agent_marker: PhantomData<Ag>,
    state_marker: PhantomData<S>,
    action_marker: PhantomData<Ac>,
}

impl<Ag, R, S, Ac, Factory, Env, Int> Learning<Ag, R, S, Ac, Factory, Env, Int>
where
    R: Reward,
    Ag: Agent<S, Ac>,
    Factory: AgentFactory<Ag, R>,
    Env: Environment<S, Ac>,
    Int: Interpreter<S, R>,
{
    pub fn learn(mut self) {
        let mut agents = self.factory.generate_initial();

        loop {
            let mut agent_rewards = Vec::with_capacity(agents.len());
            for mut agent in agents.into_iter() {
                let mut state = Some(self.environment.reset());
                let mut reward = R::minimal();
                while let Some(s) = state {
                    let current_reward = self.interpreter.calculate_reward(&s);
                    reward = max(reward, current_reward);

                    match agent.operate(&s) {
                        None => break,
                        Some(action) => {
                            state = self.environment.update(action);
                        }
                    }
                }
                agent_rewards.push((agent, reward));
            }

            agents = self.factory.generate_new_agents(agent_rewards);
        }
    }
}
