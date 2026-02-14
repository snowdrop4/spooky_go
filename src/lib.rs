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
    use crate::bitboard::nw_for_board;
    use crate::board::Board;
    use crate::encode;
    use crate::game::Game;
    use crate::outcome::GameOutcome;
    use crate::player::Player;
    use crate::position::Position;
    use crate::r#move::Move;

    // -----------------------------------------------------------------------
    // Enum dispatch via paste! for Game<NW> and Board<NW>
    // -----------------------------------------------------------------------

    macro_rules! define_dispatch {
        ($($nw:literal),*) => {
            paste::paste! {
                #[derive(Clone, Debug)]
                enum GameInner {
                    $( [<Nw $nw>](Game<$nw>), )*
                }

                #[derive(Clone, Debug)]
                enum BoardInner {
                    $( [<Nw $nw>](Board<$nw>), )*
                }

                macro_rules! dispatch_game {
                    ($self_:expr, $g:ident => $body:expr) => {
                        match $self_ {
                            $( GameInner::[<Nw $nw>]($g) => $body, )*
                        }
                    };
                }

                macro_rules! dispatch_game_mut {
                    ($self_:expr, $g:ident => $body:expr) => {
                        match $self_ {
                            $( GameInner::[<Nw $nw>]($g) => $body, )*
                        }
                    };
                }

                macro_rules! dispatch_board {
                    ($self_:expr, $b:ident => $body:expr) => {
                        match $self_ {
                            $( BoardInner::[<Nw $nw>]($b) => $body, )*
                        }
                    };
                }

                macro_rules! dispatch_board_mut {
                    ($self_:expr, $b:ident => $body:expr) => {
                        match $self_ {
                            $( BoardInner::[<Nw $nw>]($b) => $body, )*
                        }
                    };
                }

                fn make_game_inner(width: u8, height: u8) -> GameInner {
                    let nw = nw_for_board(width, height);
                    match nw {
                        $( $nw => GameInner::[<Nw $nw>](Game::new(width, height)), )*
                        _ => unreachable!("NW out of range: {}", nw),
                    }
                }

                fn make_game_inner_with_options(
                    width: u8, height: u8, komi: f32,
                    min_moves: u16, max_moves: u16, superko: bool,
                ) -> GameInner {
                    let nw = nw_for_board(width, height);
                    match nw {
                        $( $nw => GameInner::[<Nw $nw>](Game::with_options(
                            width, height, komi, min_moves, max_moves, superko
                        )), )*
                        _ => unreachable!("NW out of range: {}", nw),
                    }
                }

                fn make_board_inner(width: u8, height: u8) -> BoardInner {
                    let nw = nw_for_board(width, height);
                    match nw {
                        $( $nw => BoardInner::[<Nw $nw>](Board::new(width, height)), )*
                        _ => unreachable!("NW out of range: {}", nw),
                    }
                }

                macro_rules! game_to_board_inner {
                    ($game_inner:expr) => {
                        match $game_inner {
                            $( GameInner::[<Nw $nw>](g) => BoardInner::[<Nw $nw>](*g.board()), )*
                        }
                    };
                }
            }
        }
    }

    define_dispatch!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

    // -----------------------------------------------------------------------
    // PyBoard
    // -----------------------------------------------------------------------

    #[pyclass(name = "Board")]
    #[derive(Clone)]
    pub struct PyBoard {
        inner: BoardInner,
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
                inner: make_board_inner(width as u8, height as u8),
            })
        }

        #[staticmethod]
        pub fn standard() -> Self {
            PyBoard {
                inner: make_board_inner(19, 19),
            }
        }

        pub fn width(&self) -> usize {
            dispatch_board!(&self.inner, b => b.width() as usize)
        }

        pub fn height(&self) -> usize {
            dispatch_board!(&self.inner, b => b.height() as usize)
        }

        pub fn get_piece(&self, col: usize, row: usize) -> Option<i8> {
            let pos = Position::new(col as u8, row as u8);
            dispatch_board!(&self.inner, b => b.get_piece(&pos).map(|p| p as i8))
        }

        pub fn set_piece(&mut self, col: usize, row: usize, piece: Option<i8>) {
            let pos = Position::new(col as u8, row as u8);
            let player = piece.map(|p| Player::from_int(p).expect("Invalid player value"));
            dispatch_board_mut!(&mut self.inner, b => b.set_piece(&pos, player))
        }

        pub fn clear(&mut self) {
            dispatch_board_mut!(&mut self.inner, b => b.clear())
        }

        pub fn __str__(&self) -> String {
            dispatch_board!(&self.inner, b => b.to_string())
        }

        pub fn __repr__(&self) -> String {
            let w = self.width();
            let h = self.height();
            format!("Board(width={}, height={})", w, h)
        }
    }

    // -----------------------------------------------------------------------
    // PyGame
    // -----------------------------------------------------------------------

    #[pyclass(name = "Game")]
    pub struct PyGame {
        inner: GameInner,
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
                inner: make_game_inner(width as u8, height as u8),
            })
        }

        #[staticmethod]
        #[pyo3(signature = (width, height, komi, min_moves_before_pass_ends, max_moves, superko=false))]
        pub fn with_options(
            width: usize,
            height: usize,
            komi: f32,
            min_moves_before_pass_ends: usize,
            max_moves: usize,
            superko: bool,
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
                inner: make_game_inner_with_options(
                    width as u8,
                    height as u8,
                    komi,
                    min_moves_before_pass_ends as u16,
                    max_moves as u16,
                    superko,
                ),
            })
        }

        #[staticmethod]
        pub fn standard() -> Self {
            PyGame {
                inner: make_game_inner(19, 19),
            }
        }

        pub fn komi(&self) -> f32 {
            dispatch_game!(&self.inner, g => g.komi())
        }

        pub fn min_moves_before_pass_ends(&self) -> usize {
            dispatch_game!(&self.inner, g => g.min_moves_before_pass_ends() as usize)
        }

        pub fn max_moves(&self) -> usize {
            dispatch_game!(&self.inner, g => g.max_moves() as usize)
        }

        pub fn move_count(&self) -> usize {
            dispatch_game!(&self.inner, g => g.move_count())
        }

        pub fn score(&self) -> (f32, f32) {
            dispatch_game!(&self.inner, g => g.score())
        }

        pub fn width(&self) -> usize {
            dispatch_game!(&self.inner, g => g.width() as usize)
        }

        pub fn height(&self) -> usize {
            dispatch_game!(&self.inner, g => g.height() as usize)
        }

        pub fn get_piece(&self, col: usize, row: usize) -> Option<i8> {
            let pos = Position::new(col as u8, row as u8);
            dispatch_game!(&self.inner, g => g.get_piece(&pos).map(|p| p as i8))
        }

        pub fn set_piece(&mut self, col: usize, row: usize, piece: Option<i8>) {
            let pos = Position::new(col as u8, row as u8);
            let player = piece.map(|p| Player::from_int(p).expect("Invalid player value"));
            dispatch_game_mut!(&mut self.inner, g => g.set_piece(&pos, player))
        }

        pub fn turn(&self) -> i8 {
            dispatch_game!(&self.inner, g => g.turn() as i8)
        }

        pub fn is_over(&self) -> bool {
            dispatch_game!(&self.inner, g => g.is_over())
        }

        // ---------------------------------------------------------------------
        // Unified Game Protocol Methods
        // ---------------------------------------------------------------------

        pub fn legal_action_indices(&self) -> Vec<usize> {
            dispatch_game!(&self.inner, g => {
                let w = g.width();
                let h = g.height();
                g.legal_moves()
                    .into_iter()
                    .map(|m| encode::encode_move(&m, w, h))
                    .collect()
            })
        }

        pub fn apply_action(&mut self, action: usize) -> bool {
            dispatch_game_mut!(&mut self.inner, g => {
                let w = g.width();
                let h = g.height();
                if let Some(move_) = encode::decode_move(action, w, h) {
                    g.make_move(&move_)
                } else {
                    false
                }
            })
        }

        pub fn action_size(&self) -> usize {
            dispatch_game!(&self.inner, g => encode::total_actions(g.width(), g.height()))
        }

        pub fn board_shape(&self) -> (usize, usize) {
            dispatch_game!(&self.inner, g => (g.height() as usize, g.width() as usize))
        }

        pub fn input_plane_count(&self) -> usize {
            encode::TOTAL_INPUT_PLANES
        }

        pub fn reward_absolute(&self) -> f32 {
            dispatch_game!(&self.inner, g => {
                g.outcome()
                    .map(|o| o.encode_winner_absolute())
                    .unwrap_or(0.0)
            })
        }

        pub fn reward_from_perspective(&self, perspective: i8) -> f32 {
            dispatch_game!(&self.inner, g => {
                g.outcome()
                    .map(|o| {
                        o.encode_winner_from_perspective(
                            Player::from_int(perspective).expect("Invalid perspective"),
                        )
                    })
                    .unwrap_or(0.0)
            })
        }

        pub fn name(&self) -> String {
            dispatch_game!(&self.inner, g => format!("go_{}x{}", g.width(), g.height()))
        }

        pub fn outcome(&self) -> Option<PyGameOutcome> {
            dispatch_game!(&self.inner, g => g.outcome().map(|o| PyGameOutcome { outcome: o }))
        }

        pub fn legal_moves(&self) -> Vec<PyMove> {
            dispatch_game!(&self.inner, g => {
                g.legal_moves()
                    .into_iter()
                    .map(|m| PyMove { move_: m })
                    .collect()
            })
        }

        pub fn is_legal_move(&self, move_: &PyMove) -> bool {
            dispatch_game!(&self.inner, g => g.is_legal_move(&move_.move_))
        }

        pub fn make_move(&mut self, move_: &PyMove) -> bool {
            dispatch_game_mut!(&mut self.inner, g => g.make_move(&move_.move_))
        }

        pub fn unmake_move(&mut self) -> bool {
            dispatch_game_mut!(&mut self.inner, g => g.unmake_move())
        }

        pub fn board(&self) -> PyBoard {
            PyBoard {
                inner: game_to_board_inner!(&self.inner),
            }
        }

        pub fn superko(&self) -> bool {
            dispatch_game!(&self.inner, g => g.superko())
        }

        pub fn ko_point(&self) -> Option<(usize, usize)> {
            dispatch_game!(&self.inner, g => {
                g.ko_point().map(|p| (p.col as usize, p.row as usize))
            })
        }

        pub fn clone(&self) -> PyGame {
            PyGame {
                inner: self.inner.clone(),
            }
        }

        pub fn __hash__(&self) -> u64 {
            use std::hash::{Hash, Hasher};
            dispatch_game!(&self.inner, g => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                g.board().hash(&mut hasher);
                (g.turn() as i8).hash(&mut hasher);
                g.ko_point().hash(&mut hasher);
                hasher.finish()
            })
        }

        pub fn encode_game_planes(&self) -> (Vec<f32>, usize, usize, usize) {
            dispatch_game!(&self.inner, g => encode::encode_game_planes(g))
        }

        pub fn decode_action(&self, action: usize) -> Option<PyMove> {
            dispatch_game!(&self.inner, g => {
                let w = g.width();
                let h = g.height();
                encode::decode_move(action, w, h).map(|move_| PyMove { move_ })
            })
        }

        pub fn total_actions(&self) -> usize {
            dispatch_game!(&self.inner, g => encode::total_actions(g.width(), g.height()))
        }

        pub fn __str__(&self) -> String {
            dispatch_game!(&self.inner, g => g.to_string())
        }

        pub fn __repr__(&self) -> String {
            dispatch_game!(&self.inner, g => {
                format!(
                    "Game(width={}, height={}, turn={:?}, over={}, superko={})",
                    g.width(),
                    g.height(),
                    g.turn(),
                    g.is_over(),
                    g.superko()
                )
            })
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
