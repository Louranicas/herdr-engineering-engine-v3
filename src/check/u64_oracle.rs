//! Exact frozen development-oracle decoding and output agreement only.
//!
//! Expectations come from ORACLE-U64-001/v1, independently frozen using literal
//! boundaries and arbitrary-precision grammar arithmetic. Candidate/reference
//! implementation code is never used to derive them. Matching output does not
//! prove execution, protected custody, general correctness or module admission.

use crate::contracts::parse_u64_decimal;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const VECTOR_COUNT: usize = 335;
pub const ORACLE_SHA256: &str = "9354a52b2539e562ec8c3b957dbe693d75411e29c88d19a446b7cc488862b427";
/// The frozen oracle's own declared identity; the one home for this literal.
pub const ORACLE_ID: &str = "ORACLE-U64-001/v1";
const MAX_ORACLE: usize = 128 * 1024;
const MAX_INPUT: usize = 1025;
const MAX_PUBLIC: usize = 32 * 1024;
const MAX_RECORD: usize = 64;
const MAX_OUTPUT: usize = VECTOR_COUNT * MAX_RECORD;
const HEX: &[u8; 16] = b"0123456789abcdef";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleError {
    Bound,
    Digest,
    Malformed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputError {
    Bound,
    Malformed,
    Incomplete,
    WrongOrder,
    ExtraRecords,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum ParseError {
    Empty,
    InvalidCharacter,
    LeadingZero,
    Overflow,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Answer {
    Value(u64),
    Error(ParseError),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorMatch {
    pub index: usize,
    pub id: String,
    pub expected: Answer,
    pub actual: Answer,
    pub matched: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub vectors: Vec<VectorMatch>,
    pub matched: usize,
    pub failed: usize,
}
#[derive(Debug)]
pub struct FrozenOracle {
    cases: Vec<Case>,
}
#[derive(Debug)]
struct Case {
    id: String,
    input: String,
    expected: Answer,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOracle {
    id: String,
    case_count_is_not_module_credit: bool,
    cases: Vec<RawCase>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCase {
    id: String,
    input: String,
    expected: RawAnswer,
    #[serde(rename = "basis")]
    _basis: Basis,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Basis {
    FixedBoundary,
    GrammarArbitraryPrecision,
}
#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum RawAnswer {
    Value { value: String },
    Error { error: ParseError },
}

impl FrozenOracle {
    /// Decode only the exact immutable protected development-oracle object.
    ///
    /// # Errors
    /// Byte bounds precede digest validation. Changed bytes fail Digest before
    /// interpretation; invalid closed records/values fail Malformed.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OracleError> {
        if bytes.len() > MAX_ORACLE {
            return Err(OracleError::Bound);
        }
        let hash = Sha256::digest(bytes);
        let expected = ORACLE_SHA256.as_bytes();
        if hash.iter().enumerate().any(|(i, byte)| {
            HEX[usize::from(byte >> 4)] != expected[i * 2]
                || HEX[usize::from(byte & 15)] != expected[i * 2 + 1]
        }) {
            return Err(OracleError::Digest);
        }
        let raw: RawOracle = serde_json::from_slice(bytes).map_err(|_| OracleError::Malformed)?;
        if raw.id != ORACLE_ID
            || !raw.case_count_is_not_module_credit
            || raw.cases.len() != VECTOR_COUNT
        {
            return Err(OracleError::Malformed);
        }
        let mut names = BTreeSet::new();
        let mut public_length = 0;
        let mut cases = Vec::with_capacity(VECTOR_COUNT);
        for row in raw.cases {
            if row.id.is_empty()
                || row.id.len() > 64
                || !row
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                || !names.insert(row.id.clone())
            {
                return Err(OracleError::Malformed);
            }
            if row.input.len() > MAX_INPUT {
                return Err(OracleError::Bound);
            }
            public_length += row.input.len() * 2 + 1;
            if public_length > MAX_PUBLIC {
                return Err(OracleError::Bound);
            }
            let expected = match row.expected {
                RawAnswer::Value { value } => {
                    Answer::Value(parse_u64_decimal(&value).map_err(|_| OracleError::Malformed)?)
                }
                RawAnswer::Error { error } => Answer::Error(error),
            };
            cases.push(Case {
                id: row.id,
                input: row.input,
                expected,
            });
        }
        Ok(Self { cases })
    }

    #[must_use]
    pub fn case_count(&self) -> usize {
        self.cases.len()
    }

    /// Public ordered UTF8 input bytes encoded as lower-case hex plus LF. No
    /// expected answer, protected case ID or basis is included.
    #[must_use]
    pub fn public_inputs(&self) -> Vec<u8> {
        public_inputs(&self.cases)
    }

    /// Compare complete ordered raw output against protected frozen expectations.
    /// Even all matched vectors do not establish stream/producer custody.
    ///
    /// # Errors
    /// Refuses overbounds, malformed framing/tokens, incomplete or extra records,
    /// and nonsequential indices. No partial aggregate is returned on protocol error.
    pub fn evaluate(&self, stdout: &[u8]) -> Result<Evaluation, OutputError> {
        evaluate(&self.cases, stdout)
    }
}

fn public_inputs(cases: &[Case]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for case in cases {
        for byte in case.input.bytes() {
            bytes.push(HEX[usize::from(byte >> 4)]);
            bytes.push(HEX[usize::from(byte & 15)]);
        }
        bytes.push(b'\n');
    }
    bytes
}

fn evaluate(cases: &[Case], stdout: &[u8]) -> Result<Evaluation, OutputError> {
    if stdout.len() > MAX_OUTPUT {
        return Err(OutputError::Bound);
    }
    if stdout.is_empty() || stdout.last() != Some(&b'\n') {
        return Err(OutputError::Incomplete);
    }
    let text = std::str::from_utf8(stdout).map_err(|_| OutputError::Malformed)?;
    let mut vectors = Vec::with_capacity(cases.len());
    let mut matched = 0;
    for (index, line) in text[..text.len() - 1].split('\n').enumerate() {
        let case = cases.get(index).ok_or(OutputError::ExtraRecords)?;
        if line.len() >= MAX_RECORD {
            return Err(OutputError::Bound);
        }
        let actual = parse_line(line, index)?;
        let same = case.expected == actual;
        matched += usize::from(same);
        vectors.push(VectorMatch {
            index,
            id: case.id.clone(),
            expected: case.expected,
            actual,
            matched: same,
        });
    }
    if vectors.len() != cases.len() {
        return Err(OutputError::Incomplete);
    }
    Ok(Evaluation {
        failed: vectors.len() - matched,
        vectors,
        matched,
    })
}
fn parse_line(line: &str, index: usize) -> Result<Answer, OutputError> {
    let mut fields = line.split('\t');
    let number = fields.next().ok_or(OutputError::Malformed)?;
    let kind = fields.next().ok_or(OutputError::Malformed)?;
    let value = fields.next().ok_or(OutputError::Malformed)?;
    if fields.next().is_some() {
        return Err(OutputError::Malformed);
    }
    let number = parse_u64_decimal(number).map_err(|_| OutputError::Malformed)?;
    if number != u64::try_from(index).map_err(|_| OutputError::Bound)? {
        return Err(OutputError::WrongOrder);
    }
    match kind {
        "ok" => parse_u64_decimal(value)
            .map(Answer::Value)
            .map_err(|_| OutputError::Malformed),
        "error" => match value {
            "Empty" => Ok(Answer::Error(ParseError::Empty)),
            "InvalidCharacter" => Ok(Answer::Error(ParseError::InvalidCharacter)),
            "LeadingZero" => Ok(Answer::Error(ParseError::LeadingZero)),
            "Overflow" => Ok(Answer::Error(ParseError::Overflow)),
            _ => Err(OutputError::Malformed),
        },
        _ => Err(OutputError::Malformed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;
    const FROZEN: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/evaluation/tasks/WL-U64-PARSE-001/v1/oracle/cases.json"
    ));
    // These literal expectations were frozen before implementation in the private
    // protocol-controls.json. They are development controls, not workload credits.
    const OUTPUT: &[u8] = b"0\terror\tEmpty\n1\tok\t0\n2\tok\t18446744073709551615\n3\terror\tInvalidCharacter\n4\terror\tLeadingZero\n5\terror\tOverflow\n";
    fn literal() -> Vec<Case> {
        [
            ("empty", "", Answer::Error(ParseError::Empty)),
            ("zero", "0", Answer::Value(0)),
            ("maximum", "18446744073709551615", Answer::Value(u64::MAX)),
            ("invalid", "x", Answer::Error(ParseError::InvalidCharacter)),
            ("leading", "01", Answer::Error(ParseError::LeadingZero)),
            (
                "overflow",
                "18446744073709551616",
                Answer::Error(ParseError::Overflow),
            ),
        ]
        .into_iter()
        .map(|(id, input, expected)| Case {
            id: id.into(),
            input: input.into(),
            expected,
        })
        .collect()
    }
    fn changed(old: &str, new: &str) -> Vec<u8> {
        String::from_utf8(OUTPUT.to_vec())
            .unwrap()
            .replacen(old, new, 1)
            .into_bytes()
    }
    #[test]
    fn literal_complete_output_preserves_six_bindings() {
        let result = evaluate(&literal(), OUTPUT).unwrap();
        assert_eq!((result.matched, result.failed), (6, 0));
        assert_eq!(result.vectors[0].id, "empty");
        assert_eq!(result.vectors[2].actual, Answer::Value(u64::MAX));
        assert!(
            result
                .vectors
                .iter()
                .enumerate()
                .all(|(i, v)| v.index == i && v.matched)
        );
    }
    #[test]
    fn public_hex_projection_matches_frozen_literal_without_answers() {
        assert_eq!(public_inputs(&literal()),b"\n30\n3138343436373434303733373039353531363135\n78\n3031\n3138343436373434303733373039353531363136\n");
    }
    #[test]
    fn canonical_wrong_value_is_a_bound_vector_failure() {
        let r = evaluate(&literal(), &changed("1\tok\t0", "1\tok\t1")).unwrap();
        assert_eq!((r.matched, r.failed), (5, 1));
        assert!(!r.vectors[1].matched);
        assert_eq!(r.vectors[1].expected, Answer::Value(0));
        assert_eq!(r.vectors[1].actual, Answer::Value(1));
    }
    #[test]
    fn known_wrong_error_is_a_failure_not_protocol_refusal() {
        let r = evaluate(
            &literal(),
            &changed("0\terror\tEmpty", "0\terror\tOverflow"),
        )
        .unwrap();
        assert_eq!((r.matched, r.failed), (5, 1));
        assert!(!r.vectors[0].matched);
    }
    #[test]
    fn wrong_kind_but_valid_answer_is_failed() {
        let r = evaluate(&literal(), &changed("0\terror\tEmpty", "0\tok\t0")).unwrap();
        assert_eq!((r.matched, r.failed), (5, 1));
    }
    #[test]
    fn empty_stdout_cannot_be_zero_case_success() {
        assert_eq!(evaluate(&literal(), b""), Err(OutputError::Incomplete));
    }
    #[test]
    fn missing_final_lf_is_incomplete() {
        assert_eq!(
            evaluate(&literal(), &OUTPUT[..OUTPUT.len() - 1]),
            Err(OutputError::Incomplete)
        );
    }
    #[test]
    fn fewer_complete_records_is_incomplete() {
        assert_eq!(
            evaluate(&literal(), &changed("5\terror\tOverflow\n", "")),
            Err(OutputError::Incomplete)
        );
    }
    #[test]
    fn unknown_error_or_status_is_malformed() {
        for replacement in [
            "0\terror\tOther",
            "0\tError\tEmpty",
            "0\tpass\tEmpty",
            "0\terror\tempty",
        ] {
            assert_eq!(
                evaluate(&literal(), &changed("0\terror\tEmpty", replacement)),
                Err(OutputError::Malformed)
            );
        }
    }
    #[test]
    fn invalid_success_decimal_is_not_a_value() {
        for value in [
            "00",
            "+1",
            "1.0",
            "1e0",
            "18446744073709551616",
            " 1",
            "1 ",
            "١",
            "",
        ] {
            assert_eq!(
                evaluate(&literal(), &changed("1\tok\t0", &format!("1\tok\t{value}"))),
                Err(OutputError::Malformed)
            );
        }
    }
    #[test]
    fn lexical_index_is_canonical_decimal() {
        for value in ["00", "+0", "-0", "0.0", "", "18446744073709551616"] {
            assert_eq!(
                evaluate(
                    &literal(),
                    &changed("0\terror\tEmpty", &format!("{value}\terror\tEmpty"))
                ),
                Err(OutputError::Malformed)
            );
        }
    }
    #[test]
    fn repeated_skipped_and_reversed_indices_refuse_order() {
        for (old, new) in [
            ("1\tok\t0", "0\tok\t0"),
            ("1\tok\t0", "2\tok\t0"),
            ("0\terror\tEmpty", "1\terror\tEmpty"),
        ] {
            assert_eq!(
                evaluate(&literal(), &changed(old, new)),
                Err(OutputError::WrongOrder)
            );
        }
    }
    #[test]
    fn any_extra_record_refuses_whole_output() {
        for suffix in [b"6\tok\t0\n".as_slice(), b"\n", b"PASS\n"] {
            let mut bytes = OUTPUT.to_vec();
            bytes.extend_from_slice(suffix);
            assert_eq!(evaluate(&literal(), &bytes), Err(OutputError::ExtraRecords));
        }
    }
    #[test]
    fn columns_are_exactly_three_and_nonempty() {
        for value in [
            "0\terror",
            "0\terror\tEmpty\textra",
            "",
            "0\t\tEmpty",
            "0\terror\t",
        ] {
            assert_eq!(
                evaluate(&literal(), &changed("0\terror\tEmpty", value)),
                Err(OutputError::Malformed)
            );
        }
    }
    #[test]
    fn framing_rejects_cr_bom_invalid_utf8_and_blank_records() {
        let mut invalid = OUTPUT.to_vec();
        invalid[0] = 255;
        let mut bom = b"\xef\xbb\xbf".to_vec();
        bom.extend_from_slice(OUTPUT);
        for bytes in [
            invalid,
            bom,
            changed("Empty\n", "Empty\r\n"),
            changed("Empty", "Em\rpty"),
            changed("0\terror\tEmpty", ""),
        ] {
            assert_eq!(evaluate(&literal(), &bytes), Err(OutputError::Malformed));
        }
    }
    #[test]
    fn printed_pass_is_not_a_vector_protocol() {
        assert_eq!(evaluate(&literal(), b"PASS\n"), Err(OutputError::Malformed));
    }
    #[test]
    fn a_single_record_is_bounded_before_token_interpretation() {
        let line = format!("0\terror\t{}\n", "x".repeat(64));
        assert_eq!(
            evaluate(&literal(), line.as_bytes()),
            Err(OutputError::Bound)
        );
    }
    #[test]
    fn aggregate_output_bound_precedes_incomplete_status() {
        assert_eq!(
            evaluate(&literal(), &vec![b'x'; MAX_OUTPUT + 1]),
            Err(OutputError::Bound)
        );
    }
    #[test]
    fn changed_oracle_is_rejected_even_if_json_is_valid() {
        let mut bytes = FROZEN.to_vec();
        bytes.push(b' ');
        assert!(matches!(
            FrozenOracle::from_bytes(&bytes),
            Err(OracleError::Digest)
        ));
        assert!(matches!(
            FrozenOracle::from_bytes(b"{}"),
            Err(OracleError::Digest)
        ));
    }
    #[test]
    fn protected_oracle_byte_bound_precedes_digest() {
        assert!(matches!(
            FrozenOracle::from_bytes(&vec![0; MAX_ORACLE + 1]),
            Err(OracleError::Bound)
        ));
    }
    #[test]
    fn frozen_oracle_has_exact_count_and_public_projection_bounds() {
        let oracle = FrozenOracle::from_bytes(FROZEN).unwrap();
        assert_eq!(oracle.case_count(), 335);
        let public = oracle.public_inputs();
        assert_eq!(public.len(), 13385);
        assert!(public.starts_with(b"\n30\n"));
        assert!(
            public
                .iter()
                .all(|b| b.is_ascii_digit() || matches!(*b, b'a'..=b'f' | b'\n'))
        );
        assert_eq!(std::str::from_utf8(&public).unwrap().lines().count(), 335);
    }
    #[test]
    fn full_frozen_answer_table_decodes_without_case_credit() {
        // Table projection tests plumbing only; it creates no independent oracle.
        let table: serde_json::Value = serde_json::from_slice(FROZEN).unwrap();
        let mut output = String::new();
        for (index, row) in table["cases"].as_array().unwrap().iter().enumerate() {
            if let Some(value) = row["expected"]["value"].as_str() {
                writeln!(output, "{index}\tok\t{value}").unwrap();
            } else {
                writeln!(
                    output,
                    "{index}\terror\t{}",
                    row["expected"]["error"].as_str().unwrap()
                )
                .unwrap();
            }
        }
        let result = FrozenOracle::from_bytes(FROZEN)
            .unwrap()
            .evaluate(output.as_bytes())
            .unwrap();
        assert_eq!(
            (result.matched, result.failed, result.vectors.len()),
            (335, 0, 335)
        );
    }
}
