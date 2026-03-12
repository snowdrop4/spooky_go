use super::error::GtpError;

/// A parsed GTP response.
#[derive(Debug, Clone)]
pub struct GtpResponse {
    pub id: Option<u32>,
    pub success: bool,
    pub content: String,
}

/// Format a GTP command with an ID.
pub fn format_command(id: u32, command: &str, args: &[&str]) -> String {
    if args.is_empty() {
        format!("{} {}\n", id, command)
    } else {
        format!("{} {} {}\n", id, command, args.join(" "))
    }
}

/// Parse a raw GTP response string into a GtpResponse.
pub fn parse_response(raw: &str) -> Result<GtpResponse, GtpError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(GtpError::Protocol("empty response".to_string()));
    }

    let (success, rest) = if let Some(rest) = trimmed.strip_prefix('=') {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix('?') {
        (false, rest)
    } else {
        return Err(GtpError::Protocol(format!(
            "response must start with '=' or '?': {}",
            trimmed
        )));
    };

    let rest = rest.trim_start();

    // Try to parse an ID at the start
    let (id, content) = if let Some(space_idx) = rest.find(|c: char| !c.is_ascii_digit()) {
        if space_idx > 0 {
            let id_str = &rest[..space_idx];
            if let Ok(id) = id_str.parse::<u32>() {
                (Some(id), rest[space_idx..].trim().to_string())
            } else {
                (None, rest.to_string())
            }
        } else {
            (None, rest.to_string())
        }
    } else if rest.chars().all(|c| c.is_ascii_digit()) && !rest.is_empty() {
        // Entire rest is digits — it's just an ID with no content
        let id = rest.parse::<u32>().ok();
        (id, String::new())
    } else {
        (None, rest.to_string())
    };

    Ok(GtpResponse {
        id,
        success,
        content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_command_no_args() {
        assert_eq!(format_command(1, "name", &[]), "1 name\n");
    }

    #[test]
    fn test_format_command_with_args() {
        assert_eq!(
            format_command(2, "play", &["black", "D4"]),
            "2 play black D4\n"
        );
    }

    #[test]
    fn test_parse_success_with_id() {
        let resp = parse_response("=1 GnuGo").expect("should parse");
        assert!(resp.success);
        assert_eq!(resp.id, Some(1));
        assert_eq!(resp.content, "GnuGo");
    }

    #[test]
    fn test_parse_success_no_id() {
        let resp = parse_response("= GnuGo").expect("should parse");
        assert!(resp.success);
        assert_eq!(resp.id, None);
        assert_eq!(resp.content, "GnuGo");
    }

    #[test]
    fn test_parse_error_with_id() {
        let resp = parse_response("?3 illegal move").expect("should parse");
        assert!(!resp.success);
        assert_eq!(resp.id, Some(3));
        assert_eq!(resp.content, "illegal move");
    }

    #[test]
    fn test_parse_success_empty_content() {
        let resp = parse_response("=1").expect("should parse");
        assert!(resp.success);
        assert_eq!(resp.id, Some(1));
        assert_eq!(resp.content, "");
    }

    #[test]
    fn test_parse_empty_fails() {
        assert!(parse_response("").is_err());
    }

    #[test]
    fn test_parse_no_prefix_fails() {
        assert!(parse_response("hello").is_err());
    }
}
