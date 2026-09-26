#!/usr/bin/env python3
"""Author fixed expectations using grammar and arbitrary-precision arithmetic.

This is a development oracle author, never part of the candidate snapshot.
The test runner consumes the frozen output and checks its hashes first.
"""
import json
import re
from pathlib import Path

MAXIMUM = 2**64 - 1


def expected(value):
    if value == "":
        return {"error": "Empty"}
    if re.fullmatch(r"[0-9]+", value) is None:
        return {"error": "InvalidCharacter"}
    if re.fullmatch(r"0|[1-9][0-9]*", value) is None:
        return {"error": "LeadingZero"}
    number = int(value)
    return {"value": str(number)} if number <= MAXIMUM else {"error": "Overflow"}


def cases():
    fixed = [
        ("empty", "", {"error": "Empty"}),
        ("zero", "0", {"value": "0"}),
        ("maximum", "18446744073709551615", {"value": "18446744073709551615"}),
        ("one_past_maximum", "18446744073709551616", {"error": "Overflow"}),
        ("leading_zero", "01", {"error": "LeadingZero"}),
        ("leading_zero_overflow", "018446744073709551616", {"error": "LeadingZero"}),
        ("invalid_after_overflow", "18446744073709551616x", {"error": "InvalidCharacter"}),
        ("invalid_after_leading_zero", "00x", {"error": "InvalidCharacter"}),
        ("plus_sign", "+1", {"error": "InvalidCharacter"}),
        ("minus_sign", "-1", {"error": "InvalidCharacter"}),
        ("non_ascii_digit", "١", {"error": "InvalidCharacter"}),
        ("full_width_digit", "１", {"error": "InvalidCharacter"}),
        ("embedded_nul", "1\x000", {"error": "InvalidCharacter"}),
        ("trailing_newline", "1\n", {"error": "InvalidCharacter"}),
        ("leading_space", " 1", {"error": "InvalidCharacter"}),
        ("internal_zero", "10001", {"value": "10001"}),
    ]
    result = []
    for name, value, oracle in fixed:
        if expected(value) != oracle:
            raise ValueError("Arithmetic/grammar oracle contradicts fixed boundary " + name)
        result.append({"id": name, "input": value, "expected": oracle, "basis": "fixed_boundary"})
    generated = {str(n) for n in range(10)}
    generated.update(str(2**power + offset) for power in range(65) for offset in (-1, 0, 1) if 2**power + offset >= 0)
    generated.update({"00", "0" * 1024, "9" * 1024, "9" * 1024 + "!", "0" * 1024 + "!"})
    generated.update("1" + chr(code) + "0" for code in range(128) if not 48 <= code <= 57)
    generated.update({"1_000", "1.0", "1e3", "0x10", "\ufeff1", "1\u00a0", "1\u200b", "1😀"})
    used = {row["input"] for row in result}
    for index, value in enumerate(sorted(generated - used)):
        result.append({"id": f"grammar_arithmetic_{index:03d}", "input": value, "expected": expected(value), "basis": "grammar_arbitrary_precision"})
    return {"id": "ORACLE-U64-001/v1", "case_count_is_not_module_credit": True, "cases": result}


if __name__ == "__main__":
    Path(__file__).with_name("cases.json").write_text(json.dumps(cases(), indent=2, ensure_ascii=False) + "\n")
