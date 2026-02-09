use crate::bitboard::{Bitboard, BoardGeometry};
use crate::board::{Board, STANDARD_COLS, STANDARD_ROWS};
use crate::outcome::GameOutcome;
use crate::player::Player;
use crate::position::Position;
use crate::r#move::Move;

#[derive(Clone, Debug)]
struct MoveHistoryEntry {
    move_: Move,
    captured_stones: Bitboard,
    previous_ko_point: Option<Position>,
}

pub const DEFAULT_KOMI: f32 = 7.5;

#[derive(Clone, Debug)]
pub struct Game {
    board: Board,
    geo: BoardGeometry,
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
            geo: BoardGeometry::new(width, height),
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

        black_score += self.board.black_stones().count() as f32;
        white_score += self.board.white_stones().count() as f32;

        let occupied = self.board.occupied();
        let mut remaining_empty = self.board.empty_squares(self.geo.board_mask);

        while let Some(idx) = remaining_empty.lowest_bit_index() {
            let seed = Bitboard::single(idx);
            let empty_mask = self.geo.board_mask & !occupied;
            let region = self.geo.flood_fill(seed, empty_mask);

            // Remove this region from remaining
            remaining_empty &= !region;

            // Check which players are adjacent to this region
            let region_neighbors = self.geo.neighbors(&region);
            let black_adjacent = (region_neighbors & self.board.black_stones()).is_nonzero();
            let white_adjacent = (region_neighbors & self.board.white_stones()).is_nonzero();

            let territory = region.count() as f32;
            match (black_adjacent, white_adjacent) {
                (true, false) => black_score += territory,
                (false, true) => white_score += territory,
                _ => {}
            }
        }

        (black_score, white_score)
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

    /// Check if placing a stone at `idx` for `player` would be suicide.
    /// Zero heap allocations — works entirely on stack-based bitboards.
    fn would_be_suicide(&self, idx: usize, player: Player) -> bool {
        let nw = self.geo.nw;
        let bit = Bitboard::single(idx);
        let own = self.board.stones_for(player).or_w(bit, nw);
        let opp = self.board.stones_for(player.opposite());
        let occupied = own.or_w(opp, nw);
        let empty = self.geo.board_mask.andnot_w(occupied, nw);

        // Fast path: if the new stone itself has any empty neighbor,
        // the group containing it has at least one liberty → not suicide.
        // This skips the expensive flood-fill for ~95% of positions.
        let bit_neighbors = self.geo.neighbors(&bit);
        if bit_neighbors.and_w(empty, nw).is_nonzero_w(nw) {
            return false;
        }

        // Slow path: stone has no empty neighbors. Flood-fill to find full group.
        let group = self.geo.flood_fill(bit, own);

        // Check if this group has any liberties
        let group_neighbors = self.geo.neighbors(&group);
        if group_neighbors.and_w(empty, nw).is_nonzero_w(nw) {
            return false; // has liberties — not suicide
        }

        // No liberties, but check if any adjacent opponent group would be captured
        let adjacent_opponent = group_neighbors.and_w(opp, nw);
        if adjacent_opponent.is_empty_w(nw) {
            return true; // no adjacent opponents to capture — it's suicide
        }

        // Check each adjacent opponent group
        let mut remaining_adj_opp = adjacent_opponent;
        while let Some(opp_idx) = remaining_adj_opp.lowest_bit_index() {
            let opp_seed = Bitboard::single(opp_idx);
            let opp_group = self.geo.flood_fill(opp_seed, opp);

            // Remove this entire group from remaining so we don't re-check it
            remaining_adj_opp = remaining_adj_opp.andnot_w(opp_group, nw);

            // This opponent group's liberties (excluding the new stone's position)
            let opp_group_neighbors = self.geo.neighbors(&opp_group);
            let opp_liberties = opp_group_neighbors.and_w(empty, nw);

            if opp_liberties.is_empty_w(nw) {
                // This opponent group has no liberties (our new stone took the last one)
                // → it would be captured → not suicide
                return false;
            }
        }

        true // no opponent group can be captured, own group has no liberties → suicide
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        if self.is_over {
            return Vec::new();
        }

        let mut moves = Vec::new();
        let empty = self.board.empty_squares(self.geo.board_mask);
        let ko_idx = self.ko_point.map(|p| p.to_index(self.geo.width));

        for idx in empty.iter_ones() {
            if let Some(ki) = ko_idx {
                if ki == idx {
                    continue;
                }
            }

            if self.would_be_suicide(idx, self.current_player) {
                continue;
            }

            let pos = Position::from_index(idx, self.geo.width);
            moves.push(Move::place(pos.col, pos.row));
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

                let idx = pos.to_index(self.geo.width);

                if self.board.occupied().get(idx) {
                    return false;
                }

                if let Some(ko) = self.ko_point {
                    if ko == pos {
                        return false;
                    }
                }

                if self.would_be_suicide(idx, self.current_player) {
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
        let mut captured_stones = Bitboard::empty();
        self.ko_point = None;

        match move_ {
            Move::Pass => {
                self.consecutive_passes += 1;

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
                let idx = pos.to_index(self.geo.width);
                self.board.set_bit(idx, self.current_player);

                let opponent = self.current_player.opposite();
                let bit = Bitboard::single(idx);
                let bit_neighbors = self.geo.neighbors(&bit);
                let adjacent_opponent = bit_neighbors & self.board.stones_for(opponent);

                let mut total_captured: u32 = 0;
                let mut single_capture_idx: Option<usize> = None;

                // Check each adjacent opponent group for capture
                let mut remaining = adjacent_opponent;
                while let Some(opp_idx) = remaining.lowest_bit_index() {
                    let opp_seed = Bitboard::single(opp_idx);
                    let opp_group =
                        self.geo.flood_fill(opp_seed, self.board.stones_for(opponent));

                    // Remove this group from remaining
                    remaining &= !opp_group;

                    // Check if this group has any liberties
                    let opp_neighbors = self.geo.neighbors(&opp_group);
                    let opp_empty = self.board.empty_squares(self.geo.board_mask);
                    if (opp_neighbors & opp_empty).is_empty() {
                        // Captured!
                        let group_size = opp_group.count();
                        if group_size == 1 && total_captured == 0 {
                            single_capture_idx = Some(opp_idx);
                        } else {
                            single_capture_idx = None;
                        }
                        total_captured += group_size;
                        captured_stones |= opp_group;
                        self.board.remove_stones(opp_group);
                    }
                }

                // Ko detection
                if total_captured == 1 {
                    if let Some(cap_idx) = single_capture_idx {
                        let placed_group =
                            self.geo
                                .flood_fill(bit, self.board.stones_for(self.current_player));
                        if placed_group.count() == 1 {
                            let placed_neighbors = self.geo.neighbors(&placed_group);
                            let placed_liberties =
                                placed_neighbors & self.board.empty_squares(self.geo.board_mask);
                            if placed_liberties.count() == 1 {
                                self.ko_point =
                                    Some(Position::from_index(cap_idx, self.geo.width));
                            }
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
                    let idx = pos.to_index(self.geo.width);
                    self.board.clear_bit(idx);

                    // Restore captured stones for the opponent
                    let opponent = self.current_player.opposite();
                    self.board.restore_stones(entry.captured_stones, opponent);

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
        assert!(game.outcome().is_some());
    }

    #[test]
    fn test_scoring_black_wins() {
        let mut game = Game::with_options(5, 5, 0.5, 0, 1000);

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
        let (black_score, white_score) = game.score();
        assert!(black_score > white_score);
        assert_eq!(game.outcome(), Some(GameOutcome::BlackWin));
    }

    #[test]
    fn test_scoring_with_territory() {
        let mut game = Game::with_options(5, 5, 0.0, 0, 1000);

        game.make_move(&Move::place(0, 2)); // Black
        game.make_move(&Move::pass()); // White
        game.make_move(&Move::place(0, 3)); // Black
        game.make_move(&Move::pass()); // White
        game.make_move(&Move::place(1, 2)); // Black
        game.make_move(&Move::pass()); // White
        game.make_move(&Move::pass()); // Black - game ends

        let (black_score, white_score) = game.score();
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
        let mut game = Game::with_options(5, 5, DEFAULT_KOMI, 0, 1000);

        game.make_move(&Move::place(1, 0));
        game.make_move(&Move::pass());
        game.make_move(&Move::place(0, 1));
        game.make_move(&Move::pass());
        game.make_move(&Move::pass());

        let suicide_move = Move::place(0, 0);
        assert!(!game.is_legal_move(&suicide_move));
    }

    #[test]
    fn test_ko_rule() {
        let mut game = Game::new(5, 5);

        game.make_move(&Move::place(1, 0)); // B
        game.make_move(&Move::place(2, 0)); // W

        game.make_move(&Move::place(0, 1)); // B
        game.make_move(&Move::place(1, 1)); // W

        game.make_move(&Move::place(1, 2)); // B
        game.make_move(&Move::place(2, 2)); // W

        game.make_move(&Move::pass()); // B pass
        game.make_move(&Move::place(3, 1)); // W

        let ko_capture = Move::place(2, 1);
        assert!(game.is_legal_move(&ko_capture));
        game.make_move(&ko_capture);

        assert!(game.board().get_piece(&Position::new(1, 1)).is_none());
        assert_eq!(game.ko_point(), Some(Position::new(1, 1)));

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
        let mut game = Game::with_options(9, 9, DEFAULT_KOMI, 0, 1000);

        game.make_move(&Move::pass());
        game.make_move(&Move::pass());

        assert!(game.is_over());
        assert_eq!(game.legal_moves().len(), 0);
    }
}
