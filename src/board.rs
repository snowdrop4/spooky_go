use std::fmt;

use crate::player::Player;
use crate::position::Position;

pub const STANDARD_COLS: usize = 19;
pub const STANDARD_ROWS: usize = 19;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Board {
    squares: Vec<Option<Player>>,
    width: usize,
    height: usize,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        Board {
            squares: vec![None; width * height],
            width,
            height,
        }
    }

    pub fn standard() -> Self {
        Self::new(STANDARD_COLS, STANDARD_ROWS)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn index(&self, col: usize, row: usize) -> usize {
        row * self.width + col
    }

    pub fn get_piece(&self, pos: &Position) -> Option<Player> {
        if pos.is_valid(self.width, self.height) {
            self.squares[pos.to_index(self.width)]
        } else {
            None
        }
    }

    pub fn set_piece(&mut self, pos: &Position, player: Option<Player>) {
        if pos.is_valid(self.width, self.height) {
            self.squares[pos.to_index(self.width)] = player;
        }
    }

    pub fn clear(&mut self) {
        self.squares = vec![None; self.width * self.height];
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::standard()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in (0..self.height).rev() {
            write!(f, "|")?;

            for col in 0..self.width {
                let index = self.index(col, row);

                let c = if let Some(player) = self.squares[index] {
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
        for col in 0..self.width {
            write!(f, "{} ", col)?;
        }
        writeln!(f)?;

        Ok(())
    }
}
