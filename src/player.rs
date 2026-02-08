#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Player {
    Black = 1,
    White = -1,
}

impl Player {
    pub fn opposite(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Player::Black => 'B',
            Player::White => 'W',
        }
    }

    pub fn from_char(c: char) -> Option<Player> {
        match c {
            'B' | 'b' => Some(Player::Black),
            'W' | 'w' => Some(Player::White),
            _ => None,
        }
    }

    pub fn from_int(i: i8) -> Option<Player> {
        match i {
            1 => Some(Player::Black),
            -1 => Some(Player::White),
            _ => None,
        }
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let player_str = match self {
            Player::Black => "Black",
            Player::White => "White",
        };
        write!(f, "{}", player_str)
    }
}
