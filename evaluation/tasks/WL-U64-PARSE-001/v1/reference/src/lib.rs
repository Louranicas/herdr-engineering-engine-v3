//! Reference solution for the frozen development workload.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseError {
    Empty,
    InvalidCharacter,
    LeadingZero,
    Overflow,
}

/// Parse canonical ASCII decimal text without normalization.
///
/// # Errors
/// Reports empty input, any invalid character, leading zero, then overflow.
pub fn parse_u64(input: &str) -> Result<u64, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    if !input.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ParseError::InvalidCharacter);
    }
    if input.len() > 1 && input.starts_with('0') {
        return Err(ParseError::LeadingZero);
    }
    input.parse().map_err(|_| ParseError::Overflow)
}
