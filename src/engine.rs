use std::time::Instant;

use crate::{
    args::EngineArgs,
    board::{Action, Board},
    mcts::MctsEngine,
    minimax::MinimaxEngine,
};

pub trait Engine {
    /// Selects the next action with a deadline
    /// If the engine returns `None`, it will be interpret as having lost.
    fn select(&mut self, board: &Board, deadline: Instant) -> Option<Action>;

    /// Forget all history (to be called to reset the engine in between games)
    fn clear(&mut self);
}

/// Create the engine corresponding to the EngineArgs provided
///
/// Returning a Box<dyn ...> is necessary to be able to return any Engine struct
/// as the returned object's size is only known at runtime.
pub fn create_engine(args: &EngineArgs) -> Box<dyn Engine> {
    match args.mode {
        crate::args::Mode::Minimax { depth } => Box::new(MinimaxEngine::new(depth)),
        crate::args::Mode::Mcts { exploration_weight } => {
            Box::new(MctsEngine::new(exploration_weight))
        }
    }
}
