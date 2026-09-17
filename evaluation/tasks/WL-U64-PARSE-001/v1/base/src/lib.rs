//! Development workload: replace the permissive parser with the task contract.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseError {
    Empty,
    InvalidCharacter,
    LeadingZero,
    Overflow,
}

/// Parse decimal text.
///
/// # Errors
/// Returns an error for empty input or an unsuccessful integer conversion.
pub fn parse_u64(input: &str) -> Result<u64, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    input.parse().map_err(|_| ParseError::Overflow)
}
