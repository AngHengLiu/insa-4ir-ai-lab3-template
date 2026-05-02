use std::time::Instant;
use rand::seq::IndexedRandom;

use crate::engine::Engine;
use crate::mcts::{Output};

use super::board::*;

pub struct RandomEngine {
}

impl RandomEngine {
    pub fn new() -> RandomEngine {
        RandomEngine {  }
    }
}

impl Engine for RandomEngine {

    // Select a random action between all valid actions 
    fn select(&mut self, board: &Board, _deadline: Instant, print: bool) -> Output {
        let actions = board.actions();
        let mut best_action = None;
        let mut tab_actions : Vec<Action> = Vec::new(); 
        for a in actions {
            tab_actions.push(a.clone());
        }
        if !tab_actions.is_empty() {
            best_action = tab_actions.choose(&mut rand::rng()); 
        }
        let output= Output::create(best_action.cloned(),[[0.0;2];3],None);
        return output;
    }

    fn clear(&mut self) {
        // no history to clean
    }
}