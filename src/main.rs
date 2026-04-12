#![allow(unused)]
use std::time::{Duration, Instant};

use board::*;
use engine::Engine;
use mcts::MctsEngine;
use minimax::MinimaxEngine;
use rand::seq::IndexedRandom;
use mcts::*;

pub mod board;
pub mod engine;
pub mod mcts;
pub mod minimax;

/// Applies a number of randomly select moves and return the resulting board.
///
/// This function is typically used to generate original starting point for the games.
fn after_random_moves(board: &Board, n: usize) -> Board {
    let mut cur = board.clone();
    for _ in 0..n {
        let actions = &mut cur.actions();
        let action = actions.choose(&mut rand::rng()).unwrap();
        cur.apply_mut(action);
    }
    cur
}

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
    let mut nb_moves = 0; 
    let print_every_n_moves = 20; 
    let mut print = false; 

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
        if let Some(action) = engine.select(&board, deadline, print) {
            if verbose {
                println!("\n action: {action}\n");
            }
            board.apply_mut(&action);

            // print information every n moves 
            nb_moves += 1; 
            print = (nb_moves % print_every_n_moves) == 0; 

        } else {
            // no possible actions, game is over
            return board;
        }
    }
    board
}

fn main() {
    let b = Board::init();

    let mut white_engine = MinimaxEngine::new(6);
    let mut black_engine = MctsEngine::new(0.2);
    let time_per_move : Duration = Duration::new(0, 100);

    play_game(&b, &mut white_engine, &mut black_engine, time_per_move, true);
}

#[allow(unused)]
fn example_game() {
    // generate the initial board
    let board = Board::init();

    // play a few random move to make sure we have an fairly original starting point
    let board = after_random_moves(&board, 2);

    let mut white_engine = MinimaxEngine::new(6);
    // let mut white_engine = MctsEngine::new(1.);
    let mut black_engine = MinimaxEngine::new(6);

    let final_board = play_game(
        &board,
        &mut white_engine,
        &mut black_engine,
        Duration::from_millis(5),
        true,
    );

    println!("Final board: \n{final_board}");

    // Printing white's score
    let white_score = white_score(&final_board);
    println!("White's score: {white_score}");
}

#[allow(unused)]
fn test_rollout() {

    let mut index = 0.0;  
    let mut sum = 0.0;

    let timer = Instant::now();

    while timer.elapsed().as_millis() < 1000 {
        let board = Board::init(); 
        sum += mcts::rollout(&board); 
        index += 1.0; 
    }

    println!("Mean rollout : {}", (sum/index));
    println!("Number of execution: {}", index);
}

