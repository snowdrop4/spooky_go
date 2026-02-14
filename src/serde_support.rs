use crate::game::Game;
use crate::r#move::Move;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Game {
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

        serializer.serialize_str(&moves_str)
    }
}

impl<'de> Deserialize<'de> for Game {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let moves_str = String::deserialize(deserializer)?;

        let mut game = Game::standard();

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

    #[test]
    fn test_game_serde_empty() {
        let game = Game::standard();

        let json = serde_json::to_string(&game).unwrap();
        assert_eq!(json, r#""""#);

        let game2: Game = serde_json::from_str(&json).unwrap();
        assert_eq!(game2.move_history().len(), 0);
        assert!(!game2.is_over());
    }

    #[test]
    fn test_game_serde_with_moves() {
        let mut game = Game::new(9, 9);

        game.make_move(&Move::place(3, 3));
        game.make_move(&Move::place(4, 4));
        game.make_move(&Move::place(5, 5));

        let json = serde_json::to_string(&game).unwrap();
        assert_eq!(json, r#""3,3;4,4;5,5""#);

        let game2: Game = serde_json::from_str(&json).unwrap();
        assert_eq!(game2.move_history().len(), 3);
    }

    #[test]
    fn test_game_serde_with_pass() {
        let mut game = Game::new(9, 9);

        game.make_move(&Move::place(0, 0));
        game.make_move(&Move::pass());
        game.make_move(&Move::place(1, 1));

        let json = serde_json::to_string(&game).unwrap();
        assert_eq!(json, r#""0,0;pass;1,1""#);

        let game2: Game = serde_json::from_str(&json).unwrap();
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
        let mut game = Game::new(9, 9);

        game.make_move(&Move::place(4, 4));
        game.make_move(&Move::place(3, 3));
        game.make_move(&Move::place(5, 5));
        game.make_move(&Move::pass());
        game.make_move(&Move::place(2, 2));

        let json = serde_json::to_string(&game).unwrap();

        let game2: Game = serde_json::from_str(&json).unwrap();

        assert_eq!(game.move_history().len(), game2.move_history().len());
        for (m1, m2) in game.move_history().iter().zip(game2.move_history().iter()) {
            assert_eq!(m1, m2);
        }
    }

    #[test]
    fn test_bincode_game() {
        let mut game = Game::new(9, 9);
        game.make_move(&Move::place(3, 3));
        game.make_move(&Move::place(4, 4));

        let encoded = bincode::serialize(&game).unwrap();

        let game2: Game = bincode::deserialize(&encoded).unwrap();

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
}
