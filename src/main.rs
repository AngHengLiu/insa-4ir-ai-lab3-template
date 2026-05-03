#![allow(unused)]
use std::time::{Duration, Instant};

use board::*;
use engine::Engine;
use mcts::MctsEngine;
use minimax::MinimaxEngine;
use rand::seq::IndexedRandom;
use mcts::*;

use crate::random::RandomEngine;

pub mod board;
pub mod engine;
pub mod mcts;
pub mod minimax;
pub mod random;

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
) -> Output {
    // erase engines history
    white.clear();
    black.clear();

    let mut board = board.clone();
    let mut nb_moves = 0; 
    let print_every_n_moves = 5; 
    let mut print = true;
    let mut output: Output = Output::create(None,[[0.0;3];3],None); 
    let mut selected_output = Output::create(None,[[0.0;3];3],None); 
    let mut game_metrics = [[0.0;3];3];
    let mut metric_index = 0;

    while !(board.is_draw() || board.actions().is_empty()) {
        if verbose {
            println!("{board}");
        }
        // select the engine
        let engine = match board.turn {
            board::Color::White => &mut *white,
            board::Color::Black => &mut *black,
        };

        nb_moves += 1; 
        print = (nb_moves % print_every_n_moves) == 0; 
        let deadline = Instant::now() + time_per_move;
        selected_output = engine.select(&board,deadline,print);


        // ------------------------------ CHANGE COLOR FOR EVALUATE THE MCTS ENGINE ---------------------------------------------------//

        if print && board.turn == board::Color::White && metric_index < 3 { 
            game_metrics[metric_index][0] = selected_output.metrics[0][0];
            game_metrics[metric_index][1] = selected_output.metrics[0][1];
            game_metrics[metric_index][2] = selected_output.metrics[0][2];
            metric_index += 1;
        }
        if let Some(action) = selected_output.action {
            if verbose {
                println!("\n action: {action}\n");
            }
            board.apply_mut(&action);
        } else {
            // no possible actions, game is over
            output = Output::create(None, game_metrics,Some(board));
            return output;
        }
    }
    output = Output::create(None,game_metrics,Some(board));
    return output;
}

fn main() {
    let num_games = 100;
    let mut current_game_num: i32 = 0;
    let mut tot_rate = 0.;
    let mut tot_depth = 0.;

    let mut w_score: f32 = 0.;
    let mut avr_score: f32 = 0.; 
    let mut tot_wins_white = 0.;

    let mut tot_depth_start: f64 = 0.;
    let mut tot_rate_start: f64 = 0.;
    let mut tot_principal_var_start: f64 = 0.;

    let mut tot_depth_middle: f64 = 0.;
    let mut tot_rate_middle: f64 = 0.;
    let mut tot_principal_var_middle: f64 = 0.;

    let mut tot_depth_end: f64 = 0.;
    let mut tot_rate_end: f64 = 0.;
    let mut tot_principal_var_end: f64 = 0.;

    let mut final_board: Output;

    let b = Board::init();

    // ----------------------------------------- ENGINE CONFIGURATION -----------------------------------------------------------//

    let mut white_engine = MctsEngine::new(0.2, 0, 6); //MinimaxEngine::new(6);
    let mut black_engine = MinimaxEngine::new(6); //MinimaxEngine::new(6);
    let time_per_move : Duration = Duration::new(0, 10000000);

    // --------------------------------------------------------------------------------------------------------------------------//

    let mut i = 0;
    while i != num_games {
        final_board = play_game(&b, &mut white_engine, &mut black_engine, time_per_move, false);
        w_score = white_score(&final_board.board.unwrap());
        println!("White's score: {w_score}");
        if w_score == 1.0 {
            tot_wins_white += w_score;
        }
        avr_score += w_score; 
        current_game_num += 1;

        // Metrics
        println!("White has so far won {tot_wins_white} out of {current_game_num} games");
        tot_rate_start += final_board.metrics[0][0];
        tot_depth_start += final_board.metrics[0][1];
        tot_principal_var_start += final_board.metrics[0][2];

        tot_rate_middle += final_board.metrics[1][0];
        tot_depth_middle += final_board.metrics[1][1];
        tot_principal_var_middle += final_board.metrics[1][2];

        tot_rate_end += final_board.metrics[2][0];
        tot_depth_end += final_board.metrics[2][1];
        tot_principal_var_end += final_board.metrics[2][2];

        i += 1;
    }

    let percentage_white_wins = (tot_wins_white / num_games as f32)*100.0;
    let avrg_rate_start = (tot_rate_start / num_games as f64);
    let avrg_depth_start = (tot_depth_start / num_games as f64);
    let length_pv_start = (tot_principal_var_start / num_games as f64);
    let avrg_rate_middle = (tot_rate_middle / num_games as f64);
    let avrg_depth_middle = (tot_depth_middle / num_games as f64);
    let length_pv_middle = (tot_principal_var_middle / num_games as f64);
    let avrg_rate_end = (tot_rate_end / num_games as f64);
    let avrg_depth_end = (tot_depth_end / num_games as f64);
    let length_pv_end = (tot_principal_var_end / num_games as f64);
    avr_score /= (num_games as f32); 

    println!("White has won {percentage_white_wins}% of the games");
    println!("Average score of White: {avr_score}"); 

    println!("Average playouts per second in the beginning: {avrg_rate_start}");
    println!("Average of average playout depth in the beginning: {avrg_depth_start}");
    println!("Average lenght of the principal variation in the beginning: {length_pv_start}\n");

    println!("Average playouts per second in the middle: {avrg_rate_middle}");
    println!("Average of average playout depth in the middle: {avrg_depth_middle}");
    println!("Average lenght of the principal variation in the middle: {length_pv_middle}\n");

    println!("Average playouts per second in the end: {avrg_rate_end}");
    println!("Average of average playout depth in the end: {avrg_depth_end}");
    println!("Average lenght of the principal variation in the end: {length_pv_end}\n");
}

#[allow(unused)]
fn example_game() {
    // generate the initial board
    let board = Board::init();

    // play a few random move to make sure we have an fairly original starting point
    let board = after_random_moves(&board, 2);

    let mut white_engine = MctsEngine::new(0.2, 1, 6);
    // let mut white_engine = MctsEngine::new(1.);
    let mut black_engine = MinimaxEngine::new(6);

    let final_output = play_game(
        &board,
        &mut white_engine,
        &mut black_engine,
        Duration::from_millis(5),
        true,
    );

    let final_board : Board = final_output.board.unwrap();
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

