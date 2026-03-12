use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use super::py_move::PyMove;
use crate::gtp::{GenmoveResult, GtpEngine};
use crate::player::Player;

#[pyclass(name = "GtpEngine")]
pub struct PyGtpEngine {
    inner: Option<GtpEngine>,
}

impl PyGtpEngine {
    fn engine(&self) -> PyResult<&GtpEngine> {
        self.inner
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("GTP engine has been shut down"))
    }

    fn engine_mut(&mut self) -> PyResult<&mut GtpEngine> {
        self.inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("GTP engine has been shut down"))
    }
}

fn gtp_err_to_py(e: crate::gtp::GtpError) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}

#[pymethods]
impl PyGtpEngine {
    #[new]
    #[pyo3(signature = (program, args=vec![], size=19, komi=7.5))]
    pub fn new(program: &str, args: Vec<String>, size: u8, komi: f32) -> PyResult<Self> {
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let engine = GtpEngine::new(program, &arg_refs, size, komi).map_err(gtp_err_to_py)?;
        Ok(PyGtpEngine {
            inner: Some(engine),
        })
    }

    /// Play a move for the current turn's player.
    pub fn play(&mut self, m: &PyMove) -> PyResult<()> {
        self.engine_mut()?
            .play(*m.as_inner())
            .map_err(gtp_err_to_py)
    }

    /// Play a move as a specific player (1=Black, -1=White).
    pub fn play_as(&mut self, player: i8, m: &PyMove) -> PyResult<()> {
        let p = Player::from_int(player)
            .ok_or_else(|| PyRuntimeError::new_err("Invalid player value"))?;
        self.engine_mut()?
            .play_as(p, *m.as_inner())
            .map_err(gtp_err_to_py)
    }

    /// Ask the engine to generate a move. Returns a PyMove, or None if the engine resigns.
    pub fn genmove(&mut self) -> PyResult<Option<PyMove>> {
        match self.engine_mut()?.genmove().map_err(gtp_err_to_py)? {
            GenmoveResult::Move(m) => Ok(Some(PyMove::from_move(m))),
            GenmoveResult::Resign => Ok(None),
        }
    }

    /// Ask the engine to generate a move as a specific player.
    pub fn genmove_as(&mut self, player: i8) -> PyResult<Option<PyMove>> {
        let p = Player::from_int(player)
            .ok_or_else(|| PyRuntimeError::new_err("Invalid player value"))?;
        match self.engine_mut()?.genmove_as(p).map_err(gtp_err_to_py)? {
            GenmoveResult::Move(m) => Ok(Some(PyMove::from_move(m))),
            GenmoveResult::Resign => Ok(None),
        }
    }

    /// Undo the last move.
    pub fn undo(&mut self) -> PyResult<()> {
        self.engine_mut()?.undo().map_err(gtp_err_to_py)
    }

    /// Clear the board.
    pub fn clear_board(&mut self) -> PyResult<()> {
        self.engine_mut()?.clear_board().map_err(gtp_err_to_py)
    }

    /// Set komi.
    pub fn set_komi(&mut self, komi: f32) -> PyResult<()> {
        self.engine_mut()?.set_komi(komi).map_err(gtp_err_to_py)
    }

    /// Get the current turn (1=Black, -1=White).
    pub fn turn(&self) -> PyResult<i8> {
        Ok(self.engine()?.turn() as i8)
    }

    /// Check if the game is over.
    pub fn is_over(&self) -> PyResult<bool> {
        Ok(self.engine()?.is_over())
    }

    /// Get legal moves.
    pub fn legal_moves(&self) -> PyResult<Vec<PyMove>> {
        Ok(self
            .engine()?
            .legal_moves()
            .into_iter()
            .map(PyMove::from_move)
            .collect())
    }

    /// Get the score as (black_score, white_score).
    pub fn score(&self) -> PyResult<(f32, f32)> {
        Ok(self.engine()?.score())
    }

    /// Get the board size.
    pub fn size(&self) -> PyResult<u8> {
        Ok(self.engine()?.size())
    }

    /// Get the engine's name, or None if the engine doesn't support it.
    pub fn engine_name(&mut self) -> PyResult<Option<String>> {
        Ok(self.engine_mut()?.engine_name())
    }

    /// Get the engine's version, or None if the engine doesn't support it.
    pub fn engine_version(&mut self) -> PyResult<Option<String>> {
        Ok(self.engine_mut()?.engine_version())
    }

    /// Send a raw GTP command. Returns the response string.
    pub fn send_command(&mut self, cmd: &str, args: Vec<String>) -> PyResult<String> {
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        self.engine_mut()?
            .client()
            .send_command(cmd, &arg_refs)
            .map_err(gtp_err_to_py)
    }

    /// Shut down the engine process.
    pub fn quit(&mut self) -> PyResult<()> {
        if let Some(mut engine) = self.inner.take() {
            let _ = engine.client().quit();
        }
        Ok(())
    }
}
