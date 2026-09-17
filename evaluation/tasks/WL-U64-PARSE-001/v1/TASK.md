# WL-U64-PARSE-001/v1

Development split of `WL-RUST-SMALL-001/v1`; family `strict-ascii-u64`;
profile `rust-library-change/1`. Public synthetic data, offline, no dependencies.

Change only `src/lib.rs` in the supplied `base/` library. Keep its public
`ParseError` variants and `parse_u64(&str) -> Result<u64, ParseError>` API.
Accept exactly `0` or an ASCII nonzero digit followed by ASCII digits, provided
the value is at most 18446744073709551615. Valid values round-trip to the same
decimal text. Reject empty input, invalid characters anywhere, leading zero,
and overflow, in that order. No whitespace trimming, signs, Unicode digits,
separators or normalization. Borrow the input; do not allocate a normalized copy.

At most three authored source/test files and 200 changed logical lines. Do not
change the manifest, dependencies, build hooks, oracle or task policy. Return a
patch and an explanation. Candidate output does not authorize acceptance.

`reference/` is an independently checked development solution, not candidate
input. `oracle/` is verifier input. Only `TASK.md` and `base/` belong in a
candidate snapshot. This repository's ordinary file permissions do not provide
hostile isolation; the later worker/collector must enforce that separation.

`manifest.json` freezes the exact development fixture and oracle bytes. The
complete balanced 80-development/200-held-out suite remains a T12 obligation;
this single development family is never eligible for the held-out denominator.
