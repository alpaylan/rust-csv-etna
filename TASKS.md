# rust-csv — ETNA Tasks

Total tasks: 28

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task: the command executes the framework-specific adapter against the buggy variant branch and should report a counterexample.

Run against a variant by first checking out its branch (`git checkout etna/<variant>`) or applying its patch on a clean tree (`git apply patches/<variant>.patch`).

## Task Index

| Task | Variant | Framework | Property | Witness(es) | Command |
|------|---------|-----------|----------|-------------|---------|
| 001 | `core_reader_reset_output_pos_zero_066de4a_1` | proptest | `property_reset_clears_output_position` | `witness_reset_clears_output_position_case_short_field` | `cargo run --release --bin etna -- proptest ResetClearsOutputPosition` |
| 002 | `core_reader_reset_output_pos_zero_066de4a_1` | quickcheck | `property_reset_clears_output_position` | `witness_reset_clears_output_position_case_short_field` | `cargo run --release --bin etna -- quickcheck ResetClearsOutputPosition` |
| 003 | `core_reader_reset_output_pos_zero_066de4a_1` | crabcheck | `property_reset_clears_output_position` | `witness_reset_clears_output_position_case_short_field` | `cargo run --release --bin etna -- crabcheck ResetClearsOutputPosition` |
| 004 | `core_reader_reset_output_pos_zero_066de4a_1` | hegel | `property_reset_clears_output_position` | `witness_reset_clears_output_position_case_short_field` | `cargo run --release --bin etna -- hegel ResetClearsOutputPosition` |
| 005 | `reader_trim_all_without_headers_ce01ae7_1` | proptest | `property_trim_all_applies_without_headers` | `witness_trim_all_applies_without_headers_case_three_fields` | `cargo run --release --bin etna -- proptest TrimAllAppliesWithoutHeaders` |
| 006 | `reader_trim_all_without_headers_ce01ae7_1` | quickcheck | `property_trim_all_applies_without_headers` | `witness_trim_all_applies_without_headers_case_three_fields` | `cargo run --release --bin etna -- quickcheck TrimAllAppliesWithoutHeaders` |
| 007 | `reader_trim_all_without_headers_ce01ae7_1` | crabcheck | `property_trim_all_applies_without_headers` | `witness_trim_all_applies_without_headers_case_three_fields` | `cargo run --release --bin etna -- crabcheck TrimAllAppliesWithoutHeaders` |
| 008 | `reader_trim_all_without_headers_ce01ae7_1` | hegel | `property_trim_all_applies_without_headers` | `witness_trim_all_applies_without_headers_case_three_fields` | `cargo run --release --bin etna -- hegel TrimAllAppliesWithoutHeaders` |
| 009 | `writer_comment_char_auto_quote_0f64d3f_1` | proptest | `property_writer_comment_char_auto_quote` | `witness_writer_comment_char_auto_quote_case_hash_prefix` | `cargo run --release --bin etna -- proptest WriterCommentCharAutoQuote` |
| 010 | `writer_comment_char_auto_quote_0f64d3f_1` | quickcheck | `property_writer_comment_char_auto_quote` | `witness_writer_comment_char_auto_quote_case_hash_prefix` | `cargo run --release --bin etna -- quickcheck WriterCommentCharAutoQuote` |
| 011 | `writer_comment_char_auto_quote_0f64d3f_1` | crabcheck | `property_writer_comment_char_auto_quote` | `witness_writer_comment_char_auto_quote_case_hash_prefix` | `cargo run --release --bin etna -- crabcheck WriterCommentCharAutoQuote` |
| 012 | `writer_comment_char_auto_quote_0f64d3f_1` | hegel | `property_writer_comment_char_auto_quote` | `witness_writer_comment_char_auto_quote_case_hash_prefix` | `cargo run --release --bin etna -- hegel WriterCommentCharAutoQuote` |
| 013 | `byte_record_eq_field_boundaries_efc4a51_1` | proptest | `property_byte_record_eq_matches_fields` | `witness_byte_record_eq_matches_fields_case_boundary_shift` | `cargo run --release --bin etna -- proptest ByteRecordEqMatchesFields` |
| 014 | `byte_record_eq_field_boundaries_efc4a51_1` | quickcheck | `property_byte_record_eq_matches_fields` | `witness_byte_record_eq_matches_fields_case_boundary_shift` | `cargo run --release --bin etna -- quickcheck ByteRecordEqMatchesFields` |
| 015 | `byte_record_eq_field_boundaries_efc4a51_1` | crabcheck | `property_byte_record_eq_matches_fields` | `witness_byte_record_eq_matches_fields_case_boundary_shift` | `cargo run --release --bin etna -- crabcheck ByteRecordEqMatchesFields` |
| 016 | `byte_record_eq_field_boundaries_efc4a51_1` | hegel | `property_byte_record_eq_matches_fields` | `witness_byte_record_eq_matches_fields_case_boundary_shift` | `cargo run --release --bin etna -- hegel ByteRecordEqMatchesFields` |
| 017 | `byte_record_eq_length_check_23fb0cd_1` | proptest | `property_byte_record_eq_matches_fields` | `witness_byte_record_eq_matches_fields_case_length_mismatch` | `cargo run --release --bin etna -- proptest ByteRecordEqMatchesFields` |
| 018 | `byte_record_eq_length_check_23fb0cd_1` | quickcheck | `property_byte_record_eq_matches_fields` | `witness_byte_record_eq_matches_fields_case_length_mismatch` | `cargo run --release --bin etna -- quickcheck ByteRecordEqMatchesFields` |
| 019 | `byte_record_eq_length_check_23fb0cd_1` | crabcheck | `property_byte_record_eq_matches_fields` | `witness_byte_record_eq_matches_fields_case_length_mismatch` | `cargo run --release --bin etna -- crabcheck ByteRecordEqMatchesFields` |
| 020 | `byte_record_eq_length_check_23fb0cd_1` | hegel | `property_byte_record_eq_matches_fields` | `witness_byte_record_eq_matches_fields_case_length_mismatch` | `cargo run --release --bin etna -- hegel ByteRecordEqMatchesFields` |
| 021 | `core_reader_comment_only_at_record_start_a5745ba_1` | proptest | `property_comment_only_at_record_start` | `witness_comment_only_at_record_start_case_mid_record_hash` | `cargo run --release --bin etna -- proptest CommentOnlyAtRecordStart` |
| 022 | `core_reader_comment_only_at_record_start_a5745ba_1` | quickcheck | `property_comment_only_at_record_start` | `witness_comment_only_at_record_start_case_mid_record_hash` | `cargo run --release --bin etna -- quickcheck CommentOnlyAtRecordStart` |
| 023 | `core_reader_comment_only_at_record_start_a5745ba_1` | crabcheck | `property_comment_only_at_record_start` | `witness_comment_only_at_record_start_case_mid_record_hash` | `cargo run --release --bin etna -- crabcheck CommentOnlyAtRecordStart` |
| 024 | `core_reader_comment_only_at_record_start_a5745ba_1` | hegel | `property_comment_only_at_record_start` | `witness_comment_only_at_record_start_case_mid_record_hash` | `cargo run --release --bin etna -- hegel CommentOnlyAtRecordStart` |
| 025 | `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | proptest | `property_deserialize_byte_buf_accepts_non_utf8` | `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` | `cargo run --release --bin etna -- proptest DeserializeByteBufAcceptsNonUtf8` |
| 026 | `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | quickcheck | `property_deserialize_byte_buf_accepts_non_utf8` | `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` | `cargo run --release --bin etna -- quickcheck DeserializeByteBufAcceptsNonUtf8` |
| 027 | `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | crabcheck | `property_deserialize_byte_buf_accepts_non_utf8` | `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` | `cargo run --release --bin etna -- crabcheck DeserializeByteBufAcceptsNonUtf8` |
| 028 | `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | hegel | `property_deserialize_byte_buf_accepts_non_utf8` | `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` | `cargo run --release --bin etna -- hegel DeserializeByteBufAcceptsNonUtf8` |

## Witness catalog

Each witness is a deterministic concrete test in `tests/etna_witnesses.rs`. Base build: passes. Variant-active build: fails.

- `witness_reset_clears_output_position_case_short_field` — `property_reset_clears_output_position(b"hello".to_vec())` → `Pass`. Under `core_reader_reset_output_pos_zero_066de4a_1` the reader's stale `output_pos` survives the `reset()` call and `ends[0]` reports an inflated offset instead of `5`.
- `witness_trim_all_applies_without_headers_case_three_fields` — `property_trim_all_applies_without_headers(vec![b"a1", b"b1", b"c1"])` → `Pass`. Under `reader_trim_all_without_headers_ce01ae7_1` the `Trim::All` pass is gated behind the headers `else` arm and `has_headers(false)` records retain their surrounding whitespace.
- `witness_writer_comment_char_auto_quote_case_hash_prefix` — `property_writer_comment_char_auto_quote(b" comment".to_vec())` → `Pass`. Under `writer_comment_char_auto_quote_0f64d3f_1` a field starting with `#` writes unquoted; on read-back with `comment = Some(b'#')` the entire row is swallowed as a comment.
- `witness_byte_record_eq_matches_fields_case_boundary_shift` — `property_byte_record_eq_matches_fields(b"1234".to_vec(), vec![2], vec![3], 0)` → `Pass`. Same base `"1234"`, splits at `[2]` vs `[3]` yield `["12","34"]` vs `["123","4"]`. Under `byte_record_eq_field_boundaries_efc4a51_1` the two records compare equal because the fallback compares concatenated field bytes.
- `witness_byte_record_eq_matches_fields_case_length_mismatch` — `property_byte_record_eq_matches_fields(b"123456".to_vec(), vec![2, 4], vec![2, 4], 2)` → `Pass`. Left uses full base → `["12","34","56"]`; right uses base truncated by 2 → `["12","34"]`. Under `byte_record_eq_length_check_23fb0cd_1` the shorter record compares equal because the zip-based field check silently truncates at the shorter iterator.
- `witness_comment_only_at_record_start_case_mid_record_hash` — `property_comment_only_at_record_start(b"bar".to_vec())` → `Pass`. Under `core_reader_comment_only_at_record_start_a5745ba_1` parsing `first,#bar\n` with `comment = Some(b'#')` drops the second field as a comment; the parser emits only one field.
- `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` — `property_deserialize_byte_buf_accepts_non_utf8(b"foo\xFFbar".to_vec())` → `Pass`. Under `deserialize_byte_buf_bypasses_utf8_9e644e6_1` the middle `#[serde(with = "serde_bytes")]` field is UTF-8-validated and the deserialize call fails with an invalid-UTF-8 error.
