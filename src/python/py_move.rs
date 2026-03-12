use pyo3::prelude::*;

use crate::encode;
use crate::r#move::Move;

#[pyclass(name = "Move")]
#[derive(Clone, Debug)]
pub struct PyMove {
    pub(super) move_: Move,
}

#[hotpath::measure_all]
impl PyMove {
    pub(super) fn from_move(move_: Move) -> Self {
        PyMove { move_ }
    }

    pub(super) fn as_inner(&self) -> &Move {
        &self.move_
    }
}

#[hotpath::measure_all]
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
