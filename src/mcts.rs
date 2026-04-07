use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use hashbrown::HashMap;
use itertools::Itertools;
use rand::seq::IndexedRandom;

use crate::engine::Engine;

use super::board::*;

/// Function that evaluates a final board (draw, or no remaning actions for the current player).
pub fn white_score(board: &Board) -> f32 {
    debug_assert!(
        board.is_draw() || board.actions().is_empty(),
        "The board is not final"
    );
    // Draw if max number of turns played
    if board.is_draw() {
        return 0.5;
    } else if board.actions().is_empty() {
        if board.turn == Color::White {
        // Loose if it is white who has no available actions left
            return 0.0;
        // Win if it is black who has no available actions left
        } else {
            return 1.0;
        }
    } else {
        // Compiler comprend que on aura jamais ce cas
        unreachable!();
    }
        
}

/// Performs a single rollout and returns the evaluation of the final state.
pub fn rollout(board: &Board) -> f32 {

    let mut current_board : Board = board.clone(); 

    while !(current_board.is_draw() || current_board.actions().is_empty()) {
        // 1. choose a random action 
        let chosen_action : Option<Action> = (current_board.actions()).choose(&mut rand::rng()).cloned();

        // 2. generate the resulting state 
        match chosen_action {
            None => panic!("No more valid actions are available but the board is not final !"),
            Some(action) =>  current_board = current_board.apply(&action),
        }
    }

    white_score(&current_board)
}

/// Alias type to repesent a count of selections.
pub type Count = u64;

/// Node of the MCTS graph
struct Node {
    /// s: Board of the node
    board: Board,
    /// N(s): Number of times this node has been selected
    count: Count,
    /// *All* valid actions available on the board, together with the number of times they have been selected (potentially 0)
    /// and the last known evaluation of the result board.
    /// The actions define the outgoing edges (the target nodes can be computed by applying the action on the board)
    out_edges: Vec<OutEdge>,
    /// U(s): Evaluation given by the initial rollout on expansion
    initial_eval: f32,
    /// Q(s): complete evaluation of the node (to be updated after each playout)
    eval: f32,
}

impl Node {
    /// Creates the node with a single evaluation from a rollout
    pub fn init(board: Board, initial_eval: f32) -> Node {
        // create one outgoing edge per valid action
        let out_edges = board
            .actions()
            .into_iter()
            .map(|a| OutEdge::new(a))
            .collect_vec();
        Node {
            board,
            count: 1,
            out_edges,
            initial_eval,
            eval: initial_eval,
        }
    }
}

/// Edge of the MCTS graph.
///
/// An `OutEdge` is attached to a node (source) and target can be computed by applying the action to the source.
struct OutEdge {
    // a: action of the edge
    action: Action,
    // N(s,a): number of times this edge was selected
    visits: Count,
    // Q(s,a): Last known evaluation of the board resulting from the action
    eval: f32,
}
impl OutEdge {
    /// Initializes a new edge for this actions (with a count and eval at 0)
    pub fn new(action: Action) -> OutEdge {
        OutEdge {
            action,
            visits: 0,
            eval: 0.,
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "\n{}", self.board)?;
        write!(f, "Q: {}    (N: {})\n", self.eval, self.count)?;
        // display edges by decreasing number of samples
        for OutEdge {
            action,
            visits,
            eval,
        } in self.out_edges.iter().sorted_by_key(|e| u64::MAX - e.visits)
        {
            write!(f, "{visits:>8} {action}   [{eval}]\n")?;
        }
        Ok(())
    }
}

pub struct MctsEngine {
    /// Graph structure
    nodes: HashMap<Board, Node>,
    /// weight given to the exploration term in UCB1
    pub exploration_weight: f32,
}
impl MctsEngine {
    pub fn new(exploration_weight: f32) -> MctsEngine {
        MctsEngine {
            nodes: HashMap::new(),
            exploration_weight,
        }
    }
}

impl MctsEngine {
    /// Selects the best action according to UCB1, or `None` if no action is available.
    pub fn select_ucb1(&self, board: &Board) -> Option<Action> {
        debug_assert!(self.nodes.contains_key(board));
        
        let mut best_action : Option<Action> = None;
        let mut max_ucb1 : f32 = 0.0; 
        let node : &Node = &self.nodes[board];

        let turn: f32;
        if board.turn == Color::White {
            turn = 1.0; 
        } else {
            turn = -1.0; 
        }

        for out_edge in &self.nodes[board].out_edges {
            let ucb1 = turn * out_edge.eval + self.exploration_weight * ((2 * (node.count.ilog(10)) / (out_edge.visits as u32)).isqrt() as f32);

            if ucb1 >= max_ucb1 {
                max_ucb1 = ucb1; 
                best_action = Some(out_edge.action.clone())
           }
        }

        best_action

    }

    /// Performs a playout for this board (s) and returns the (updated) evaluation of the board (Q(s))
    fn playout(&mut self, board: &Board) -> f32 {

        let current_board : Board = board.clone();

        // If board not already "rollouted"
        if !self.nodes.contains_key(&current_board) {                                    
            let initial_eval = rollout(&current_board);                                     // Rollout
            let new_node : Node = Node::init(current_board.clone(),initial_eval);           // Create a new node with inital evaluation
            self.nodes.insert(current_board,new_node);
            return initial_eval;                                                            // Add it to the graph (= expand)
        } else {
            let best_action : Option<Action> = self.select_ucb1(&current_board);
            let mut new_board : Board;
            let mut action_eval : f32;
            let updated_eval : f32;
            match best_action {
                // If board is not final
                Some(x) => {new_board = current_board.apply(&x);
                        action_eval = self.playout(&new_board);                             // Recursive playout
                        updated_eval = self.update_eval(&current_board,&x,action_eval);     // Update evaluation
                        return updated_eval},
                // If board is final
                None => return self.nodes[board].eval,
            };
        }
    }

    /// Updates the evaluation (Q(s)) of the board (s), after selected the action (a) for a new playout
    /// which yieled an evaluation of `action_eval` (Q(s,a))
    fn update_eval(&mut self, board: &Board, action: &Action, action_eval: f32) -> f32 {
        debug_assert!(self.nodes.contains_key(board));

        let mut updated_node : &mut Node = &mut self.nodes.get_mut(board).unwrap();

        // Update the number of times this node was selected
        updated_node.count += 1;
    
        let mut selected_edge_eval : f32 = 0.0;
        let mut sum : Count = 0;

        // Finding which edge to update
        for mut out_edge in updated_node.out_edges .iter_mut() {
            if *action == out_edge.action {
                selected_edge_eval = out_edge.eval ;
                // Update number of times this action was selected for this board
                out_edge.visits += 1;
                // Update evaluation of taking action a
                out_edge.eval = action_eval;
            }
            // Computing the sum term for the updated evaluation of the node
            sum = sum + out_edge.visits/updated_node.count;  
        }

        // Updates evaluation for node
        updated_node.eval = updated_node.initial_eval/(updated_node.count as f32) + (sum as f32) * selected_edge_eval;

        return updated_node.eval;
    }
        
}

impl Engine for MctsEngine {
    fn select(&mut self, board: &Board, deadline: Instant) -> Option<Action> {

        let mut time_remaining: bool = Instant::now() < deadline;
        let mut best_action : Option<Action> = None;

        while time_remaining {
            self.playout(board);

            let max_visits = 0; 
            let mut actions = self.nodes.get_mut(board).unwrap();

            for out_edge in actions.out_edges .iter_mut() {
                if out_edge.visits > max_visits {
                    best_action = Some(out_edge.action.clone())
                }
            }
        }
        best_action
    }

    fn clear(&mut self) {
        self.nodes.clear();
    }
}

#[cfg(test)]
mod test {
    use crate::Color;

    use super::{Board, MctsEngine};

    #[test]
    fn test_mcts() {
        let board = Board::parse(
            "
              ABCDEFGH   White  (32 plies)
            1  b . b b
            2 . . . b
            3  . . . w
            4 . . . .
            5  . . . .
            6 . b w .
            7  . . w .
            8 w w w .",
            Color::White,
        );
        let mut mcts = MctsEngine::new(1.);

        println!("{board}");

        for i in 1..=4 {
            mcts.playout(&board);
            println!("After {i} playouts: \n{}", mcts.nodes[&board]);
        }
        println!("{board}");
    }
}
