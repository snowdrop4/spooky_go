use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use crate::player::Player;
use crate::r#move::Move;

use super::error::{GenmoveResult, GtpError};
use super::protocol::{format_command, parse_response};
use super::vertex::{move_to_gtp, player_to_gtp};

/// A raw GTP client that communicates with an engine subprocess.
pub struct GtpClient {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    next_id: u32,
}

impl GtpClient {
    /// Spawn a new GTP engine process.
    pub fn new(program: &str, args: &[&str]) -> Result<Self, GtpError> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| GtpError::Protocol("failed to open stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| GtpError::Protocol("failed to open stdout".to_string()))?;

        Ok(GtpClient {
            child,
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    /// Send a raw GTP command and return the response content.
    pub fn send_command(&mut self, cmd: &str, args: &[&str]) -> Result<String, GtpError> {
        let id = self.next_id;
        self.next_id += 1;

        let formatted = format_command(id, cmd, args);
        self.stdin.write_all(formatted.as_bytes())?;
        self.stdin.flush()?;

        // Read response lines until we get an empty line
        let mut response_text = String::new();
        loop {
            let mut line = String::new();
            let bytes = self.stdout.read_line(&mut line)?;
            if bytes == 0 {
                return Err(GtpError::ProcessNotRunning);
            }
            if line.trim().is_empty() {
                if !response_text.is_empty() {
                    break;
                }
                // Skip leading empty lines
                continue;
            }
            if !response_text.is_empty() {
                response_text.push('\n');
            }
            response_text.push_str(line.trim_end());
        }

        let resp = parse_response(&response_text)?;
        if resp.success {
            Ok(resp.content)
        } else {
            Err(GtpError::EngineError(resp.content))
        }
    }

    // -------------------------------------------------------------------------
    // Typed GTP command wrappers
    // -------------------------------------------------------------------------

    pub fn protocol_version(&mut self) -> Result<String, GtpError> {
        self.send_command("protocol_version", &[])
    }

    pub fn name(&mut self) -> Result<String, GtpError> {
        self.send_command("name", &[])
    }

    pub fn version(&mut self) -> Result<String, GtpError> {
        self.send_command("version", &[])
    }

    pub fn known_command(&mut self, cmd: &str) -> Result<bool, GtpError> {
        let resp = self.send_command("known_command", &[cmd])?;
        Ok(resp.trim().eq_ignore_ascii_case("true"))
    }

    pub fn list_commands(&mut self) -> Result<Vec<String>, GtpError> {
        let resp = self.send_command("list_commands", &[])?;
        Ok(resp.lines().map(|l| l.trim().to_string()).collect())
    }

    pub fn boardsize(&mut self, size: u8) -> Result<(), GtpError> {
        let s = size.to_string();
        self.send_command("boardsize", &[&s])?;
        Ok(())
    }

    pub fn clear_board(&mut self) -> Result<(), GtpError> {
        self.send_command("clear_board", &[])?;
        Ok(())
    }

    pub fn komi(&mut self, komi: f32) -> Result<(), GtpError> {
        let s = format!("{}", komi);
        self.send_command("komi", &[&s])?;
        Ok(())
    }

    pub fn play(&mut self, player: Player, m: &Move, board_height: u8) -> Result<(), GtpError> {
        let color = player_to_gtp(player);
        let vertex = move_to_gtp(m, board_height);
        self.send_command("play", &[color, &vertex])?;
        Ok(())
    }

    pub fn genmove(&mut self, player: Player, board_height: u8) -> Result<GenmoveResult, GtpError> {
        let color = player_to_gtp(player);
        let resp = self.send_command("genmove", &[color])?;
        let lower = resp.trim().to_lowercase();
        if lower == "resign" {
            Ok(GenmoveResult::Resign)
        } else if lower == "pass" {
            Ok(GenmoveResult::Move(Move::pass()))
        } else {
            let pos = super::vertex::vertex_to_position(&resp, board_height)?;
            Ok(GenmoveResult::Move(Move::place(pos.col, pos.row)))
        }
    }

    pub fn undo(&mut self) -> Result<(), GtpError> {
        self.send_command("undo", &[])?;
        Ok(())
    }

    pub fn showboard(&mut self) -> Result<String, GtpError> {
        self.send_command("showboard", &[])
    }

    pub fn final_score(&mut self) -> Result<String, GtpError> {
        self.send_command("final_score", &[])
    }

    pub fn quit(&mut self) -> Result<(), GtpError> {
        // Ignore errors from quit — the engine might already be gone
        let _ = self.send_command("quit", &[]);
        Ok(())
    }
}

impl Drop for GtpClient {
    fn drop(&mut self) {
        let _ = self.send_command("quit", &[]);
        let _ = self.child.wait();
    }
}
