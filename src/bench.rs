mod args;
mod board;
mod engine;
mod mcts;
mod minimax;

use anyhow::bail;
use board::Board;

use std::time::Duration;

use ai_tournament::prelude::*;

use crate::board::Color;

fn main() {
    // setup agent constraints
    let constraints = ConstraintsBuilder::new()
        .with_ram_per_agent(1000)
        .with_action_timeout(Duration::from_millis(100))
        .with_time_margin(Duration::from_millis(10)) // this margin should no be relied upon
        .build()
        .expect("builder");

    // setup evaluator's configuration
    let config = Configuration::new()
        .with_verbose(true)
        .with_allow_uncontained(true) // allow the evaluator to not only run on linux, but make sure to not use too much RAM because it will be enforced during evaluation!
        .with_compile_agents(true)
        .with_self_test(true) // edge case when agent is in current directory
        .with_test_all_configs(true) // unlike final evaluation, testing all configs in config.yaml
        .with_log("./logs");

    // setup tournament
    let tournament = SwissTournament::with_auto_rounds(8);
    let factory = Checkers::init();
    let evaluator = Evaluator::new(factory, config, constraints);
    let (results, errors) = evaluator.evaluate(".", tournament).expect("evaluation");

    let mut results = results.into_iter().collect::<Vec<_>>();
    results.sort_by(|a, b| b.1.cmp(&a.1)); // descending order
    results
        .iter()
        .for_each(|(name, value)| println!("{name}: {value}"));

    println!("\nNon-compiling agents:");
    for (name, error) in errors {
        println!(" - {name}: {error}");
    }
}

/// Structure used as an interface to ai_tournament by implementing Game and GameFactory traits
#[derive(Clone, Debug)]
pub struct Checkers {
    board: Board,
    winner: i8,
}

impl Checkers {
    pub fn init() -> Checkers {
        Checkers {
            board: Board::init(),
            winner: -1,
        }
    }

    fn set_other_to_winner(&mut self) {
        self.winner = if self.board.turn == Color::White {
            1
        } else {
            0
        };
    }
}

impl ai_tournament::game_interface::Game for Checkers {
    type State = Board;

    type Action = board::Action;

    type Score = f32;

    fn apply_action(&mut self, action: &Option<Self::Action>) -> ai_tournament::anyhow::Result<()> {
        let Some(action) = action else {
            self.set_other_to_winner();
            bail!("action is None");
        };
        if !self.board.actions().contains(action) {
            self.set_other_to_winner();
            bail!("invalid action: {action}");
        }
        self.board.apply_mut(action);
        if self.board.actions().is_empty() && self.winner == -1 {
            // current player changed with `apply_mut` so the other (the one who just played) won
            self.set_other_to_winner();
        }
        Ok(())
    }

    fn get_state(&self) -> Self::State {
        self.board.clone()
    }

    fn get_current_player_number(&self) -> usize {
        if self.board.turn == Color::White {
            0
        } else {
            1
        }
    }

    fn is_finished(&self) -> bool {
        self.winner != -1 || self.board.is_draw()
    }

    fn get_player_score(&self, player_number: u32) -> f32 {
        if self.board.is_draw() {
            0.5
        } else if player_number == self.winner as u32 {
            1.0
        } else {
            0.0
        }
    }
}

/// used by ai_tournament::Evaluator to create new game
impl ai_tournament::game_interface::GameFactory<Checkers> for Checkers {
    fn new_game(&self) -> Checkers {
        Checkers::init()
    }
}
