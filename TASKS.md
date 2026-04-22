# rust-csv — ETNA Tasks

Total tasks: 28

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `byte_record_eq_field_boundaries_efc4a51_1` | proptest | `ByteRecordEqMatchesFields` | `witness_byte_record_eq_matches_fields_case_boundary_shift` |
| 002 | `byte_record_eq_field_boundaries_efc4a51_1` | quickcheck | `ByteRecordEqMatchesFields` | `witness_byte_record_eq_matches_fields_case_boundary_shift` |
| 003 | `byte_record_eq_field_boundaries_efc4a51_1` | crabcheck | `ByteRecordEqMatchesFields` | `witness_byte_record_eq_matches_fields_case_boundary_shift` |
| 004 | `byte_record_eq_field_boundaries_efc4a51_1` | hegel | `ByteRecordEqMatchesFields` | `witness_byte_record_eq_matches_fields_case_boundary_shift` |
| 005 | `byte_record_eq_length_check_23fb0cd_1` | proptest | `ByteRecordEqMatchesFields` | `witness_byte_record_eq_matches_fields_case_length_mismatch` |
| 006 | `byte_record_eq_length_check_23fb0cd_1` | quickcheck | `ByteRecordEqMatchesFields` | `witness_byte_record_eq_matches_fields_case_length_mismatch` |
| 007 | `byte_record_eq_length_check_23fb0cd_1` | crabcheck | `ByteRecordEqMatchesFields` | `witness_byte_record_eq_matches_fields_case_length_mismatch` |
| 008 | `byte_record_eq_length_check_23fb0cd_1` | hegel | `ByteRecordEqMatchesFields` | `witness_byte_record_eq_matches_fields_case_length_mismatch` |
| 009 | `core_reader_comment_only_at_record_start_a5745ba_1` | proptest | `CommentOnlyAtRecordStart` | `witness_comment_only_at_record_start_case_mid_record_hash` |
| 010 | `core_reader_comment_only_at_record_start_a5745ba_1` | quickcheck | `CommentOnlyAtRecordStart` | `witness_comment_only_at_record_start_case_mid_record_hash` |
| 011 | `core_reader_comment_only_at_record_start_a5745ba_1` | crabcheck | `CommentOnlyAtRecordStart` | `witness_comment_only_at_record_start_case_mid_record_hash` |
| 012 | `core_reader_comment_only_at_record_start_a5745ba_1` | hegel | `CommentOnlyAtRecordStart` | `witness_comment_only_at_record_start_case_mid_record_hash` |
| 013 | `core_reader_reset_output_pos_zero_066de4a_1` | proptest | `ResetClearsOutputPosition` | `witness_reset_clears_output_position_case_short_field` |
| 014 | `core_reader_reset_output_pos_zero_066de4a_1` | quickcheck | `ResetClearsOutputPosition` | `witness_reset_clears_output_position_case_short_field` |
| 015 | `core_reader_reset_output_pos_zero_066de4a_1` | crabcheck | `ResetClearsOutputPosition` | `witness_reset_clears_output_position_case_short_field` |
| 016 | `core_reader_reset_output_pos_zero_066de4a_1` | hegel | `ResetClearsOutputPosition` | `witness_reset_clears_output_position_case_short_field` |
| 017 | `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | proptest | `DeserializeByteBufAcceptsNonUtf8` | `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` |
| 018 | `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | quickcheck | `DeserializeByteBufAcceptsNonUtf8` | `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` |
| 019 | `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | crabcheck | `DeserializeByteBufAcceptsNonUtf8` | `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` |
| 020 | `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | hegel | `DeserializeByteBufAcceptsNonUtf8` | `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` |
| 021 | `reader_trim_all_without_headers_ce01ae7_1` | proptest | `TrimAllAppliesWithoutHeaders` | `witness_trim_all_applies_without_headers_case_three_fields` |
| 022 | `reader_trim_all_without_headers_ce01ae7_1` | quickcheck | `TrimAllAppliesWithoutHeaders` | `witness_trim_all_applies_without_headers_case_three_fields` |
| 023 | `reader_trim_all_without_headers_ce01ae7_1` | crabcheck | `TrimAllAppliesWithoutHeaders` | `witness_trim_all_applies_without_headers_case_three_fields` |
| 024 | `reader_trim_all_without_headers_ce01ae7_1` | hegel | `TrimAllAppliesWithoutHeaders` | `witness_trim_all_applies_without_headers_case_three_fields` |
| 025 | `writer_comment_char_auto_quote_0f64d3f_1` | proptest | `WriterCommentCharAutoQuote` | `witness_writer_comment_char_auto_quote_case_hash_prefix` |
| 026 | `writer_comment_char_auto_quote_0f64d3f_1` | quickcheck | `WriterCommentCharAutoQuote` | `witness_writer_comment_char_auto_quote_case_hash_prefix` |
| 027 | `writer_comment_char_auto_quote_0f64d3f_1` | crabcheck | `WriterCommentCharAutoQuote` | `witness_writer_comment_char_auto_quote_case_hash_prefix` |
| 028 | `writer_comment_char_auto_quote_0f64d3f_1` | hegel | `WriterCommentCharAutoQuote` | `witness_writer_comment_char_auto_quote_case_hash_prefix` |

## Witness Catalog

- `witness_byte_record_eq_matches_fields_case_boundary_shift` — base passes, variant fails
- `witness_byte_record_eq_matches_fields_case_length_mismatch` — base passes, variant fails
- `witness_comment_only_at_record_start_case_mid_record_hash` — base passes, variant fails
- `witness_reset_clears_output_position_case_short_field` — base passes, variant fails
- `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` — base passes, variant fails
- `witness_trim_all_applies_without_headers_case_three_fields` — base passes, variant fails
- `witness_writer_comment_char_auto_quote_case_hash_prefix` — base passes, variant fails
