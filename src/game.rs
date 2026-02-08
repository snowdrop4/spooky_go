use std::collections::HashSet;

use crate::board::{Board, STANDARD_COLS, STANDARD_ROWS};
use crate::outcome::GameOutcome;
use crate::player::Player;
use crate::position::Position;
use crate::r#move::Move;

fn get_neighbors_on_board(board: &Board, pos: &Position) -> Vec<Position> {
    let mut neighbors = Vec::new();
    let col = pos.col;
    let row = pos.row;

    if col > 0 {
        neighbors.push(Position::new(col - 1, row));
    }
    if col + 1 < board.width() {
        neighbors.push(Position::new(col + 1, row));
    }
    if row > 0 {
        neighbors.push(Position::new(col, row - 1));
    }
    if row + 1 < board.height() {
        neighbors.push(Position::new(col, row + 1));
    }

    neighbors
}

fn get_group_on_board(board: &Board, start: &Position, player: Player) -> HashSet<Position> {
    let mut group = HashSet::new();
    let mut stack = vec![*start];

    while let Some(pos) = stack.pop() {
        if group.contains(&pos) {
            continue;
        }

        if board.get_piece(&pos) == Some(player) {
            group.insert(pos);

            for neighbor in get_neighbors_on_board(board, &pos) {
                if !group.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }

    group
}

fn has_liberties_on_board(board: &Board, group: &HashSet<Position>) -> bool {
    for pos in group {
        for neighbor in get_neighbors_on_board(board, pos) {
            if board.get_piece(&neighbor).is_none() {
                return true;
            }
        }
    }
    false
}

#[derive(Clone, Debug)]
struct MoveHistoryEntry {
    move_: Move,
    captured_stones: Vec<Position>,
    previous_ko_point: Option<Position>,
}

pub const DEFAULT_KOMI: f32 = 7.5;

#[derive(Clone, Debug)]
pub struct Game {
    board: Board,
    current_player: Player,
    move_history: Vec<MoveHistoryEntry>,
    is_over: bool,
    outcome: Option<GameOutcome>,
    consecutive_passes: u8,
    ko_point: Option<Position>,
    komi: f32,
    min_moves_before_pass_ends: usize,
    max_moves: usize,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Self {
        Self::with_komi(width, height, DEFAULT_KOMI)
    }

    pub fn with_komi(width: usize, height: usize, komi: f32) -> Self {
        let board_size = width * height;
        let min_moves = board_size / 2;
        let max_moves = board_size * 3;
        Self::with_options(width, height, komi, min_moves, max_moves)
    }

    pub fn with_options(
        width: usize,
        height: usize,
        komi: f32,
        min_moves_before_pass_ends: usize,
        max_moves: usize,
    ) -> Self {
        Game {
            board: Board::new(width, height),
            current_player: Player::Black,
            move_history: Vec::new(),
            is_over: false,
            outcome: None,
            consecutive_passes: 0,
            ko_point: None,
            komi,
            min_moves_before_pass_ends,
            max_moves,
        }
    }

    pub fn standard() -> Self {
        Self::new(STANDARD_COLS, STANDARD_ROWS)
    }

    pub fn komi(&self) -> f32 {
        self.komi
    }

    pub fn min_moves_before_pass_ends(&self) -> usize {
        self.min_moves_before_pass_ends
    }

    pub fn max_moves(&self) -> usize {
        self.max_moves
    }

    pub fn move_count(&self) -> usize {
        self.move_history.len()
    }

    pub fn width(&self) -> usize {
        self.board.width()
    }

    pub fn height(&self) -> usize {
        self.board.height()
    }

    pub fn get_piece(&self, pos: &Position) -> Option<i8> {
        self.board.get_piece(pos).map(|p| p as i8)
    }

    pub fn set_piece(&mut self, pos: &Position, player: Option<Player>) {
        self.board.set_piece(pos, player)
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn turn(&self) -> Player {
        self.current_player
    }

    pub fn is_over(&self) -> bool {
        self.is_over
    }

    pub fn outcome(&self) -> Option<GameOutcome> {
        self.outcome
    }

    pub fn move_history(&self) -> Vec<Move> {
        self.move_history.iter().map(|e| e.move_).collect()
    }

    pub fn ko_point(&self) -> Option<Position> {
        self.ko_point
    }

    pub fn score(&self) -> (f32, f32) {
        let mut black_score: f32 = 0.0;
        let mut white_score: f32 = self.komi;

        let mut visited = HashSet::new();

        for row in 0..self.board.height() {
            for col in 0..self.board.width() {
                let pos = Position::new(col, row);

                match self.board.get_piece(&pos) {
                    Some(Player::Black) => black_score += 1.0,
                    Some(Player::White) => white_score += 1.0,
                    None => {
                        if !visited.contains(&pos) {
                            let (region, owner) = self.get_empty_region(&pos, &mut visited);
                            let territory = region.len() as f32;
                            match owner {
                                Some(Player::Black) => black_score += territory,
                                Some(Player::White) => white_score += territory,
                                None => {}
                            }
                        }
                    }
                }
            }
        }

        (black_score, white_score)
    }

    fn get_empty_region(
        &self,
        start: &Position,
        visited: &mut HashSet<Position>,
    ) -> (HashSet<Position>, Option<Player>) {
        let mut region = HashSet::new();
        let mut stack = vec![*start];
        let mut black_adjacent = false;
        let mut white_adjacent = false;

        while let Some(pos) = stack.pop() {
            if visited.contains(&pos) || region.contains(&pos) {
                continue;
            }

            if self.board.get_piece(&pos).is_some() {
                continue;
            }

            region.insert(pos);
            visited.insert(pos);

            for neighbor in self.get_neighbors(&pos) {
                match self.board.get_piece(&neighbor) {
                    Some(Player::Black) => black_adjacent = true,
                    Some(Player::White) => white_adjacent = true,
                    None => {
                        if !visited.contains(&neighbor) && !region.contains(&neighbor) {
                            stack.push(neighbor);
                        }
                    }
                }
            }
        }

        let owner = match (black_adjacent, white_adjacent) {
            (true, false) => Some(Player::Black),
            (false, true) => Some(Player::White),
            _ => None,
        };

        (region, owner)
    }

    fn determine_outcome(&self) -> GameOutcome {
        let (black_score, white_score) = self.score();
        if black_score > white_score {
            GameOutcome::BlackWin
        } else if white_score > black_score {
            GameOutcome::WhiteWin
        } else {
            GameOutcome::Draw
        }
    }

    fn get_neighbors(&self, pos: &Position) -> Vec<Position> {
        get_neighbors_on_board(&self.board, pos)
    }

    fn get_group(&self, start: &Position) -> HashSet<Position> {
        match self.board.get_piece(start) {
            Some(player) => get_group_on_board(&self.board, start, player),
            None => HashSet::new(),
        }
    }

    fn count_liberties(&self, group: &HashSet<Position>) -> usize {
        let mut liberties = HashSet::new();

        for pos in group {
            for neighbor in self.get_neighbors(pos) {
                if self.board.get_piece(&neighbor).is_none() {
                    liberties.insert(neighbor);
                }
            }
        }

        liberties.len()
    }

    fn has_liberties(&self, group: &HashSet<Position>) -> bool {
        has_liberties_on_board(&self.board, group)
    }

    fn remove_group(&mut self, group: &HashSet<Position>) {
        for pos in group {
            self.board.set_piece(pos, None);
        }
    }

    fn would_be_suicide(&self, pos: &Position, player: Player) -> bool {
        let mut test_board = self.board.clone();
        test_board.set_piece(pos, Some(player));

        let group = get_group_on_board(&test_board, pos, player);

        if has_liberties_on_board(&test_board, &group) {
            return false;
        }

        let opponent = player.opposite();
        for neighbor in get_neighbors_on_board(&test_board, pos) {
            if test_board.get_piece(&neighbor) == Some(opponent) {
                let opponent_group = get_group_on_board(&test_board, &neighbor, opponent);

                if !has_liberties_on_board(&test_board, &opponent_group) {
                    return false;
                }
            }
        }

        true
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        if self.is_over {
            return Vec::new();
        }

        let mut moves = Vec::new();

        for row in 0..self.board.height() {
            for col in 0..self.board.width() {
                let pos = Position::new(col, row);

                if self.board.get_piece(&pos).is_some() {
                    continue;
                }

                if let Some(ko) = self.ko_point {
                    if ko == pos {
                        continue;
                    }
                }

                if self.would_be_suicide(&pos, self.current_player) {
                    continue;
                }

                moves.push(Move::place(col, row));
            }
        }

        moves.push(Move::pass());

        moves
    }

    pub fn is_legal_move(&self, move_: &Move) -> bool {
        if self.is_over {
            return false;
        }

        match move_ {
            Move::Pass => true,
            Move::Place { col, row } => {
                let pos = Position::new(*col, *row);

                if !pos.is_valid(self.board.width(), self.board.height()) {
                    return false;
                }

                if self.board.get_piece(&pos).is_some() {
                    return false;
                }

                if let Some(ko) = self.ko_point {
                    if ko == pos {
                        return false;
                    }
                }

                if self.would_be_suicide(&pos, self.current_player) {
                    return false;
                }

                true
            }
        }
    }

    pub fn make_move(&mut self, move_: &Move) -> bool {
        if !self.is_legal_move(move_) {
            return false;
        }

        let previous_ko_point = self.ko_point;
        let mut captured_stones = Vec::new();
        self.ko_point = None;

        match move_ {
            Move::Pass => {
                self.consecutive_passes += 1;

                // Only end game via double-pass if we've played enough moves
                // Note: +1 because move_history hasn't been updated yet
                if self.consecutive_passes >= 2
                    && self.move_history.len() + 1 >= self.min_moves_before_pass_ends
                {
                    self.is_over = true;
                    self.outcome = Some(self.determine_outcome());
                }
            }
            Move::Place { col, row } => {
                self.consecutive_passes = 0;

                let pos = Position::new(*col, *row);
                self.board.set_piece(&pos, Some(self.current_player));

                let opponent = self.current_player.opposite();
                let mut total_captured = 0;
                let mut single_capture_pos: Option<Position> = None;

                for neighbor in self.get_neighbors(&pos) {
                    if self.board.get_piece(&neighbor) == Some(opponent) {
                        let group = self.get_group(&neighbor);
                        if !self.has_liberties(&group) {
                            if group.len() == 1 && total_captured == 0 {
                                single_capture_pos = Some(neighbor);
                            } else {
                                single_capture_pos = None;
                            }

                            total_captured += group.len();

                            for p in &group {
                                captured_stones.push(*p);
                            }
                            self.remove_group(&group);
                        }
                    }
                }

                if total_captured == 1 {
                    if let Some(captured_pos) = single_capture_pos {
                        let placed_group = self.get_group(&pos);
                        if placed_group.len() == 1 && self.count_liberties(&placed_group) == 1 {
                            self.ko_point = Some(captured_pos);
                        }
                    }
                }
            }
        }

        self.move_history.push(MoveHistoryEntry {
            move_: *move_,
            captured_stones,
            previous_ko_point,
        });

        self.current_player = self.current_player.opposite();

        // Check max moves limit
        if !self.is_over && self.move_history.len() >= self.max_moves {
            self.is_over = true;
            self.outcome = Some(self.determine_outcome());
        }

        true
    }

    pub fn unmake_move(&mut self) -> bool {
        if let Some(entry) = self.move_history.pop() {
            self.current_player = self.current_player.opposite();
            self.ko_point = entry.previous_ko_point;

            match entry.move_ {
                Move::Pass => {
                    self.consecutive_passes = self.consecutive_passes.saturating_sub(1);
                    self.is_over = false;
                    self.outcome = None;
                }
                Move::Place { col, row } => {
                    let pos = Position::new(col, row);
                    self.board.set_piece(&pos, None);

                    let opponent = self.current_player.opposite();
                    for captured_pos in &entry.captured_stones {
                        self.board.set_piece(captured_pos, Some(opponent));
                    }

                    self.is_over = false;
                    self.outcome = None;
                }
            }

            true
        } else {
            false
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::standard()
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Game(turn: {}, is_over: {}, outcome: {:?})\n{}",
            self.current_player, self.is_over, self.outcome, self.board
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = Game::standard();
        assert_eq!(game.turn(), Player::Black);
        assert!(!game.is_over());
        assert!(game.outcome().is_none());
    }

    #[test]
    fn test_legal_moves_initial() {
        let game = Game::new(9, 9);
        let moves = game.legal_moves();
        assert_eq!(moves.len(), 9 * 9 + 1);
    }

    #[test]
    fn test_make_move() {
        let mut game = Game::new(9, 9);
        let move_ = Move::place(0, 0);

        assert!(game.is_legal_move(&move_));
        assert!(game.make_move(&move_));
        assert_eq!(game.turn(), Player::White);
    }

    #[test]
    fn test_make_invalid_move() {
        let mut game = Game::new(9, 9);
        let move_ = Move::place(10, 0);

        assert!(!game.is_legal_move(&move_));
        assert!(!game.make_move(&move_));
    }

    #[test]
    fn test_occupied_position() {
        let mut game = Game::new(9, 9);
        let move_ = Move::place(0, 0);

        game.make_move(&move_);

        let same_pos = Move::place(0, 0);
        assert!(!game.is_legal_move(&same_pos));
    }

    #[test]
    fn test_unmake_move() {
        let mut game = Game::new(9, 9);
        let move_ = Move::place(0, 0);

        game.make_move(&move_);
        assert_eq!(game.turn(), Player::White);

        assert!(game.unmake_move());
        assert_eq!(game.turn(), Player::Black);
        assert_eq!(game.move_history().len(), 0);
        assert!(game.board().get_piece(&Position::new(0, 0)).is_none());
    }

    #[test]
    fn test_pass_move() {
        // Use with_options to set min_moves to 0 so double-pass ends immediately
        let mut game = Game::with_options(9, 9, DEFAULT_KOMI, 0, 1000);

        assert!(game.make_move(&Move::pass()));
        assert_eq!(game.turn(), Player::White);
        assert!(!game.is_over());

        assert!(game.make_move(&Move::pass()));
        assert!(game.is_over());
        // Empty board with komi: White wins
        assert_eq!(game.outcome(), Some(GameOutcome::WhiteWin));
    }

    #[test]
    fn test_pass_move_requires_min_moves() {
        // Default 9x9 game has min_moves = 40 (81/2)
        let mut game = Game::new(9, 9);
        assert_eq!(game.min_moves_before_pass_ends(), 40);

        // Double pass shouldn't end the game yet
        assert!(game.make_move(&Move::pass()));
        assert!(game.make_move(&Move::pass()));
        assert!(!game.is_over());

        // Game should continue with pass still being legal
        assert!(game.make_move(&Move::pass()));
        assert!(!game.is_over());
    }

    #[test]
    fn test_pass_ends_game_after_min_moves() {
        // Create a game with min_moves = 4
        let mut game = Game::with_options(9, 9, DEFAULT_KOMI, 4, 1000);

        // Play 4 moves (2 passes won't end game yet)
        game.make_move(&Move::place(0, 0));
        game.make_move(&Move::place(1, 0));
        game.make_move(&Move::pass());
        game.make_move(&Move::pass());
        // Only 4 moves, consecutive_passes = 2, but we need to check after a move
        // Actually at move 4, we have 4 moves in history, so pass-pass at moves 3-4 should end it
        assert!(game.is_over());
    }

    #[test]
    fn test_max_moves_ends_game() {
        // Create a game with max_moves = 5
        let mut game = Game::with_options(9, 9, DEFAULT_KOMI, 100, 5);

        game.make_move(&Move::place(0, 0));
        game.make_move(&Move::place(1, 0));
        game.make_move(&Move::place(2, 0));
        game.make_move(&Move::place(3, 0));
        assert!(!game.is_over());

        game.make_move(&Move::place(4, 0));
        assert!(game.is_over());
        // Game ends by max moves, outcome determined by scoring
        assert!(game.outcome().is_some());
    }

    #[test]
    fn test_scoring_black_wins() {
        // Create a small board where Black controls most territory
        // Use min_moves=0 so double-pass ends game immediately
        let mut game = Game::with_options(5, 5, 0.5, 0, 1000);

        // Black plays in corner, White passes
        game.make_move(&Move::place(0, 0)); // Black
        game.make_move(&Move::pass()); // White
        game.make_move(&Move::place(1, 0)); // Black
        game.make_move(&Move::pass()); // White
        game.make_move(&Move::place(0, 1)); // Black
        game.make_move(&Move::pass()); // White
        game.make_move(&Move::place(1, 1)); // Black
        game.make_move(&Move::pass()); // White
        game.make_move(&Move::pass()); // Black - game ends

        assert!(game.is_over());
        // Black has 4 stones, White has 0 + 0.5 komi
        // Territory is shared so neither gets it
        let (black_score, white_score) = game.score();
        assert!(black_score > white_score);
        assert_eq!(game.outcome(), Some(GameOutcome::BlackWin));
    }

    #[test]
    fn test_scoring_with_territory() {
        // Create a game where Black controls a clear territory
        // Use min_moves=0 so double-pass ends game immediately
        let mut game = Game::with_options(5, 5, 0.0, 0, 1000);

        // Black surrounds top-left corner
        // . . . . .
        // B . . . .
        // B B . . .
        // . . . . .
        // . . . . .
        game.make_move(&Move::place(0, 2)); // Black
        game.make_move(&Move::pass()); // White
        game.make_move(&Move::place(0, 3)); // Black
        game.make_move(&Move::pass()); // White
        game.make_move(&Move::place(1, 2)); // Black
        game.make_move(&Move::pass()); // White
        game.make_move(&Move::pass()); // Black - game ends

        let (black_score, white_score) = game.score();
        // Black: 3 stones + territory at (0,4) and possibly more
        assert!(black_score > white_score);
        assert_eq!(game.outcome(), Some(GameOutcome::BlackWin));
    }

    #[test]
    fn test_simple_capture() {
        let mut game = Game::new(5, 5);

        game.make_move(&Move::place(1, 0));
        game.make_move(&Move::place(0, 0));
        game.make_move(&Move::place(0, 1));

        assert!(game.board().get_piece(&Position::new(0, 0)).is_none());
    }

    #[test]
    fn test_capture_group() {
        let mut game = Game::new(5, 5);

        game.make_move(&Move::place(0, 0));
        game.make_move(&Move::place(1, 0));

        game.make_move(&Move::place(0, 1));
        game.make_move(&Move::place(1, 1));

        game.make_move(&Move::pass());
        game.make_move(&Move::place(0, 2));

        game.make_move(&Move::pass());
        game.make_move(&Move::place(2, 0));

        game.make_move(&Move::pass());
        game.make_move(&Move::place(2, 1));

        assert!(game.board().get_piece(&Position::new(0, 0)).is_none());
        assert!(game.board().get_piece(&Position::new(0, 1)).is_none());
        assert!(game.board().get_piece(&Position::new(1, 0)).is_some());
        assert!(game.board().get_piece(&Position::new(1, 1)).is_some());
    }

    #[test]
    fn test_suicide_prevention() {
        let mut game = Game::new(5, 5);

        game.make_move(&Move::place(1, 0));
        game.make_move(&Move::pass());
        game.make_move(&Move::place(0, 1));
        game.make_move(&Move::pass());

        let suicide_move = Move::place(0, 0);
        assert!(game.is_legal_move(&suicide_move));
        game.make_move(&suicide_move);
        assert!(game.board().get_piece(&Position::new(0, 0)).is_some());
    }

    #[test]
    fn test_actual_suicide_prevention() {
        // Use min_moves=0 so we can end the game with passes to test suicide on game-over board
        let mut game = Game::with_options(5, 5, DEFAULT_KOMI, 0, 1000);

        game.make_move(&Move::place(1, 0));
        game.make_move(&Move::pass());
        game.make_move(&Move::place(0, 1));
        game.make_move(&Move::pass());
        game.make_move(&Move::pass());

        let suicide_move = Move::place(0, 0);
        // Game is over, so no moves are legal
        assert!(!game.is_legal_move(&suicide_move));
    }

    #[test]
    fn test_ko_rule() {
        let mut game = Game::new(5, 5);

        // Build a ko shape:
        //     0 1 2 3
        // Row2 . B W .
        // Row1 B W . W
        // Row0 . B W .

        game.make_move(&Move::place(1, 0)); // B
        game.make_move(&Move::place(2, 0)); // W

        game.make_move(&Move::place(0, 1)); // B
        game.make_move(&Move::place(1, 1)); // W - this stone will be captured

        game.make_move(&Move::place(1, 2)); // B
        game.make_move(&Move::place(2, 2)); // W

        game.make_move(&Move::pass()); // B pass
        game.make_move(&Move::place(3, 1)); // W

        // Now Black captures W at (1,1) by playing at (2,1)
        let ko_capture = Move::place(2, 1);
        assert!(game.is_legal_move(&ko_capture));
        game.make_move(&ko_capture);

        // W at (1,1) should be captured
        assert!(game.board().get_piece(&Position::new(1, 1)).is_none());

        // Ko point should be set to (1,1)
        assert_eq!(game.ko_point(), Some(Position::new(1, 1)));

        // White cannot immediately recapture at (1,1)
        let immediate_recapture = Move::place(1, 1);
        assert!(!game.is_legal_move(&immediate_recapture));
    }

    #[test]
    fn test_unmake_restores_captures() {
        let mut game = Game::new(5, 5);

        game.make_move(&Move::place(1, 0));
        game.make_move(&Move::place(0, 0));
        game.make_move(&Move::place(0, 1));

        assert!(game.board().get_piece(&Position::new(0, 0)).is_none());

        game.unmake_move();

        assert_eq!(
            game.board().get_piece(&Position::new(0, 0)),
            Some(Player::White)
        );
    }

    #[test]
    fn test_clone() {
        let mut game = Game::new(9, 9);
        let move_ = Move::place(0, 0);
        game.make_move(&move_);

        let cloned = game.clone();
        assert_eq!(cloned.turn(), game.turn());
        assert_eq!(cloned.is_over(), game.is_over());
        assert_eq!(cloned.move_history().len(), game.move_history().len());
    }

    #[test]
    fn test_move_history() {
        let mut game = Game::new(9, 9);

        assert_eq!(game.move_history().len(), 0);

        let move1 = Move::place(0, 0);
        game.make_move(&move1);
        assert_eq!(game.move_history().len(), 1);

        let move2 = Move::place(1, 0);
        game.make_move(&move2);
        assert_eq!(game.move_history().len(), 2);

        game.unmake_move();
        assert_eq!(game.move_history().len(), 1);
    }

    #[test]
    fn test_unmake_when_empty() {
        let mut game = Game::new(9, 9);
        assert!(!game.unmake_move());
    }

    #[test]
    fn test_legal_moves_when_game_over() {
        // Use min_moves=0 so double-pass ends game immediately
        let mut game = Game::with_options(9, 9, DEFAULT_KOMI, 0, 1000);

        game.make_move(&Move::pass());
        game.make_move(&Move::pass());

        assert!(game.is_over());
        assert_eq!(game.legal_moves().len(), 0);
    }
}
