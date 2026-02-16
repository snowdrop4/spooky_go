use crate::bitboard::nw_for_board;
use crate::board::{STANDARD_COLS, STANDARD_ROWS};
use crate::game::Game;
use crate::r#move::Move;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl<const NW: usize> Serialize for Game<NW> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let moves: Vec<String> = self
            .move_history()
            .iter()
            .map(|m| match m {
                Move::Place { col, row } => format!("{},{}", col, row),
                Move::Pass => "pass".to_string(),
            })
            .collect();
        let moves_str = moves.join(";");

        // Include board dimensions: "WxH:moves"
        let full = format!("{}x{}:{}", self.width(), self.height(), moves_str);
        serializer.serialize_str(&full)
    }
}

impl<'de> Deserialize<'de> for Game<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Support both "WxH:moves" format and legacy "moves" format
        let (width, height, moves_str) = if let Some((dims, rest)) = s.split_once(':') {
            let (w, h) = dims
                .split_once('x')
                .ok_or_else(|| serde::de::Error::custom("Invalid dimensions format"))?;
            let width: u8 = w
                .parse()
                .map_err(|e| serde::de::Error::custom(format!("Invalid width: {}", e)))?;
            let height: u8 = h
                .parse()
                .map_err(|e| serde::de::Error::custom(format!("Invalid height: {}", e)))?;

            let expected_nw = nw_for_board(width, height);
            let actual_nw = nw_for_board(STANDARD_COLS, STANDARD_ROWS);
            if expected_nw != actual_nw {
                return Err(serde::de::Error::custom(format!(
                    "Board {}x{} requires NW={}, but deserializing into Game with NW={}",
                    width, height, expected_nw, actual_nw
                )));
            }

            (width, height, rest)
        } else {
            // Legacy format: assume standard board
            (STANDARD_COLS, STANDARD_ROWS, s.as_str())
        };

        let mut game = Game::with_options(width, height, crate::game::DEFAULT_KOMI, 0, u16::MAX, true);

        if moves_str.is_empty() {
            return Ok(game);
        }

        for move_str in moves_str.split(';') {
            let move_str = move_str.trim();

            let mv = if move_str == "pass" {
                Move::pass()
            } else {
                let parts: Vec<&str> = move_str.split(',').collect();
                if parts.len() != 2 {
                    return Err(serde::de::Error::custom(format!(
                        "Invalid move format: {}",
                        move_str
                    )));
                }

                let col: u8 = parts[0]
                    .trim()
                    .parse()
                    .map_err(|e| serde::de::Error::custom(format!("Invalid column: {}", e)))?;
                let row: u8 = parts[1]
                    .trim()
                    .parse()
                    .map_err(|e| serde::de::Error::custom(format!("Invalid row: {}", e)))?;

                Move::place(col, row)
            };

            if !game.make_move(&mv) {
                return Err(serde::de::Error::custom(format!("Invalid move: {:?}", mv)));
            }
        }

        Ok(game)
    }
}

impl Serialize for Move {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Move::Place { col, row } => serializer.serialize_str(&format!("{},{}", col, row)),
            Move::Pass => serializer.serialize_str("pass"),
        }
    }
}

impl<'de> Deserialize<'de> for Move {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s == "pass" {
            return Ok(Move::pass());
        }

        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "Invalid move format: {}",
                s
            )));
        }

        let col: u8 = parts[0]
            .trim()
            .parse()
            .map_err(|e| serde::de::Error::custom(format!("Invalid column: {}", e)))?;
        let row: u8 = parts[1]
            .trim()
            .parse()
            .map_err(|e| serde::de::Error::custom(format!("Invalid row: {}", e)))?;

        Ok(Move::place(col, row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type StandardGame = Game<{ nw_for_board(STANDARD_COLS, STANDARD_ROWS) }>;

    #[test]
    fn test_game_serde_empty() {
        let game = StandardGame::standard();

        let json = serde_json::to_string(&game).unwrap();
        assert_eq!(json, r#""19x19:""#);

        let game2: StandardGame = serde_json::from_str(&json).unwrap();
        assert_eq!(game2.move_history().len(), 0);
        assert!(!game2.is_over());
    }

    #[test]
    fn test_game_serde_with_moves() {
        let mut game = Game::<{ nw_for_board(9, 9) }>::new(9, 9);

        game.make_move(&Move::place(3, 3));
        game.make_move(&Move::place(4, 4));
        game.make_move(&Move::place(5, 5));

        let json = serde_json::to_string(&game).unwrap();
        assert_eq!(json, r#""9x9:3,3;4,4;5,5""#);
    }

    #[test]
    fn test_game_serde_with_pass() {
        let mut game = StandardGame::with_options(19, 19, crate::game::DEFAULT_KOMI, 0, 1000, true);

        game.make_move(&Move::place(0, 0));
        game.make_move(&Move::pass());
        game.make_move(&Move::place(1, 1));

        let json = serde_json::to_string(&game).unwrap();
        assert_eq!(json, r#""19x19:0,0;pass;1,1""#);

        let game2: StandardGame = serde_json::from_str(&json).unwrap();
        assert_eq!(game2.move_history().len(), 3);
        assert!(game2.move_history()[1].is_pass());
    }

    #[test]
    fn test_move_serde_place() {
        let move_ = Move::place(3, 4);

        let json = serde_json::to_string(&move_).unwrap();
        assert_eq!(json, r#""3,4""#);

        let move2: Move = serde_json::from_str(&json).unwrap();
        assert_eq!(move2, move_);
    }

    #[test]
    fn test_move_serde_pass() {
        let move_ = Move::pass();

        let json = serde_json::to_string(&move_).unwrap();
        assert_eq!(json, r#""pass""#);

        let move2: Move = serde_json::from_str(&json).unwrap();
        assert!(move2.is_pass());
    }

    #[test]
    fn test_game_roundtrip() {
        let mut game = StandardGame::with_options(19, 19, crate::game::DEFAULT_KOMI, 0, 1000, true);

        game.make_move(&Move::place(4, 4));
        game.make_move(&Move::place(3, 3));
        game.make_move(&Move::place(5, 5));
        game.make_move(&Move::pass());
        game.make_move(&Move::place(2, 2));

        let json = serde_json::to_string(&game).unwrap();

        let game2: StandardGame = serde_json::from_str(&json).unwrap();

        assert_eq!(game.move_history().len(), game2.move_history().len());
        for (m1, m2) in game.move_history().iter().zip(game2.move_history().iter()) {
            assert_eq!(m1, m2);
        }
    }

    #[test]
    fn test_bincode_game() {
        let mut game = StandardGame::new(19, 19);
        game.make_move(&Move::place(3, 3));
        game.make_move(&Move::place(4, 4));

        let encoded = bincode::serialize(&game).unwrap();

        let game2: StandardGame = bincode::deserialize(&encoded).unwrap();

        assert_eq!(game.move_history().len(), game2.move_history().len());
    }

    #[test]
    fn test_bincode_move() {
        let move_ = Move::place(5, 6);

        let encoded = bincode::serialize(&move_).unwrap();

        let move2: Move = bincode::deserialize(&encoded).unwrap();

        assert_eq!(move2, move_);
    }

    #[test]
    fn test_bincode_pass() {
        let move_ = Move::pass();

        let encoded = bincode::serialize(&move_).unwrap();

        let move2: Move = bincode::deserialize(&encoded).unwrap();

        assert!(move2.is_pass());
    }

    #[test]
    fn test_legacy_format_deserialize() {
        // Legacy format without dimensions should deserialize as 19x19
        let json = r#""0,0;1,1;2,2""#;
        let game: StandardGame = serde_json::from_str(json).unwrap();
        assert_eq!(game.move_history().len(), 3);
        assert_eq!(game.width(), 19);
        assert_eq!(game.height(), 19);
    }
}
