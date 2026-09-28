//! Validation of owner-provided fields. Error messages are shown to the owner
//! as they are, so they name the field and the rule.

/// A field value the daemon refuses.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct ValidationError(pub String);

pub const NAME_MAX_CHARS: usize = 64;
pub const ROLE_MAX_CHARS: usize = 200;
pub const INSTRUCTIONS_MAX_CHARS: usize = 32_000;
/// Well under the ~1 million characters Claude Code accepts per message.
pub const MESSAGE_MAX_CHARS: usize = 100_000;
/// Files per message (spec 9.5).
pub const ATTACHMENTS_MAX: usize = 10;

/// Trims a crew or bot name and checks it is a non-empty single line.
pub fn name(field: &str, value: &str) -> Result<String, ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ValidationError(format!("{field} must not be empty")));
    }
    single_line(field, value, NAME_MAX_CHARS)
}

/// Trims a bot role, a single line that may be empty.
pub fn role(value: &str) -> Result<String, ValidationError> {
    single_line("role", value.trim(), ROLE_MAX_CHARS)
}

/// Checks bot instructions, free multi-line text kept as written.
pub fn instructions(value: &str) -> Result<String, ValidationError> {
    if value.chars().count() > INSTRUCTIONS_MAX_CHARS {
        return Err(ValidationError(format!(
            "instructions must be at most {INSTRUCTIONS_MAX_CHARS} characters"
        )));
    }
    Ok(value.trim_end().to_owned())
}

/// Trims a message body or task result, free multi-line text that must not
/// be empty.
pub fn message(field: &str, value: &str) -> Result<String, ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ValidationError(format!("{field} must not be empty")));
    }
    if value.chars().count() > MESSAGE_MAX_CHARS {
        return Err(ValidationError(format!(
            "{field} must be at most {MESSAGE_MAX_CHARS} characters"
        )));
    }
    Ok(value.to_owned())
}

fn single_line(field: &str, value: &str, max: usize) -> Result<String, ValidationError> {
    if value.chars().count() > max {
        return Err(ValidationError(format!(
            "{field} must be at most {max} characters"
        )));
    }
    if value.chars().any(char::is_control) {
        return Err(ValidationError(format!(
            "{field} must be a single line of text"
        )));
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_trimmed_and_required() {
        assert_eq!(name("name", "  Revisor  "), Ok("Revisor".to_owned()));
        assert!(name("name", "   ").is_err());
        assert!(name("name", "two\nlines").is_err());
        assert!(name("name", &"x".repeat(NAME_MAX_CHARS + 1)).is_err());
        assert!(name("name", &"é".repeat(NAME_MAX_CHARS)).is_ok());
    }

    #[test]
    fn role_may_be_empty_but_single_line() {
        assert_eq!(role(""), Ok(String::new()));
        assert!(role("reviews\tcode").is_err());
    }

    #[test]
    fn instructions_keep_newlines_and_have_a_limit() {
        assert_eq!(
            instructions("line 1\nline 2\n\n"),
            Ok("line 1\nline 2".to_owned())
        );
        assert!(instructions(&"x".repeat(INSTRUCTIONS_MAX_CHARS + 1)).is_err());
    }

    #[test]
    fn messages_are_trimmed_required_and_limited() {
        assert_eq!(
            message(
                "body",
                "
 hi
there 
"
            ),
            Ok("hi
there"
                .to_owned())
        );
        assert!(
            message(
                "body", " 
 "
            )
            .is_err()
        );
        assert!(message("body", &"x".repeat(MESSAGE_MAX_CHARS + 1)).is_err());
    }
}
