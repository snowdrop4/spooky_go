pub mod bitboard;
pub mod board;
pub mod encode;
pub mod game;
pub mod r#move;
pub mod outcome;
pub mod player;
pub mod position;

#[cfg(feature = "serde")]
pub mod serde_support;

#[cfg(feature = "python")]
extern crate pyo3;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule(gil_used = false)]
fn spooky_go(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use player::Player;
    use python_bindings::*;
    m.add_class::<PyBoard>()?;
    m.add_class::<PyGame>()?;
    m.add_class::<PyMove>()?;
    m.add_class::<PyGameOutcome>()?;
    m.add("BLACK", Player::Black as i8)?;
    m.add("WHITE", Player::White as i8)?;
    m.add("TOTAL_INPUT_PLANES", encode::TOTAL_INPUT_PLANES)?;
    Ok(())
}

#[cfg(feature = "python")]
mod python_bindings {
    use super::*;
    use crate::board::Board;
    use crate::encode;
    use crate::game::Game;
    use crate::outcome::GameOutcome;
    use crate::player::Player;
    use crate::position::Position;
    use crate::r#move::Move;

    #[pyclass(name = "Board")]
    #[derive(Clone)]
    pub struct PyBoard {
        board: Board,
    }

    #[pymethods]
    impl PyBoard {
        #[new]
        pub fn new(width: usize, height: usize) -> PyResult<Self> {
            if !(2..=32).contains(&width) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board width must be between 2 and 32",
                ));
            }
            if !(2..=32).contains(&height) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board height must be between 2 and 32",
                ));
            }
            Ok(PyBoard {
                board: Board::new(width as u8, height as u8),
            })
        }

        #[staticmethod]
        pub fn standard() -> Self {
            PyBoard {
                board: Board::standard(),
            }
        }

        pub fn width(&self) -> usize {
            self.board.width() as usize
        }

        pub fn height(&self) -> usize {
            self.board.height() as usize
        }

        pub fn get_piece(&self, col: usize, row: usize) -> Option<i8> {
            let pos = Position::new(col as u8, row as u8);
            self.board.get_piece(&pos).map(|p| p as i8)
        }

        pub fn set_piece(&mut self, col: usize, row: usize, piece: Option<i8>) {
            let pos = Position::new(col as u8, row as u8);
            let player = piece.map(|p| Player::from_int(p).expect("Invalid player value"));
            self.board.set_piece(&pos, player)
        }

        pub fn clear(&mut self) {
            self.board.clear()
        }

        pub fn __str__(&self) -> String {
            self.board.to_string()
        }

        pub fn __repr__(&self) -> String {
            format!(
                "Board(width={}, height={})",
                self.board.width(),
                self.board.height()
            )
        }
    }

    #[pyclass(name = "Game")]
    pub struct PyGame {
        game: Game,
    }

    #[pymethods]
    impl PyGame {
        #[new]
        pub fn new(width: usize, height: usize) -> PyResult<Self> {
            if !(2..=32).contains(&width) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board width must be between 2 and 32",
                ));
            }
            if !(2..=32).contains(&height) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board height must be between 2 and 32",
                ));
            }
            Ok(PyGame {
                game: Game::new(width as u8, height as u8),
            })
        }

        #[staticmethod]
        pub fn with_komi(width: usize, height: usize, komi: f32) -> PyResult<Self> {
            if !(2..=32).contains(&width) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board width must be between 2 and 32",
                ));
            }
            if !(2..=32).contains(&height) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board height must be between 2 and 32",
                ));
            }
            Ok(PyGame {
                game: Game::with_komi(width as u8, height as u8, komi),
            })
        }

        #[staticmethod]
        pub fn with_options(
            width: usize,
            height: usize,
            komi: f32,
            min_moves_before_pass_ends: usize,
            max_moves: usize,
        ) -> PyResult<Self> {
            if !(2..=32).contains(&width) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board width must be between 2 and 32",
                ));
            }
            if !(2..=32).contains(&height) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board height must be between 2 and 32",
                ));
            }
            Ok(PyGame {
                game: Game::with_options(
                    width as u8,
                    height as u8,
                    komi,
                    min_moves_before_pass_ends as u16,
                    max_moves as u16,
                ),
            })
        }

        #[staticmethod]
        pub fn standard() -> Self {
            PyGame {
                game: Game::standard(),
            }
        }

        pub fn komi(&self) -> f32 {
            self.game.komi()
        }

        pub fn min_moves_before_pass_ends(&self) -> usize {
            self.game.min_moves_before_pass_ends() as usize
        }

        pub fn max_moves(&self) -> usize {
            self.game.max_moves() as usize
        }

        pub fn move_count(&self) -> usize {
            self.game.move_count()
        }

        pub fn score(&self) -> (f32, f32) {
            self.game.score()
        }

        pub fn width(&self) -> usize {
            self.game.board().width() as usize
        }

        pub fn height(&self) -> usize {
            self.game.board().height() as usize
        }

        pub fn get_piece(&self, col: usize, row: usize) -> Option<i8> {
            let pos = Position::new(col as u8, row as u8);
            self.game.get_piece(&pos).map(|p| p as i8)
        }

        pub fn set_piece(&mut self, col: usize, row: usize, piece: Option<i8>) {
            let pos = Position::new(col as u8, row as u8);
            let player = piece.map(|p| Player::from_int(p).expect("Invalid player value"));
            self.game.set_piece(&pos, player)
        }

        pub fn turn(&self) -> i8 {
            self.game.turn() as i8
        }

        pub fn is_over(&self) -> bool {
            self.game.is_over()
        }

        // ---------------------------------------------------------------------
        // Unified Game Protocol Methods
        // ---------------------------------------------------------------------

        pub fn legal_action_indices(&self) -> Vec<usize> {
            let w = self.game.width();
            let h = self.game.height();
            self.game
                .legal_moves()
                .into_iter()
                .map(|m| encode::encode_move(&m, w, h))
                .collect()
        }

        pub fn apply_action(&mut self, action: usize) -> bool {
            let w = self.game.width();
            let h = self.game.height();
            if let Some(move_) = encode::decode_move(action, w, h) {
                self.game.make_move(&move_)
            } else {
                false
            }
        }

        pub fn action_size(&self) -> usize {
            encode::total_actions(self.game.width(), self.game.height())
        }

        pub fn board_shape(&self) -> (usize, usize) {
            (self.game.height() as usize, self.game.width() as usize)
        }

        pub fn input_plane_count(&self) -> usize {
            encode::TOTAL_INPUT_PLANES
        }

        pub fn reward_absolute(&self) -> f32 {
            self.game
                .outcome()
                .map(|o| o.encode_winner_absolute())
                .unwrap_or(0.0)
        }

        pub fn reward_from_perspective(&self, perspective: i8) -> f32 {
            self.game
                .outcome()
                .map(|o| {
                    o.encode_winner_from_perspective(
                        Player::from_int(perspective).expect("Invalid perspective"),
                    )
                })
                .unwrap_or(0.0)
        }

        pub fn name(&self) -> String {
            format!("go_{}x{}", self.game.width(), self.game.height())
        }

        pub fn outcome(&self) -> Option<PyGameOutcome> {
            self.game.outcome().map(|o| PyGameOutcome { outcome: o })
        }

        pub fn legal_moves(&self) -> Vec<PyMove> {
            self.game
                .legal_moves()
                .into_iter()
                .map(|m| PyMove { move_: m })
                .collect()
        }

        pub fn is_legal_move(&self, move_: &PyMove) -> bool {
            self.game.is_legal_move(&move_.move_)
        }

        pub fn make_move(&mut self, move_: &PyMove) -> bool {
            self.game.make_move(&move_.move_)
        }

        pub fn unmake_move(&mut self) -> bool {
            self.game.unmake_move()
        }

        pub fn board(&self) -> PyBoard {
            PyBoard {
                board: self.game.board().clone(),
            }
        }

        pub fn ko_point(&self) -> Option<(usize, usize)> {
            self.game
                .ko_point()
                .map(|p| (p.col as usize, p.row as usize))
        }

        pub fn clone(&self) -> PyGame {
            PyGame {
                game: self.game.clone(),
            }
        }

        pub fn __hash__(&self) -> u64 {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();

            self.game.board().hash(&mut hasher);
            (self.game.turn() as i8).hash(&mut hasher);
            self.game.ko_point().hash(&mut hasher);

            hasher.finish()
        }

        pub fn encode_game_planes(&self) -> (Vec<f32>, usize, usize, usize) {
            encode::encode_game_planes(&self.game)
        }

        pub fn decode_action(&self, action: usize) -> Option<PyMove> {
            let w = self.game.width();
            let h = self.game.height();
            encode::decode_move(action, w, h).map(|move_| PyMove { move_ })
        }

        pub fn total_actions(&self) -> usize {
            encode::total_actions(self.game.width(), self.game.height())
        }

        pub fn __str__(&self) -> String {
            self.game.to_string()
        }

        pub fn __repr__(&self) -> String {
            format!(
                "Game(width={}, height={}, turn={:?}, over={})",
                self.game.board().width(),
                self.game.board().height(),
                self.game.turn(),
                self.game.is_over()
            )
        }
    }

    #[pyclass(name = "Move")]
    #[derive(Clone, Debug)]
    pub struct PyMove {
        move_: Move,
    }

    #[pymethods]
    impl PyMove {
        #[staticmethod]
        pub fn place(col: usize, row: usize) -> Self {
            PyMove {
                move_: Move::place(col as u8, row as u8),
            }
        }

        #[staticmethod]
        pub fn pass_move() -> Self {
            PyMove {
                move_: Move::pass(),
            }
        }

        pub fn is_pass(&self) -> bool {
            self.move_.is_pass()
        }

        pub fn col(&self) -> Option<usize> {
            self.move_.col().map(|c| c as usize)
        }

        pub fn row(&self) -> Option<usize> {
            self.move_.row().map(|r| r as usize)
        }

        pub fn encode(&self, board_width: usize, board_height: usize) -> usize {
            encode::encode_move(&self.move_, board_width as u8, board_height as u8)
        }

        #[staticmethod]
        pub fn decode(action: usize, board_width: usize, board_height: usize) -> PyResult<Self> {
            match encode::decode_move(action, board_width as u8, board_height as u8) {
                Some(mv) => Ok(PyMove { move_: mv }),
                _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "invalid action",
                )),
            }
        }

        pub fn __str__(&self) -> String {
            self.move_.to_string()
        }

        pub fn __repr__(&self) -> String {
            match &self.move_ {
                Move::Place { col, row } => format!("Move.place({}, {})", col, row),
                Move::Pass => "Move.pass_move()".to_string(),
            }
        }

        pub fn __eq__(&self, other: &PyMove) -> bool {
            self.move_ == other.move_
        }

        pub fn __hash__(&self) -> u64 {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            self.move_.hash(&mut hasher);
            hasher.finish()
        }
    }

    #[pyclass(name = "GameOutcome")]
    #[derive(Clone, Copy, Debug)]
    pub struct PyGameOutcome {
        outcome: GameOutcome,
    }

    #[pymethods]
    impl PyGameOutcome {
        pub fn winner(&self) -> Option<i8> {
            self.outcome.winner().map(|player| player as i8)
        }

        pub fn encode_winner_absolute(&self) -> f32 {
            self.outcome.encode_winner_absolute()
        }

        pub fn encode_winner_from_perspective(&self, perspective: i8) -> f32 {
            self.outcome.encode_winner_from_perspective(
                Player::from_int(perspective).expect("Unrecognized perspective"),
            )
        }

        pub fn is_draw(&self) -> bool {
            self.outcome.is_draw()
        }

        pub fn name(&self) -> String {
            self.outcome.to_string()
        }

        pub fn __str__(&self) -> String {
            self.outcome.to_string()
        }

        pub fn __repr__(&self) -> String {
            format!("GameOutcome({})", self.outcome)
        }

        pub fn __eq__(&self, other: &PyGameOutcome) -> bool {
            self.outcome == other.outcome
        }
    }
}
