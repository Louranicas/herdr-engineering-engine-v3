//! Primary owner: contracts. RC01 strict decimal and RC03 scalar regression scope.
//! These are development observations, not a 50-case qualification inventory.
//! Table rows and property iterations do not multiply behavioral case credit.
//! JSON framing/serialization, caller acquisition bounds, and authority are absent.

use habitat_engine::contracts::{
    Generation, ScalarError, Sha256Digest, U32Decimal, U64Decimal, UuidV4, parse_u64_decimal,
};

#[test]
fn empty_decimal_has_its_own_refusal() {
    assert_eq!(parse_u64_decimal(""), Err(ScalarError::Empty));
}

#[test]
fn all_single_ascii_digits_have_their_known_values() {
    for (text, expected) in [
        ("0", 0),
        ("1", 1),
        ("2", 2),
        ("3", 3),
        ("4", 4),
        ("5", 5),
        ("6", 6),
        ("7", 7),
        ("8", 8),
        ("9", 9),
    ] {
        assert_eq!(parse_u64_decimal(text), Ok(expected));
    }
}

#[test]
fn internal_and_trailing_zero_are_canonical() {
    assert_eq!(parse_u64_decimal("1002000"), Ok(1_002_000));
}

#[test]
fn exact_unsigned_64_bit_limit_is_accepted() {
    assert_eq!(parse_u64_decimal("18446744073709551615"), Ok(u64::MAX));
}

#[test]
fn one_past_unsigned_64_bit_limit_is_refused() {
    assert_eq!(
        parse_u64_decimal("18446744073709551616"),
        Err(ScalarError::Overflow)
    );
}

#[test]
fn long_valid_digit_sequence_is_overflow() {
    assert_eq!(
        parse_u64_decimal(&"9".repeat(512)),
        Err(ScalarError::Overflow)
    );
}

#[test]
fn leading_zero_refuses_even_when_magnitude_would_overflow() {
    for text in ["00", "01", "000184467440737095516160000"] {
        assert_eq!(parse_u64_decimal(text), Err(ScalarError::LeadingZero));
    }
}

#[test]
fn invalid_suffix_takes_precedence_over_overflow() {
    let text = format!("{}x", "9".repeat(512));
    assert_eq!(parse_u64_decimal(&text), Err(ScalarError::InvalidCharacter));
}

#[test]
fn invalid_character_takes_precedence_over_leading_zero() {
    assert_eq!(
        parse_u64_decimal("00018446744073709551616x"),
        Err(ScalarError::InvalidCharacter)
    );
}

#[test]
fn decimal_grammar_does_not_trim_or_accept_signs_and_separators() {
    for text in [
        " 1", "1 ", "\t1", "1\n", "+1", "-1", "1_000", "1,000", "1.0",
    ] {
        assert_eq!(
            parse_u64_decimal(text),
            Err(ScalarError::InvalidCharacter),
            "{text:?}"
        );
    }
}

#[test]
fn unicode_digits_are_not_ascii_digits() {
    for text in ["١", "１", "1²", "1\u{00a0}"] {
        assert_eq!(parse_u64_decimal(text), Err(ScalarError::InvalidCharacter));
    }
}

#[test]
fn embedded_nul_does_not_end_decimal_validation() {
    assert_eq!(parse_u64_decimal("1\0"), Err(ScalarError::InvalidCharacter));
}

#[test]
fn canonical_decimal_round_trip_matches_independently_chosen_numbers() {
    // One property; the enumeration and boundary values are exploration depth.
    for expected in (0_u64..=10_000).chain([
        u64::from(u32::MAX),
        9_007_199_254_740_993,
        u64::MAX - 1,
        u64::MAX,
    ]) {
        let text = expected.to_string();
        let parsed = text.parse::<U64Decimal>().unwrap();
        assert_eq!(parsed.value(), expected);
        assert_eq!(parsed.to_string(), text);
    }
}

#[test]
fn u64_wrapper_preserves_typed_parse_failures() {
    for (text, expected) in [
        ("", ScalarError::Empty),
        ("+0", ScalarError::InvalidCharacter),
        ("00", ScalarError::LeadingZero),
        ("18446744073709551616", ScalarError::Overflow),
    ] {
        assert_eq!(text.parse::<U64Decimal>(), Err(expected));
    }
}

#[test]
fn bounded_usage_accepts_zero_and_exact_u32_limit() {
    for (text, expected) in [("0", 0), ("4294967295", u32::MAX)] {
        let value = text.parse::<U32Decimal>().unwrap();
        assert_eq!(value.value(), expected);
        assert_eq!(value.to_string(), text);
    }
}

#[test]
fn bounded_usage_refuses_values_that_still_fit_u64() {
    assert_eq!(
        "4294967296".parse::<U32Decimal>(),
        Err(ScalarError::Overflow)
    );
}

#[test]
fn bounded_usage_preserves_lexical_error_precedence() {
    assert_eq!(
        "4294967296x".parse::<U32Decimal>(),
        Err(ScalarError::InvalidCharacter)
    );
    assert_eq!(
        "04294967296".parse::<U32Decimal>(),
        Err(ScalarError::LeadingZero)
    );
}

#[test]
fn generation_rejects_zero_without_changing_generic_u64_semantics() {
    assert_eq!("0".parse::<Generation>(), Err(ScalarError::ZeroGeneration));
    assert_eq!("0".parse::<U64Decimal>().unwrap().value(), 0);
    assert_eq!("00".parse::<Generation>(), Err(ScalarError::LeadingZero));
}

#[test]
fn generation_advances_without_wrapping() {
    let first = "1".parse::<Generation>().unwrap();
    assert_eq!(first.next().unwrap().value(), 2);
    let maximum = "18446744073709551615".parse::<Generation>().unwrap();
    assert_eq!(maximum.to_string(), "18446744073709551615");
    assert_eq!(maximum.next(), Err(ScalarError::Overflow));
    assert_eq!(maximum.value(), u64::MAX);
}

#[test]
fn uuid_accepts_each_rfc_variant_encoding_without_rewriting_text() {
    for text in [
        "00112233-4455-4677-8899-aabbccddeeff",
        "00112233-4455-4677-9899-aabbccddeeff",
        "00112233-4455-4677-a899-aabbccddeeff",
        "00112233-4455-4677-b899-aabbccddeeff",
    ] {
        assert_eq!(UuidV4::parse(text).unwrap().as_str(), text);
    }
}

#[test]
fn uuid_refuses_noncanonical_shape_and_case() {
    for text in [
        "",
        "00112233445546778899aabbccddeeff",
        "00112233_4455-4677-8899-aabbccddeeff",
        "00112233-4455-4677-8899-AABBCCDDEEFF",
        "00112233-4455-4677-8899-aabbccddeefg",
        "00112233-4455-4677-8899-aabbccddeeff\n",
    ] {
        assert_eq!(UuidV4::parse(text), Err(ScalarError::InvalidUuid));
    }
}

#[test]
fn uuid_refuses_wrong_version_variant_and_nil() {
    for text in [
        "00000000-0000-0000-0000-000000000000",
        "00112233-4455-5677-8899-aabbccddeeff",
        "00112233-4455-4677-7899-aabbccddeeff",
        "00112233-4455-4677-c899-aabbccddeeff",
    ] {
        assert_eq!(UuidV4::parse(text), Err(ScalarError::InvalidUuid));
    }
}

#[test]
fn digest_accepts_exact_lowercase_reference_without_computing_authenticity() {
    let text = format!("sha256:{}", "0123456789abcdef".repeat(4));
    assert_eq!(Sha256Digest::parse(&text).unwrap().as_str(), text);
}

#[test]
fn digest_refuses_wrong_length_prefix_and_alphabet() {
    for text in [
        String::new(),
        format!("sha256:{}", "a".repeat(63)),
        format!("sha256:{}", "a".repeat(65)),
        format!("SHA256:{}", "a".repeat(64)),
        format!("sha256:{}", "A".repeat(64)),
        format!("sha256:{}", "g".repeat(64)),
    ] {
        assert_eq!(Sha256Digest::parse(&text), Err(ScalarError::InvalidDigest));
    }
}
