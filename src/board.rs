use std::{fmt::Display, ops::Add};

use arrayvec::ArrayVec;
use itertools::Itertools;
use rand::seq::IndexedRandom;
use Cell::*;
use Color::*;
use Dir::*;

const PIECES: [char; 5] = ['W', 'w', '.', 'b', 'B'];

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn pawn(&self) -> Cell {
        match self {
            White => WhitePawn,
            Black => BlackPawn,
        }
    }
    pub fn queen(&self) -> Cell {
        match self {
            White => WhiteQueen,
            Black => BlackQueen,
        }
    }
}

impl Color {
    pub fn invert(&mut self) {
        *self = match *self {
            White => Black,
            Black => White,
        };
    }
}

#[repr(i8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum Cell {
    WhiteQueen = 0,
    WhitePawn = 1,
    Empty = 2,
    BlackPawn = 3,
    BlackQueen = 4,
}

impl Cell {
    pub fn adversary(self, curr_player: Color) -> bool {
        match curr_player {
            Color::White => (self as i8) >= 3,
            Color::Black => (self as i8) <= 1,
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", PIECES[*self as usize])
    }
}

/// Size of the board.
pub const N: usize = 8;

/// Maximum number of ply (moves) before declaring the game a draw.
/// This is not a value from the rules: in human checkers, draw is by agreement among the players
/// Should probably be increased for large board (12 and beyond)
pub const MAX_PLY: u16 = 100;

// ===== Constants used for representing the state and computing actions ====
// These are directly derived from N and do not need manual adjustment
/// How many cells of the board array are occipied by the first.last padding length
const PADDING_LINE_SIZE: usize = N / 2 + 1;
// Once every two lines there is an internal padding cell
const NUM_PADDING_INTERNAL_CELLS: usize = N / 2;
const CELLS_PER_ROW: usize = N / 2;
const NUM_VALID_CELLS: usize = N * N / 2;
const RECURRENCE_PADDING_CELL: usize = PADDING_LINE_SIZE + CELLS_PER_ROW;
const NUM_CELLS: usize = NUM_VALID_CELLS + 2 * PADDING_LINE_SIZE + NUM_PADDING_INTERNAL_CELLS;
const NUM_EMPTY_CELLS: usize = CELLS_PER_ROW * 2;
const NUM_PAWNS: usize = CELLS_PER_ROW * (N / 2 - 1);
const QUEEN_MAX_MOVE_LENGTH: usize = N - 1;

/// Board representation:
///  - the piece in each cell of the board
///  - which player is the next to play
///  - how many plies have been played since the begining.
///
/// Uses a padded array representation (https://3dkingdoms.com/checkers/bitboards.htm)
///
/// For an 8x8 board, we would have an array with 46 cells where the value of the cells
/// can be found at the indices below:
/// ```
///    37  38  39  40
///  32  33  34  35
///    28  29  30  31
///  23  24  25  26
///    19  20  21  22
///  14  15  16  17
///    10  11  12  13
///  05  06  07  08
/// ```
/// All values in the array are padding (invalid positions) which are useful to make sure that,
/// e.g., adding `4` to an index moves you to the above left cell (which would be a padding cell if and only if the move is invalid).
///
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub struct Board {
    // Representation of the cells as a padded array.
    // A padding cell is represented by the `None` value.
    cells: [Option<Cell>; NUM_CELLS],
    /// Indicates the color of the next player to move
    pub turn: Color,
    /// How many ply (actions) have been played since the beginning
    /// Use to check
    pub num_ply: u16,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  ")?;
        for i in 0..N {
            write!(f, " {} ", char::from('A' as u8 + i as u8))?;
        }
        write!(f, "   {:?}  ({} plies)", self.turn, self.num_ply)?;
        write!(f, "\n1 ███")?;
        for ((l, c), p) in Position::all().map(|p| (p.coords(), p)).sorted() {
            write!(f, " {} ", self.at(p))?;
            if (c as usize) < N - 1 {
                write!(f, "███")?;
            }
            let next_line = l + 2;
            if c as usize == N - 1 {
                write!(f, "\n{next_line:<2}")?;
            } else if c as usize == N - 2 && (l as usize) < (N - 1) {
                write!(f, "\n{next_line:<2}███")?;
            }
        }
        Ok(())
    }
}

impl Board {
    /// Creates the initial board
    pub fn init() -> Board {
        let mut cells = [None; NUM_CELLS];
        let mut placed_white = 0;
        let mut placed_empty = 0;
        let mut placed_black = 0;
        for i in 0..(NUM_CELLS - PADDING_LINE_SIZE) {
            if i < PADDING_LINE_SIZE || i % RECURRENCE_PADDING_CELL == 0 {
                cells[i] = None
            } else if placed_white < NUM_PAWNS {
                cells[i] = Some(Cell::WhitePawn);
                placed_white += 1;
            } else if placed_empty < NUM_EMPTY_CELLS {
                cells[i] = Some(Cell::Empty);
                placed_empty += 1;
            } else {
                cells[i] = Some(Cell::BlackPawn);
                placed_black += 1;
            }
        }
        assert_eq!(placed_white, NUM_PAWNS);
        assert_eq!(placed_black, NUM_PAWNS);
        assert_eq!(placed_empty, NUM_EMPTY_CELLS);
        Board {
            cells,
            turn: Color::White,
            num_ply: 0,
        }
    }

    fn try_parse_heading(heading: &str) -> Option<(Color, u16)> {
        let color = if heading.contains("Black") {
            Color::Black
        } else {
            Color::White
        };
        let start = heading.find('(')? + 1;
        let rest = &heading[start..];
        let end = rest.find(' ')?;
        let plies = rest[..end].parse().ok()?;
        Some((color, plies))
    }

    pub fn parse(board_str: &str) -> Board {
        let mut empty = Board {
            cells: [None; NUM_CELLS],
            turn: Color::White,
            num_ply: 0,
        };
        let mut ps = Position::all();
        let mut lines = board_str.lines().peekable();

        // Only parse and consume the first line if it is a correct heading
        if let Some((color, plies)) = lines.peek().and_then(|line| Self::try_parse_heading(line)) {
            empty.turn = color;
            empty.num_ply = plies;
            // Header line is valid, consume it
            let _ = lines.next();
        };

        let mut parsed_lines = 0;
        for line in lines.rev() {
            for c in line.chars() {
                match c {
                    'w' => empty.set(ps.next().unwrap(), Cell::WhitePawn),
                    'W' => empty.set(ps.next().unwrap(), Cell::WhiteQueen),
                    'b' => empty.set(ps.next().unwrap(), Cell::BlackPawn),
                    'B' => empty.set(ps.next().unwrap(), Cell::BlackQueen),
                    '.' => empty.set(ps.next().unwrap(), Cell::Empty),
                    _ => {}
                }
            }
            parsed_lines += 1;
            if parsed_lines == N {
                break; // ignore heading
            }
        }
        empty
    }

    /// Applies a number of randomly select moves and return the resulting board.
    ///
    /// This function is typically used to generate original starting point for the games.
    pub fn after_random_moves(n: usize) -> Board {
        let mut cur = Self::init();
        for _ in 0..n {
            let actions = &mut cur.actions();
            let action = actions.choose(&mut rand::rng()).unwrap();
            cur.apply_mut(action);
        }
        cur
    }

    /// Count the number of cells with the given value
    pub fn count(&self, cell: Cell) -> u8 {
        self.cells.iter().filter(|&c| *c == Some(cell)).count() as u8
    }

    /// Provides an iterator over all positions with the given cell value
    pub fn positions_with(&self, cell: Cell) -> impl Iterator<Item = Position> + '_ {
        (PADDING_LINE_SIZE..(NUM_CELLS - PADDING_LINE_SIZE))
            .into_iter()
            .filter(move |i| self.cells[*i] == Some(cell))
            .map(|i| Position::new(i as i8))
    }

    /// Return the value at the given position
    pub fn at(&self, pos: Position) -> Cell {
        self.cells[pos.0 as usize].unwrap()
    }

    /// Sets the position to the given cell value
    pub fn set(&mut self, pos: Position, cell: Cell) {
        self.cells[pos.0 as usize] = Some(cell)
    }

    /// Makes the indicated cell empty
    pub fn clear(&mut self, pos: Position) {
        self.cells[pos.0 as usize] = Some(Cell::Empty)
    }

    fn at_index(&self, index: Index) -> Option<Cell> {
        self.cells[index.0]
    }

    /// Returns true of the given cell is empty
    pub fn empty(&self, pos: Position) -> bool {
        self.at(pos) == Cell::Empty
    }

    /// Returns true if the given cell is occupied by an adversary of teh current player.
    pub fn adv(&self, pos: Position) -> bool {
        self.at(pos).adversary(self.turn)
    }

    pub fn is_draw(&self) -> bool {
        self.num_ply == MAX_PLY
    }

    /// Computes and returns the list of *valid* actions for the current player.
    pub fn actions(&self) -> Vec<Action> {
        let mut actions = Vec::with_capacity(32);
        self.actions_no_alloc(&mut actions);
        actions
    }

    /// A potentially more efficient version that avoids an allocation by reusing an existing vector
    /// Not publicly exposed because using it is typically a preamature optimization
    /// For instance, in a sensible rollout implementation, the compiler would optimize the allocation.
    fn actions_no_alloc(&self, actions: &mut Vec<Action>) {
        // clear the output buffer from any left overs from a previous usage
        actions.clear();

        if self.num_ply >= MAX_PLY {
            // we have reached the maximum number of turns and are not allowed to move anymore.
            // The game will be a draw if both players still have pieces.
            return;
        }

        // add all jumps (captures) to the actions buffer (for pawns and then queens)
        for pawn_pos in self.positions_with(self.turn.pawn()) {
            self.add_pawn_jumps(pawn_pos, actions);
        }
        for queen_pos in self.positions_with(self.turn.queen()) {
            self.add_queen_jumps(queen_pos, actions);
        }
        if !actions.is_empty() {
            // we have at least one action with capture
            // rule: the player must take the move with the maximal number of captures
            // keep only the actions with the maximal number of captures (longest ones) and return immediately before considering the normal moves (with 0 captures)
            let max_captures = actions.iter().map(|a| a.num_moves()).max().unwrap();
            actions.retain(|a| a.num_moves() == max_captures);
            return;
        }

        // if we reach this point no moves with captures were possible, add the normal moves
        for pawn_pos in self.positions_with(self.turn.pawn()) {
            self.add_pawn_moves(pawn_pos, self.turn, actions);
        }
        for queen_pos in self.positions_with(self.turn.queen()) {
            self.add_queen_moves(queen_pos, actions);
        }
    }

    /// Returns the result of applying the action in the state.
    ///
    /// Note: the action is assumed valid and will always produce a new (valid) board.
    ///       This will always be the case when applying actions
    pub fn apply(&self, action: &Action) -> Board {
        let mut next = self.clone();
        next.apply_mut(action);
        next
    }

    /// Modifies the current board to be the result of the action.
    pub fn apply_mut(&mut self, action: &Action) {
        let mut cur = action.0;
        for mv in &action.1 {
            cur = self.apply_move(cur, *mv);
        }
        // promote to queen if at last raw
        if self.turn == White && cur.0 as usize >= NUM_CELLS - PADDING_LINE_SIZE - CELLS_PER_ROW {
            self.set(cur, Cell::WhiteQueen);
        } else if self.turn == Black && (cur.0 as usize) < PADDING_LINE_SIZE + CELLS_PER_ROW {
            self.set(cur, Cell::BlackQueen);
        }
        // update turn and number of ply
        self.turn.invert();
        self.num_ply += 1;
    }

    /// Appllies a single move (subpart of an action over a single direction)
    fn apply_move(&mut self, pos: Position, mv: Move) -> Position {
        let mut cur = pos;
        let piece = self.at(cur);
        for _ in 0..(mv.repeat) {
            self.clear(cur);
            cur = Position((cur + mv.dir).0 as i8);
        }
        self.set(cur, piece);
        cur
    }

    fn add_pawn_moves(&self, pos: Position, color: Color, out: &mut Vec<Action>) {
        let dirs = match color {
            White => &WHITE_PAWNS_DIRS,
            Black => &BLACK_PAWNS_DIRS,
        };
        for &dir in dirs {
            if self.at_index(pos + dir) == Some(Empty) {
                out.push(Action::with_move(pos, dir.repeat(1)));
            }
        }
    }
    fn add_queen_moves(&self, pos: Position, out: &mut Vec<Action>) {
        for dir in ALL_DIRS {
            let mut cur = Index(pos.0 as usize);
            for rep in 1..N {
                let next = cur + dir;
                if self.at_index(next) != Some(Cell::Empty) {
                    break;
                }
                out.push(Action::with_move(pos, dir.repeat(rep as u8)));
                cur = next;
            }
        }
    }
    fn add_pawn_jumps(&self, initial_pos: Position, out: &mut Vec<Action>) {
        self.add_jumps::<2>(initial_pos, out);
    }
    fn add_queen_jumps(&self, initial_pos: Position, out: &mut Vec<Action>) {
        self.add_jumps::<QUEEN_MAX_MOVE_LENGTH>(initial_pos, out);
    }
    fn add_jumps<const MAX_MOVE_LENGTH: usize>(
        &self,
        initial_pos: Position,
        out: &mut Vec<Action>,
    ) {
        let mut captured = [false; NUM_CELLS];
        let empty_action = Action::empty(initial_pos);
        self.jumps_dfs::<MAX_MOVE_LENGTH>(initial_pos, &empty_action, &mut captured, out);
    }

    /// Performs a depth-first search in the tree of possible jumps
    /// Each edge of the tree is a possible jump (capture) from the parent position.
    /// Search recurses from the landing position.
    ///
    /// When a leaf of the tree is reached, an action corresponding to the sequence of jumps from the initial position is added to actions buffer.
    ///
    /// The implementation is the same for pawns and queens: they only differ by the MAX_MOVE_LENGTH const parameter (resp. 2 and N-1).
    /// The MAX_MOVE_LENGTH is a generic parameter to allow the compiler to generate two distinct methods, optimized for each case.
    /// In particular, pawns are much more frequent and many optimization should be possible for a MAX_MOVE_LENGTH of 2 (notably, removing the two inner loops).
    fn jumps_dfs<const MAX_MOVE_LENGTH: usize>(
        &self,
        last_pos: Position,
        action_prefix: &Action,
        captured: &mut [bool; NUM_CELLS],
        out: &mut Vec<Action>,
    ) {
        let mut at_least_one = false;
        for dir in ALL_DIRS {
            let mut cur = Index(last_pos.0 as usize);
            for i in 1..MAX_MOVE_LENGTH {
                cur = cur + dir;
                match self.at_index(cur) {
                    Some(Cell::Empty) => continue,
                    None => break, // reached board limt
                    Some(c) if !c.adversary(self.turn) => {
                        // reached one of or pawns, stop
                        break;
                    }
                    Some(c) if captured[cur.0] => {
                        debug_assert!(c.adversary(self.turn));
                        // there was an adversary but it was captured by the action prefix
                        // treat as empty
                        continue;
                    }
                    Some(c) => {
                        // found an adversary at `cur`
                        debug_assert!(c.adversary(self.turn));
                        let adversary = cur;
                        // mark adversary as captured for the recursive exploration
                        debug_assert!(!captured[adversary.0]);
                        captured[adversary.0] = true;
                        let mut cur_landing = adversary;
                        // now let look for the places we can lend in
                        for j in i + 1..=MAX_MOVE_LENGTH {
                            cur_landing = cur_landing + dir;
                            let empty = self.at_index(cur_landing) == Some(Empty)
                                || captured[cur_landing.0];
                            if empty {
                                at_least_one = true;
                                // this is a landing position
                                let extended = action_prefix.with_new_move(dir.repeat(j as u8));
                                let landing_pos = Position::new(cur_landing.0 as i8);
                                self.jumps_dfs::<MAX_MOVE_LENGTH>(
                                    landing_pos,
                                    &extended,
                                    captured,
                                    out,
                                );
                            } else {
                                // there is something blocking us, abort
                                break;
                            }
                        }
                        // unmark the adversary as captued
                        captured[adversary.0] = false;
                        break;
                    }
                }
            }
        }
        if !at_least_one && action_prefix.num_moves() > 0 {
            out.push(action_prefix.clone());
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index(usize);

impl Add<Dir> for Index {
    type Output = Index;

    fn add(self, rhs: Dir) -> Self::Output {
        Index((self.0 as isize + rhs as isize) as usize)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub struct Position(i8);

impl Position {
    #[inline(never)]
    pub fn all() -> impl Iterator<Item = Position> {
        (PADDING_LINE_SIZE..(NUM_CELLS - PADDING_LINE_SIZE))
            .filter(|i| *i % RECURRENCE_PADDING_CELL != 0)
            .map(|i| Position::new(i as i8))
    }
    pub fn new(pos: i8) -> Position {
        debug_assert!(pos >= 0 && pos < NUM_CELLS as i8);
        Position(pos)
    }

    pub fn line(self) -> u8 {
        let i = self.0 as usize;
        let base = if i % RECURRENCE_PADDING_CELL < PADDING_LINE_SIZE {
            0
        } else {
            1
        };
        //dbg!(self, base, i);
        let line_from_bottom = base + (i / RECURRENCE_PADDING_CELL) * 2 - 1;
        let line_from_top = N - line_from_bottom - 1;
        line_from_top as u8
    }
    pub fn column(self) -> u8 {
        let i = self.0 as usize % RECURRENCE_PADDING_CELL;
        let col = if i < PADDING_LINE_SIZE {
            (i - 1) * 2 + 1
        } else {
            (i - PADDING_LINE_SIZE) * 2
        };
        col as u8
    }

    pub fn coords(self) -> (u8, u8) {
        (self.line(), self.column())
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (l, c) = self.coords();
        let col = char::from('A' as u8 + c);
        write!(f, "{col}{}", l + 1)
    }
}

impl Add<Dir> for Position {
    type Output = Index;

    fn add(self, rhs: Dir) -> Self::Output {
        Index((self.0 + rhs as i8) as usize)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
#[repr(i8)]
pub enum Dir {
    UpLeft = CELLS_PER_ROW as i8,
    UpRight = CELLS_PER_ROW as i8 + 1,
    DownLeft = -(CELLS_PER_ROW as i8) - 1,
    DownRight = -(CELLS_PER_ROW as i8),
}
impl Dir {
    pub fn repeat(self, n: u8) -> Move {
        Move {
            dir: self,
            repeat: n,
        }
    }
}
const ALL_DIRS: [Dir; 4] = [UpLeft, UpRight, DownLeft, DownRight];
const WHITE_PAWNS_DIRS: [Dir; 2] = [UpLeft, UpRight];
const BLACK_PAWNS_DIRS: [Dir; 2] = [DownLeft, DownRight];

static_assertions::assert_eq_size!(u16, Move);
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub struct Move {
    pub dir: Dir,
    pub repeat: u8,
}
impl Add<Move> for Position {
    type Output = Position;

    fn add(self, rhs: Move) -> Self::Output {
        let i = self.0 + (rhs.dir as i8) * (rhs.repeat as i8);
        Position::new(i)
    }
}

/// An action is a list of moves
/// There might as many moves as their are pieces on the board, in case of jumps.
/// The representation is a bit wasteful consider that the list will very rarely contain more than a handful elements.
/// The size would sizeof(Move) * NUM_PAWNS bytes = 24 bytes
/// The arrayvec consumes another 2 bytes of the number of elements
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Action(Position, ArrayVec<Move, NUM_PAWNS>);
impl Action {
    pub fn empty(pos: Position) -> Action {
        Action(pos, ArrayVec::new())
    }
    pub fn num_moves(&self) -> usize {
        self.1.len()
    }
    pub fn with_move(pos: Position, mv: Move) -> Action {
        let mut a = Action(pos, ArrayVec::new());
        a.enqueue(mv);
        a
    }
    pub fn with_new_move(&self, mv: Move) -> Action {
        let mut a = self.clone();
        a.enqueue(mv);
        a
    }
    pub fn enqueue(&mut self, mv: Move) {
        self.1.push(mv);
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        let mut last = self.0;
        for m in &self.1 {
            let new = last + *m;
            write!(f, " {}", new)?;
            last = new;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use rand::random_range;

    use super::*;

    #[test]
    fn test_parse() {
        // Ideally, tests should be deterministic. However, generating random boards
        // helps validate the parser against a wide range of scenarios without manually
        // writing out 100 board strings.
        for _ in 0..100 {
            let random_board = Board::after_random_moves(random_range(3..6));
            let parsed_board = Board::parse(&format!("{random_board}"));
            assert_eq!(random_board, parsed_board);
        }
    }

    fn validate_actions(board: &str, expected_actions: &[&str]) {
        println!("====================");
        let board = Board::parse(board);
        let actions = board.actions();
        let actions = actions
            .into_iter()
            .map(|a| a.to_string())
            .sorted()
            .collect_vec();
        let expected_actions = expected_actions
            .iter()
            .map(|a| a.to_string())
            .sorted()
            .collect_vec();
        assert_eq!(actions, expected_actions, "On board \n{board}");
    }

    #[test]
    fn test_actions() {
        validate_actions(
            "
                 ABCDEFGH
                1 . . . .
                2W . . .
                3 B . . .
                4. B . .
                5 . . . .
                6. . . .
                7 . . . .
                8. . . .",
            &["A2 B1"],
        );

        validate_actions(
            "
            ABCDEFGH
           1 W . . .
           2. . . .
           3 . B . .
           4B B . .
           5 . B . .
           6B . . .
           7 . . . .
           8. . . .",
            &["B1 E4 A8", "B1 E4 B7", "B1 E4 C6"],
        );
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            White => write!(f, "White"),
            Black => write!(f, "Black"),
        }
    }
}

impl std::str::FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "White" => Ok(White),
            "Black" => Ok(Black),
            _ => Err(()),
        }
    }
}

impl std::str::FromStr for Board {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Board::parse(s))
    }
}

impl std::str::FromStr for Position {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A8" => Ok(Position::new(5)),
            "C8" => Ok(Position::new(6)),
            "E8" => Ok(Position::new(7)),
            "G8" => Ok(Position::new(8)),
            "B7" => Ok(Position::new(10)),
            "D7" => Ok(Position::new(11)),
            "F7" => Ok(Position::new(12)),
            "H7" => Ok(Position::new(13)),
            "A6" => Ok(Position::new(14)),
            "C6" => Ok(Position::new(15)),
            "E6" => Ok(Position::new(16)),
            "G6" => Ok(Position::new(17)),
            "B5" => Ok(Position::new(19)),
            "D5" => Ok(Position::new(20)),
            "F5" => Ok(Position::new(21)),
            "H5" => Ok(Position::new(22)),
            "A4" => Ok(Position::new(23)),
            "C4" => Ok(Position::new(24)),
            "E4" => Ok(Position::new(25)),
            "G4" => Ok(Position::new(26)),
            "B3" => Ok(Position::new(28)),
            "D3" => Ok(Position::new(29)),
            "F3" => Ok(Position::new(30)),
            "H3" => Ok(Position::new(31)),
            "A2" => Ok(Position::new(32)),
            "C2" => Ok(Position::new(33)),
            "E2" => Ok(Position::new(34)),
            "G2" => Ok(Position::new(35)),
            "B1" => Ok(Position::new(37)),
            "D1" => Ok(Position::new(38)),
            "F1" => Ok(Position::new(39)),
            "H1" => Ok(Position::new(40)),
            _ => Err(()),
        }
    }
}

impl std::str::FromStr for Action {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pos = vec![];
        for s in s.split(" ") {
            pos.push(Position::from_str(s)?)
        }
        if pos.len() < 2 || pos.len() - 1 > NUM_PAWNS {
            return Err(());
        }
        let mut positions = pos.into_iter();
        let first = positions.next().unwrap();
        let mut moves = ArrayVec::<Move, NUM_PAWNS>::new();
        let mut current = first;
        for pos in positions {
            // move = pos - current
            let (current_line, current_column) = current.coords();
            let (new_line, new_column) = pos.coords();
            // compute direction based of two position's line and column differences
            let dir = if current_line < new_line && current_column < new_column {
                Dir::DownRight
            } else if current_line < new_line && current_column > new_column {
                Dir::DownLeft
            } else if current_line > new_line && current_column < new_column {
                Dir::UpRight
            } else if current_line > new_line && current_column > new_column {
                Dir::UpLeft
            } else {
                return Err(());
            };
            // move length can be computed just like this:
            let repeat = current_line.abs_diff(new_line);
            let m = dir.repeat(repeat);

            moves.push(m);
            current = pos;
        }
        Ok(Action(first, moves))
    }
}

#[test]
fn test_parse_pos() {
    for pos in Position::all() {
        assert_eq!(pos, pos.to_string().parse().expect("Parse error"));
    }
}

#[test]
fn test_parse_action() {
    let board = Board::parse(
        "
             ABCDEFGH
            1 . . . .
            2W . . .
            3 B . . .
            4. B . .
            5 . . . .
            6. . . .
            7 . . . .
            8. . . .",
    );
    for action in board.actions() {
        assert_eq!(action, action.to_string().parse().expect("Oops ?"));
    }

    let board = Board::parse(
        "
        ABCDEFGH
       1 W . . .
       2. . . .
       3 . B . .
       4B B . .
       5 . B . .
       6B . . .
       7 . . . .
       8. . . .",
    );
    for action in board.actions() {
        assert_eq!(action, action.to_string().parse().expect("Oops ?"));
    }
}
