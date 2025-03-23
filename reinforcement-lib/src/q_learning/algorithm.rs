use crate::q_learning::reward_environment::RewardEnvironment;
use std::hash::Hash;
use itertools::{Itertools, MinMaxResult};

pub struct QLearning<E, const N: usize, const M: usize>
where
    E: RewardEnvironment<usize, usize>,
{
    state_space: [usize; N],
    action_space: [usize; M],
    q_table: Vec<f64>,
    environment: E,
}

impl<E, const N: usize, const M: usize> QLearning<E, N, M>
where
    E: RewardEnvironment<usize, usize>,
{
    pub fn new(state_space: [usize; N], action_space: [usize; M], env: E) -> Self {
        let total_len = state_space.len() * action_space.len();
        Self {
            state_space,
            action_space,
            q_table: Vec::with_capacity(total_len),
            environment: env,
        }
    }

    /// Runs the Q-learning algorithm on a given state-action space.
    /// Assumptions:
    ///  - Actions are chosen using the epsilon-greedy method.
    ///
    /// # Arguments
    ///
    /// * `state_space`: all possible states.
    /// * `action_space`: all possible actions.
    /// * `initial_state`: the starting state.
    /// * `goal_state`: the goal state.
    ///
    /// returns: ()
    pub fn learn(&mut self, initial_state: usize, goal_state: usize) {
        let mut q_table: Vec<f64> =
            Vec::with_capacity(self.state_space.len() * self.action_space.len());
        for _ in 0..self.state_space.len() {
            q_table.push(0.0);
        }

        self.play_episode(initial_state, goal_state);
    }

    fn play_episode(&mut self, initial_state: usize, goal_state: usize) {
        let alpha = 0.3;
        let gamma = 0.3;

        let mut state = initial_state;

        while state != goal_state {
            let action = self.epsilon_greedy(state);

        }
    }

    fn epsilon_greedy(&self, state: usize) -> usize {
        let epsilon = 0.1;
        if rand::random() < epsilon {
            self.greedy(state)
        } else {
            self.random()
        }
    }

    fn random(&self) -> usize {
        self.action_space.len()
    }

    fn greedy(&self, state: usize) -> usize {
        let from = state * self.action_space.len();
        let to = (state + 1) * self.action_space.len();
        let q_values = self.q_table.get(from..to);
        match q_values {
            Some(q_values) => {
                match q_values.iter().position_minmax() {
                    MinMaxResult::NoElements => panic!("No actions in Q table for state {state}."),
                    MinMaxResult::OneElement(i) => i,
                    MinMaxResult::MinMax(_, i) => i,
                }
            }
            None => {
                unreachable!()
            }
        }
    }
}
