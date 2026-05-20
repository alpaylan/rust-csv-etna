//! End-to-end fault-localization tests for `csv` properties.
//!
//! Each `#[test]` runs `crabcheck::quickcheck_with_locate!` on one property
//! from `etna-faultloc.rs`. Tests never panic — they print the LocateResult
//! and emit one `@@LOCATE@@ <json>` line per property so a harness can
//! collect machine-readable suspect summaries.

use std::fmt;

use crabcheck::quickcheck::{Arbitrary, Mutate};
use csv::etna::{
    property_byte_record_eq_matches_fields, property_comment_only_at_record_start,
    property_deserialize_byte_buf_accepts_non_utf8, property_reset_clears_output_position,
    property_trim_all_applies_without_headers, property_writer_comment_char_auto_quote,
    PropertyResult,
};
use rand::Rng;

#[derive(Clone)]
struct Bytes24(Vec<u8>);
impl fmt::Debug for Bytes24 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone)]
struct Splits5(Vec<u8>);
impl fmt::Debug for Splits5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone)]
struct ByteFields(Vec<Vec<u8>>);
impl fmt::Debug for ByteFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// Newtype around `u8` so we can implement `Arbitrary` + `Mutate` for it
// (crabcheck only ships impls for `i32`, `usize`, `bool`, `Vec`, and tuples).
#[derive(Clone)]
struct ByteVal(u8);
impl fmt::Debug for ByteVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<R: Rng> Arbitrary<R> for ByteVal {
    fn generate(rng: &mut R, _n: usize) -> Self {
        ByteVal(rng.random_range(0..=u8::MAX))
    }
}
impl<R: Rng> Mutate<R> for ByteVal {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        ByteVal(self.0 ^ (1u8 << rng.random_range(0u32..8)))
    }
}

fn gen_bytes<R: Rng>(rng: &mut R, max: u32) -> Vec<u8> {
    let len = rng.random_range(0..max) as usize;
    (0..len).map(|_| rng.random_range(0..=u8::MAX)).collect()
}

fn mutate_bytes<R: Rng>(rng: &mut R, v: &[u8], max: usize) -> Vec<u8> {
    let mut out = v.to_vec();
    match rng.random_range(0u8..3) {
        0 if !out.is_empty() => {
            let i = rng.random_range(0..out.len());
            out[i] ^= 1u8 << rng.random_range(0u32..8);
        }
        1 if out.len() < max => out.push(rng.random()),
        _ if !out.is_empty() => {
            out.pop();
        }
        _ => {}
    }
    out
}

impl<R: Rng> Arbitrary<R> for Bytes24 {
    fn generate(rng: &mut R, _n: usize) -> Self {
        Bytes24(gen_bytes(rng, 24))
    }
}
impl<R: Rng> Mutate<R> for Bytes24 {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        Bytes24(mutate_bytes(rng, &self.0, 24))
    }
}

impl<R: Rng> Arbitrary<R> for Splits5 {
    fn generate(rng: &mut R, _n: usize) -> Self {
        Splits5(gen_bytes(rng, 5))
    }
}
impl<R: Rng> Mutate<R> for Splits5 {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        Splits5(mutate_bytes(rng, &self.0, 5))
    }
}

impl<R: Rng> Arbitrary<R> for ByteFields {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let outer = rng.random_range(0..5u32) as usize;
        let mut out = Vec::with_capacity(outer);
        for _ in 0..outer {
            out.push(gen_bytes(rng, 6));
        }
        ByteFields(out)
    }
}
impl<R: Rng> Mutate<R> for ByteFields {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.0.clone();
        match rng.random_range(0u8..4) {
            0 if !out.is_empty() => {
                let i = rng.random_range(0..out.len());
                out[i] = mutate_bytes(rng, &out[i], 6);
            }
            1 if out.len() < 5 => out.push(gen_bytes(rng, 6)),
            _ if !out.is_empty() => {
                out.pop();
            }
            _ => {}
        }
        ByteFields(out)
    }
}

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn reset_clears_output_position_wrapper(Bytes24(v): Bytes24) -> Option<bool> {
    to_opt(property_reset_clears_output_position(v))
}

fn trim_all_applies_without_headers_wrapper(ByteFields(v): ByteFields) -> Option<bool> {
    to_opt(property_trim_all_applies_without_headers(v))
}

fn writer_comment_char_auto_quote_wrapper(
    (Bytes24(tail), Bytes24(after), ByteVal(comment)): (Bytes24, Bytes24, ByteVal),
) -> Option<bool> {
    to_opt(property_writer_comment_char_auto_quote(tail, after, comment))
}

fn byte_record_eq_matches_fields_wrapper(
    (Bytes24(base), (Splits5(sa), Splits5(sb), trunc)): (Bytes24, (Splits5, Splits5, usize)),
) -> Option<bool> {
    to_opt(property_byte_record_eq_matches_fields(base, sa, sb, trunc as u8))
}

fn comment_only_at_record_start_wrapper(
    (Bytes24(first), Bytes24(tail), ByteVal(comment)): (Bytes24, Bytes24, ByteVal),
) -> Option<bool> {
    to_opt(property_comment_only_at_record_start(first, tail, comment))
}

fn deserialize_byte_buf_accepts_non_utf8_wrapper(Bytes24(v): Bytes24) -> Option<bool> {
    to_opt(property_deserialize_byte_buf_accepts_non_utf8(v))
}

fn emit_locate_json(r: &crabcheck::profiling::LocateResult) {
    use crabcheck::quickcheck::ResultStatus;
    let status = match &r.run.status {
        ResultStatus::Failed { .. } => "Failed",
        ResultStatus::Finished => "Finished",
        ResultStatus::GaveUp => "GaveUp",
        ResultStatus::TimedOut => "TimedOut",
        ResultStatus::Aborted { .. } => "Aborted",
    };
    let top = if let Some(s) = r.top() {
        serde_json::json!({
            "rank": s.rank,
            "file": s.region.file,
            "function": s.region.function,
            "start_line": s.region.start_line,
            "end_line": s.region.end_line,
            "ochiai": s.region.suspiciousness.ochiai,
            "delta": s.region.delta,
            "panic_overlap": s.panic_overlap,
            "confidence": format!("{}", s.confidence),
            "confidence_rule": s.confidence_rule,
        })
    } else {
        serde_json::Value::Null
    };
    let top_5: Vec<_> = r
        .suspects
        .iter()
        .take(5)
        .map(|s| {
            serde_json::json!({
                "rank": s.rank,
                "file": s.region.file,
                "function": s.region.function,
                "start_line": s.region.start_line,
                "end_line": s.region.end_line,
                "confidence": format!("{}", s.confidence),
                "confidence_rule": s.confidence_rule,
                "panic_overlap": s.panic_overlap,
            })
        })
        .collect();
    let diags: Vec<_> = r.diagnostics.iter().map(|d| d.tag()).collect();
    let out = serde_json::json!({
        "status": status,
        "passed": r.run.passed,
        "discarded": r.run.discarded,
        "n_panics": r.n_panics,
        "n_suspects": r.suspects.len(),
        "top": top,
        "top_5": top_5,
        "diagnostics": diags,
    });
    println!("@@LOCATE@@ {}", out);
}

#[test]
fn locate_reset_clears_output_position() {
    let report = crabcheck::quickcheck_with_locate!(reset_clears_output_position_wrapper, "csv");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_trim_all_applies_without_headers() {
    let report =
        crabcheck::quickcheck_with_locate!(trim_all_applies_without_headers_wrapper, "csv");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_writer_comment_char_auto_quote() {
    let report = crabcheck::quickcheck_with_locate!(writer_comment_char_auto_quote_wrapper, "csv");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_byte_record_eq_matches_fields() {
    let report = crabcheck::quickcheck_with_locate!(byte_record_eq_matches_fields_wrapper, "csv");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_comment_only_at_record_start() {
    let report = crabcheck::quickcheck_with_locate!(comment_only_at_record_start_wrapper, "csv");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_deserialize_byte_buf_accepts_non_utf8() {
    let report = crabcheck::quickcheck_with_locate!(
        deserialize_byte_buf_accepts_non_utf8_wrapper,
        "csv"
    );
    eprintln!("{report}");
    emit_locate_json(&report);
}
