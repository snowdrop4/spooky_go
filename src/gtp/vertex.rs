use crate::player::Player;
use crate::position::Position;
use crate::r#move::Move;

use super::error::GtpError;

/// Convert a 0-based column index to a GTP column letter (A-T, skipping I).
pub fn col_to_letter(col: u8) -> char {
    if col < 8 {
        (b'A' + col) as char
    } else {
        (b'A' + col + 1) as char
    }
}

/// Convert a GTP column letter to a 0-based column index. Case-insensitive, skips I.
pub fn letter_to_col(ch: char) -> Result<u8, GtpError> {
    let upper = ch.to_ascii_uppercase();
    if upper == 'I' || !upper.is_ascii_alphabetic() {
        return Err(GtpError::InvalidVertex(ch.to_string()));
    }
    let raw = upper as u8 - b'A';
    if upper > 'I' {
        Ok(raw - 1)
    } else {
        Ok(raw)
    }
}

/// Convert a Position to a GTP vertex string (e.g. "C4").
pub fn position_to_vertex(pos: &Position, _height: u8) -> String {
    let letter = col_to_letter(pos.col);
    let number = pos.row + 1;
    format!("{}{}", letter, number)
}

/// Parse a GTP vertex string (e.g. "C4") into a Position.
pub fn vertex_to_position(s: &str, _height: u8) -> Result<Position, GtpError> {
    let s = s.trim();
    if s.len() < 2 {
        return Err(GtpError::InvalidVertex(s.to_string()));
    }

    let mut chars = s.chars();
    let letter = chars
        .next()
        .ok_or_else(|| GtpError::InvalidVertex(s.to_string()))?;
    let col = letter_to_col(letter)?;

    let row_str: String = chars.collect();
    let row_num: u8 = row_str
        .parse()
        .map_err(|_| GtpError::InvalidVertex(s.to_string()))?;

    if row_num == 0 {
        return Err(GtpError::InvalidVertex(s.to_string()));
    }

    Ok(Position::new(col, row_num - 1))
}

/// Convert a Move to GTP move string ("C4" or "pass").
pub fn move_to_gtp(m: &Move, height: u8) -> String {
    match m {
        Move::Pass => "pass".to_string(),
        Move::Place { col, row } => {
            let pos = Position::new(*col, *row);
            position_to_vertex(&pos, height)
        }
    }
}

/// Parse a GTP move string into a Move. Handles "pass" and vertex strings.
/// Does NOT handle "resign" — use `gtp_to_move_or_resign` for genmove responses.
pub fn gtp_to_move(s: &str, height: u8) -> Result<Move, GtpError> {
    let lower = s.trim().to_lowercase();
    if lower == "pass" {
        return Ok(Move::pass());
    }
    let pos = vertex_to_position(s, height)?;
    Ok(Move::place(pos.col, pos.row))
}

/// Convert a Player to GTP color string.
pub fn player_to_gtp(p: Player) -> &'static str {
    match p {
        Player::Black => "black",
        Player::White => "white",
    }
}

/// Parse a GTP color string into a Player.
pub fn gtp_to_player(s: &str) -> Result<Player, GtpError> {
    match s.trim().to_lowercase().as_str() {
        "black" | "b" => Ok(Player::Black),
        "white" | "w" => Ok(Player::White),
        _ => Err(GtpError::InvalidColor(s.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_col_to_letter_roundtrip() {
        for col in 0..25u8 {
            let letter = col_to_letter(col);
            assert_ne!(letter, 'I');
            let back = letter_to_col(letter).expect("should parse");
            assert_eq!(back, col);
        }
    }

    #[test]
    fn test_col_to_letter_specific() {
        assert_eq!(col_to_letter(0), 'A');
        assert_eq!(col_to_letter(7), 'H');
        assert_eq!(col_to_letter(8), 'J'); // skips I
        assert_eq!(col_to_letter(18), 'T');
    }

    #[test]
    fn test_letter_to_col_case_insensitive() {
        assert_eq!(letter_to_col('a').expect("ok"), 0);
        assert_eq!(letter_to_col('A').expect("ok"), 0);
        assert_eq!(letter_to_col('j').expect("ok"), 8);
        assert_eq!(letter_to_col('J').expect("ok"), 8);
    }

    #[test]
    fn test_letter_i_rejected() {
        assert!(letter_to_col('I').is_err());
        assert!(letter_to_col('i').is_err());
    }

    #[test]
    fn test_vertex_roundtrip() {
        let pos = Position::new(2, 3); // C4
        let vertex = position_to_vertex(&pos, 19);
        assert_eq!(vertex, "C4");
        let back = vertex_to_position(&vertex, 19).expect("should parse");
        assert_eq!(back, pos);
    }

    #[test]
    fn test_vertex_col8() {
        // col 8 should be J (not I)
        let pos = Position::new(8, 0);
        let vertex = position_to_vertex(&pos, 19);
        assert_eq!(vertex, "J1");
        let back = vertex_to_position("J1", 19).expect("should parse");
        assert_eq!(back, pos);
    }

    #[test]
    fn test_move_pass() {
        let m = Move::pass();
        assert_eq!(move_to_gtp(&m, 19), "pass");
        let back = gtp_to_move("pass", 19).expect("ok");
        assert_eq!(back, Move::pass());
    }

    #[test]
    fn test_move_place_roundtrip() {
        let m = Move::place(3, 3); // D4
        let gtp = move_to_gtp(&m, 19);
        assert_eq!(gtp, "D4");
        let back = gtp_to_move(&gtp, 19).expect("ok");
        assert_eq!(back, m);
    }

    #[test]
    fn test_player_roundtrip() {
        assert_eq!(player_to_gtp(Player::Black), "black");
        assert_eq!(player_to_gtp(Player::White), "white");
        assert_eq!(gtp_to_player("black").expect("ok"), Player::Black);
        assert_eq!(gtp_to_player("WHITE").expect("ok"), Player::White);
        assert_eq!(gtp_to_player("b").expect("ok"), Player::Black);
        assert_eq!(gtp_to_player("W").expect("ok"), Player::White);
    }

    #[test]
    fn test_invalid_vertex() {
        assert!(vertex_to_position("", 19).is_err());
        assert!(vertex_to_position("I1", 19).is_err());
        assert!(vertex_to_position("A0", 19).is_err());
        assert!(vertex_to_position("1A", 19).is_err());
    }
}
