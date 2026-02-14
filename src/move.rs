use crate::position::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Move {
    Place { col: u8, row: u8 },
    Pass,
}

impl Move {
    pub fn place(col: u8, row: u8) -> Self {
        Move::Place { col, row }
    }

    pub fn pass() -> Self {
        Move::Pass
    }

    pub fn is_pass(&self) -> bool {
        matches!(self, Move::Pass)
    }

    pub fn position(&self) -> Option<Position> {
        match self {
            Move::Place { col, row } => Some(Position::new(*col, *row)),
            Move::Pass => None,
        }
    }

    pub fn col(&self) -> Option<u8> {
        match self {
            Move::Place { col, .. } => Some(*col),
            Move::Pass => None,
        }
    }

    pub fn row(&self) -> Option<u8> {
        match self {
            Move::Place { row, .. } => Some(*row),
            Move::Pass => None,
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Place { col, row } => write!(f, "Place({}, {})", col, row),
            Move::Pass => write!(f, "Pass"),
        }
    }
}
