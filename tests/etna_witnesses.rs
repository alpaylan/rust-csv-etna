//! Deterministic witness tests for rust-csv ETNA variants.
//!
//! Each `witness_<name>_case_<tag>` passes on the base HEAD and fails under the
//! corresponding `etna/<variant>` branch (or with `M_<variant>=active` under
//! marauders). Witnesses call `property_<name>` directly with frozen inputs —
//! no proptest/quickcheck/RNG/clock machinery.

use csv::etna::{
    property_byte_record_eq_matches_fields, property_comment_only_at_record_start,
    property_deserialize_byte_buf_accepts_non_utf8, property_reset_clears_output_position,
    property_trim_all_applies_without_headers, property_writer_comment_char_auto_quote,
    PropertyResult,
};

fn expect_pass(r: PropertyResult, what: &str) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Fail(m) => panic!("{what}: property failed: {m}"),
        PropertyResult::Discard => panic!("{what}: unexpected discard"),
    }
}

// Variant: core_reader_reset_output_pos_zero_066de4a_1
#[test]
fn witness_reset_clears_output_position_case_short_field() {
    expect_pass(
        property_reset_clears_output_position(b"hello".to_vec()),
        "reset_clears_output_position / short_field",
    );
}

// Variant: reader_trim_all_without_headers_ce01ae7_1
#[test]
fn witness_trim_all_applies_without_headers_case_three_fields() {
    expect_pass(
        property_trim_all_applies_without_headers(vec![
            b"a1".to_vec(),
            b"b1".to_vec(),
            b"c1".to_vec(),
        ]),
        "trim_all_applies_without_headers / three_fields",
    );
}

// Variant: writer_comment_char_auto_quote_0f64d3f_1
#[test]
fn witness_writer_comment_char_auto_quote_case_hash_prefix() {
    expect_pass(
        property_writer_comment_char_auto_quote(b" comment".to_vec()),
        "writer_comment_char_auto_quote / hash_prefix",
    );
}

// Variant: byte_record_eq_field_boundaries_efc4a51_1
// left = split("1234", [2]) = ["12","34"]; right = split("1234", [3]) = ["123","4"].
// Same base → records share as_slice(); boundary bug makes eq return true.
#[test]
fn witness_byte_record_eq_matches_fields_case_boundary_shift() {
    expect_pass(
        property_byte_record_eq_matches_fields(
            b"1234".to_vec(),
            vec![2],
            vec![3],
            0,
        ),
        "byte_record_eq / boundary_shift",
    );
}

// Variant: byte_record_eq_length_check_23fb0cd_1
// left = split("123456", [2,4]) = ["12","34","56"];
// right = split("1234", [2,4]) = ["12","34"] (trunc=2 drops "56").
// Length bug makes zip-over-shorter-iterator return true.
#[test]
fn witness_byte_record_eq_matches_fields_case_length_mismatch() {
    expect_pass(
        property_byte_record_eq_matches_fields(
            b"123456".to_vec(),
            vec![2, 4],
            vec![2, 4],
            2,
        ),
        "byte_record_eq / length_mismatch",
    );
}

// Variant: core_reader_comment_only_at_record_start_a5745ba_1
#[test]
fn witness_comment_only_at_record_start_case_mid_record_hash() {
    expect_pass(
        property_comment_only_at_record_start(b"bar".to_vec()),
        "comment_only_at_record_start / mid_record_hash",
    );
}

// Variant: deserialize_byte_buf_bypasses_utf8_9e644e6_1
#[test]
fn witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle() {
    expect_pass(
        property_deserialize_byte_buf_accepts_non_utf8(b"foo\xFFbar".to_vec()),
        "deserialize_byte_buf_accepts_non_utf8 / invalid_middle",
    );
}
