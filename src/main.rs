#![allow(unused)]
use std::time::{Duration, Instant};

use board::*;
use clap::Parser;
use engine::Engine;
use mcts::MctsEngine;
use minimax::MinimaxEngine;
use rand::seq::IndexedRandom;

use crate::args::EngineArgs;

mod args;
mod board;
mod engine;
mod mcts;
mod minimax;

/// Plays a single game, starting form the given board.
/// Time is limited to to the given `time_per_move` at each turn
/// (but we do not enforce it and minimax would typically ignore it).
fn play_game<'a>(
    board: &Board,
    white: &'a mut dyn Engine,
    black: &'a mut dyn Engine,
    time_per_move: Duration,
    verbose: bool,
) -> Board {
    // erase engines history
    white.clear();
    black.clear();

    let mut board = board.clone();

    while !board.is_draw() {
        if verbose {
            println!("{board}");
        }
        // select the engine
        let engine = match board.turn {
            board::Color::White => &mut *white,
            board::Color::Black => &mut *black,
        };
        let deadline = Instant::now() + time_per_move;
        if let Some(action) = engine.select(&board, deadline) {
            if verbose {
                println!("\n action: {action}\n");
            }
            board.apply_mut(&action);
        } else {
            // no possible actions, game is over
            return board;
        }
    }
    board
}

fn main() {
    let args = args::EngineArgs::parse();
    let b = Board::init();

    example_game(&args);
}

#[allow(unused)]
fn example_game(args: &EngineArgs) {
    // generate the initial board and play a few
    // random move to make sure we have an fairly original starting point
    let board = Board::after_random_moves(2);

    let mut white_engine = engine::create_engine(args);
    let mut black_engine = MinimaxEngine::new(6);

    let final_board = play_game(
        &board,
        &mut *white_engine,
        &mut black_engine,
        Duration::from_millis(500),
        true,
    );

    println!("Final board: \n{final_board}");
}
