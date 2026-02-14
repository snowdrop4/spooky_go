use std::fmt;
use std::hash::{Hash, Hasher};

use crate::bitboard::Bitboard;
use crate::player::Player;
use crate::position::Position;

pub const STANDARD_COLS: u8 = 19;
pub const STANDARD_ROWS: u8 = 19;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    black: Bitboard,
    white: Bitboard,
    width: u8,
    height: u8,
}

impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.black.hash(state);
        self.white.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl Board {
    pub fn new(width: u8, height: u8) -> Self {
        Board {
            black: Bitboard::empty(),
            white: Bitboard::empty(),
            width,
            height,
        }
    }

    pub fn standard() -> Self {
        Self::new(STANDARD_COLS, STANDARD_ROWS)
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn get_piece(&self, pos: &Position) -> Option<Player> {
        if pos.is_valid(self.width, self.height) {
            let idx = pos.to_index(self.width);
            if self.black.get(idx) {
                Some(Player::Black)
            } else if self.white.get(idx) {
                Some(Player::White)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_piece(&mut self, pos: &Position, player: Option<Player>) {
        if pos.is_valid(self.width, self.height) {
            let idx = pos.to_index(self.width);
            self.black.clear(idx);
            self.white.clear(idx);
            match player {
                Some(Player::Black) => self.black.set(idx),
                Some(Player::White) => self.white.set(idx),
                None => {}
            }
        }
    }

    pub fn clear(&mut self) {
        self.black = Bitboard::empty();
        self.white = Bitboard::empty();
    }

    #[inline]
    pub(crate) fn black_stones(&self) -> Bitboard {
        self.black
    }

    #[inline]
    pub(crate) fn white_stones(&self) -> Bitboard {
        self.white
    }

    #[inline]
    pub(crate) fn occupied(&self) -> Bitboard {
        self.black | self.white
    }

    #[inline]
    pub(crate) fn empty_squares(&self, board_mask: Bitboard) -> Bitboard {
        board_mask & !(self.black | self.white)
    }

    /// Remove all stones indicated by `bb` from the board.
    #[inline]
    pub(crate) fn remove_stones(&mut self, bb: Bitboard) {
        self.black &= !bb;
        self.white &= !bb;
    }

    /// Restore stones from a captured bitboard for the given player.
    #[inline]
    pub(crate) fn restore_stones(&mut self, bb: Bitboard, player: Player) {
        match player {
            Player::Black => self.black |= bb,
            Player::White => self.white |= bb,
        }
    }

    /// Get stones bitboard for a specific player.
    #[inline]
    pub(crate) fn stones_for(&self, player: Player) -> Bitboard {
        match player {
            Player::Black => self.black,
            Player::White => self.white,
        }
    }

    /// Set a single bit for a player (no clearing — caller must ensure position is empty).
    #[inline]
    pub(crate) fn set_bit(&mut self, idx: usize, player: Player) {
        match player {
            Player::Black => self.black.set(idx),
            Player::White => self.white.set(idx),
        }
    }

    /// Clear a single bit from both bitboards.
    #[inline]
    pub(crate) fn clear_bit(&mut self, idx: usize) {
        self.black.clear(idx);
        self.white.clear(idx);
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::standard()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in (0..self.height as usize).rev() {
            write!(f, "|")?;

            for col in 0..self.width as usize {
                let pos = Position::new(col as u8, row as u8);
                let c = if let Some(player) = self.get_piece(&pos) {
                    player.to_char()
                } else {
                    '.'
                };

                write!(f, "{}", c)?;
                write!(f, "|")?;
            }

            writeln!(f)?;
        }

        // Column numbers
        write!(f, " ")?;
        for col in 0..self.width as usize {
            write!(f, "{} ", col)?;
        }
        writeln!(f)?;

        Ok(())
    }
}
