// ETNA workload runner for rust-csv.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: ResetClearsOutputPosition
//           | TrimAllAppliesWithoutHeaders
//           | WriterCommentCharAutoQuote
//           | ByteRecordEqMatchesFields
//           | CommentOnlyAtRecordStart
//           | DeserializeByteBufAcceptsNonUtf8
//           | All
//
// Each invocation emits a single JSON line on stdout and exits 0
// (usage errors exit 2).

use crabcheck::quickcheck as crabcheck_qc;
use csv::etna::{
    property_byte_record_eq_matches_fields, property_comment_only_at_record_start,
    property_deserialize_byte_buf_accepts_non_utf8, property_reset_clears_output_position,
    property_trim_all_applies_without_headers, property_writer_comment_char_auto_quote,
    PropertyResult,
};
use hegel::{generators as hgen, Hegel, Settings as HegelSettings};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestRunner};
use quickcheck::{QuickCheck, ResultStatus, TestResult};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &[
    "ResetClearsOutputPosition",
    "TrimAllAppliesWithoutHeaders",
    "WriterCommentCharAutoQuote",
    "ByteRecordEqMatchesFields",
    "CommentOnlyAtRecordStart",
    "DeserializeByteBufAcceptsNonUtf8",
];

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if let Err(e) = r {
            return (Err(e), total);
        }
    }
    (Ok(()), total)
}

// ───────────── etna tool: replays frozen witness inputs. ─────────────
fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "ResetClearsOutputPosition" => {
            to_err(property_reset_clears_output_position(b"hello".to_vec()))
        }
        "TrimAllAppliesWithoutHeaders" => {
            to_err(property_trim_all_applies_without_headers(vec![
                b"a1".to_vec(),
                b"b1".to_vec(),
                b"c1".to_vec(),
            ]))
        }
        "WriterCommentCharAutoQuote" => {
            to_err(property_writer_comment_char_auto_quote(b" comment".to_vec()))
        }
        "ByteRecordEqMatchesFields" => {
            // Two variants share this property (boundary + length). Run both
            // witnesses so etna replay flags either bug.
            let boundary = property_byte_record_eq_matches_fields(
                b"1234".to_vec(),
                vec![2],
                vec![3],
                0,
            );
            let length = property_byte_record_eq_matches_fields(
                b"123456".to_vec(),
                vec![2, 4],
                vec![2, 4],
                2,
            );
            match (boundary, length) {
                (PropertyResult::Fail(m), _) | (_, PropertyResult::Fail(m)) => Err(m),
                _ => Ok(()),
            }
        }
        "CommentOnlyAtRecordStart" => {
            to_err(property_comment_only_at_record_start(b"bar".to_vec()))
        }
        "DeserializeByteBufAcceptsNonUtf8" => to_err(
            property_deserialize_byte_buf_accepts_non_utf8(b"foo\xFFbar".to_vec()),
        ),
        _ => {
            return (
                Err(format!("Unknown property: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    (result, Metrics { inputs: 1, elapsed_us })
}

// ───────────── proptest ─────────────
fn bytes_strategy() -> BoxedStrategy<Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..24).boxed()
}

fn vec_bytes_strategy() -> BoxedStrategy<Vec<Vec<u8>>> {
    prop::collection::vec(prop::collection::vec(any::<u8>(), 0..6), 0..5).boxed()
}

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let mut runner = TestRunner::new(ProptestConfig::default());
    let result: Result<(), String> = match property {
        "ResetClearsOutputPosition" => {
            let c = counter.clone();
            runner
                .run(&bytes_strategy(), move |v| {
                    c.fetch_add(1, Ordering::Relaxed);
                    match property_reset_clears_output_position(v) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                    }
                })
                .map_err(|e| e.to_string())
        }
        "TrimAllAppliesWithoutHeaders" => {
            let c = counter.clone();
            runner
                .run(&vec_bytes_strategy(), move |v| {
                    c.fetch_add(1, Ordering::Relaxed);
                    match property_trim_all_applies_without_headers(v) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                    }
                })
                .map_err(|e| e.to_string())
        }
        "WriterCommentCharAutoQuote" => {
            let c = counter.clone();
            runner
                .run(&bytes_strategy(), move |v| {
                    c.fetch_add(1, Ordering::Relaxed);
                    match property_writer_comment_char_auto_quote(v) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                    }
                })
                .map_err(|e| e.to_string())
        }
        "ByteRecordEqMatchesFields" => {
            let c = counter.clone();
            let splits_strategy = prop::collection::vec(any::<u8>(), 0..5);
            runner
                .run(
                    &(
                        bytes_strategy(),
                        splits_strategy.clone(),
                        splits_strategy,
                        any::<u8>(),
                    ),
                    move |(base, sa, sb, trunc)| {
                        c.fetch_add(1, Ordering::Relaxed);
                        match property_byte_record_eq_matches_fields(base, sa, sb, trunc) {
                            PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                            PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                        }
                    },
                )
                .map_err(|e| e.to_string())
        }
        "CommentOnlyAtRecordStart" => {
            let c = counter.clone();
            runner
                .run(&bytes_strategy(), move |v| {
                    c.fetch_add(1, Ordering::Relaxed);
                    match property_comment_only_at_record_start(v) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                    }
                })
                .map_err(|e| e.to_string())
        }
        "DeserializeByteBufAcceptsNonUtf8" => {
            let c = counter.clone();
            runner
                .run(&bytes_strategy(), move |v| {
                    c.fetch_add(1, Ordering::Relaxed);
                    match property_deserialize_byte_buf_accepts_non_utf8(v) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                    }
                })
                .map_err(|e| e.to_string())
        }
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ───────────── quickcheck (fork with `etna` feature) ─────────────
//
// The etna feature on QuickCheck's Testable impl requires `Display` on every
// argument, which `Vec<u8>` does not implement. So the adapters take scalar
// seeds (u64) and expand them deterministically into the Vec<u8> / Vec<Vec<u8>>
// inputs that the `property_*` functions consume. This matches how other
// workloads (bitvec-rs, unicode-segmentation) handle the same constraint.
static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn seed_to_bytes(seed: u64) -> Vec<u8> {
    let len = ((seed >> 56) as usize) % 24 + 1;
    let mut out = Vec::with_capacity(len);
    let mut s = seed;
    for _ in 0..len {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        out.push((s >> 33) as u8);
    }
    out
}

fn seed_to_byte_fields(seed: u64) -> Vec<Vec<u8>> {
    let nfields = ((seed >> 60) as usize) % 5 + 1;
    let mut out = Vec::with_capacity(nfields);
    let mut s = seed;
    for i in 0..nfields {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let flen = ((s >> 58) as usize) % 6 + 1;
        let mut field = Vec::with_capacity(flen);
        for j in 0..flen {
            s = s.wrapping_mul(2862933555777941757).wrapping_add(3037000493);
            let b = (s >> 33) as u8;
            // Keep mostly-printable bytes so TrimAll etc. don't always discard.
            let b = if b == 0 { b'a' + ((i as u8).wrapping_add(j as u8) % 26) } else { b };
            field.push(b);
        }
        out.push(field);
    }
    out
}

fn qc_reset_clears_output_position(seed: u64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_reset_clears_output_position(seed_to_bytes(seed)) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_trim_all_applies_without_headers(seed: u64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_trim_all_applies_without_headers(seed_to_byte_fields(seed)) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_writer_comment_char_auto_quote(seed: u64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_writer_comment_char_auto_quote(seed_to_bytes(seed)) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn seed_to_splits(seed: u64) -> Vec<u8> {
    // Up to 4 split positions drawn from the seed.
    let n = ((seed >> 60) as usize) % 5;
    let mut out = Vec::with_capacity(n);
    let mut s = seed;
    for _ in 0..n {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        out.push((s >> 56) as u8);
    }
    out
}

fn qc_byte_record_eq_matches_fields(a: u64, b: u64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let base = seed_to_bytes(a);
    let splits_a = seed_to_splits(a ^ 0xA5A5_5A5A_5A5A_A5A5);
    let splits_b = seed_to_splits(b);
    let trunc_b = ((b >> 56) as u8) / 32;
    match property_byte_record_eq_matches_fields(base, splits_a, splits_b, trunc_b) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_comment_only_at_record_start(seed: u64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_comment_only_at_record_start(seed_to_bytes(seed)) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_deserialize_byte_buf_accepts_non_utf8(seed: u64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_deserialize_byte_buf_accepts_non_utf8(seed_to_bytes(seed)) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let mut qc = QuickCheck::new().tests(200).max_tests(2000);
    let result = match property {
        "ResetClearsOutputPosition" => {
            qc.quicktest(qc_reset_clears_output_position as fn(u64) -> TestResult)
        }
        "TrimAllAppliesWithoutHeaders" => {
            qc.quicktest(qc_trim_all_applies_without_headers as fn(u64) -> TestResult)
        }
        "WriterCommentCharAutoQuote" => {
            qc.quicktest(qc_writer_comment_char_auto_quote as fn(u64) -> TestResult)
        }
        "ByteRecordEqMatchesFields" => {
            qc.quicktest(qc_byte_record_eq_matches_fields as fn(u64, u64) -> TestResult)
        }
        "CommentOnlyAtRecordStart" => {
            qc.quicktest(qc_comment_only_at_record_start as fn(u64) -> TestResult)
        }
        "DeserializeByteBufAcceptsNonUtf8" => {
            qc.quicktest(qc_deserialize_byte_buf_accepts_non_utf8 as fn(u64) -> TestResult)
        }
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!(
            "quickcheck counterexample: ({})",
            arguments.join(" ")
        )),
        ResultStatus::Aborted { err } => Err(format!("quickcheck aborted: {err:?}")),
        ResultStatus::TimedOut => Err("quickcheck timed out".into()),
        ResultStatus::GaveUp => Err(format!(
            "quickcheck gave up: passed={}, discarded={}",
            result.n_tests_passed, result.n_tests_discarded
        )),
    };
    (status, metrics)
}

// ───────────── crabcheck ─────────────
static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn usize_to_u8(x: usize) -> u8 {
    // crabcheck's Arbitrary<usize> yields values in 0..=log2(i+1) (max ~14 over
    // a 20k-iteration run). Raw casting would collapse every input into the
    // control-character range, which `normalize_field` filters to empty and
    // every property silently discards. A modular alphabet lookup has the same
    // problem — only the first ~14 slots are ever hit. Use a multiplicative
    // hash to spread small usizes across the full byte range so alphanumeric,
    // punctuation, and high (invalid-UTF-8) bytes all appear regularly.
    let h = (x as u32).wrapping_mul(2654435761);
    (h >> 24) as u8
}

fn usize_vec_to_u8_vec(v: Vec<usize>) -> Vec<u8> {
    v.into_iter().map(usize_to_u8).collect()
}

fn nested_usize_to_u8(v: Vec<Vec<usize>>) -> Vec<Vec<u8>> {
    v.into_iter().map(usize_vec_to_u8_vec).collect()
}

fn cc_reset_clears_output_position(v: Vec<usize>) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_reset_clears_output_position(usize_vec_to_u8_vec(v)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_trim_all_applies_without_headers(v: Vec<Vec<usize>>) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_trim_all_applies_without_headers(nested_usize_to_u8(v)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_writer_comment_char_auto_quote(v: Vec<usize>) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_writer_comment_char_auto_quote(usize_vec_to_u8_vec(v)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_byte_record_eq_matches_fields(
    (base, sa, sb, trunc): (Vec<usize>, Vec<usize>, Vec<usize>, usize),
) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_byte_record_eq_matches_fields(
        usize_vec_to_u8_vec(base),
        usize_vec_to_u8_vec(sa),
        usize_vec_to_u8_vec(sb),
        usize_to_u8(trunc),
    ) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_comment_only_at_record_start(v: Vec<usize>) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_comment_only_at_record_start(usize_vec_to_u8_vec(v)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_deserialize_byte_buf_accepts_non_utf8(v: Vec<usize>) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_deserialize_byte_buf_accepts_non_utf8(usize_vec_to_u8_vec(v)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let result = match property {
        "ResetClearsOutputPosition" => crabcheck_qc::quickcheck(cc_reset_clears_output_position),
        "TrimAllAppliesWithoutHeaders" => {
            crabcheck_qc::quickcheck(cc_trim_all_applies_without_headers)
        }
        "WriterCommentCharAutoQuote" => {
            crabcheck_qc::quickcheck(cc_writer_comment_char_auto_quote)
        }
        "ByteRecordEqMatchesFields" => {
            crabcheck_qc::quickcheck(cc_byte_record_eq_matches_fields)
        }
        "CommentOnlyAtRecordStart" => crabcheck_qc::quickcheck(cc_comment_only_at_record_start),
        "DeserializeByteBufAcceptsNonUtf8" => {
            crabcheck_qc::quickcheck(cc_deserialize_byte_buf_accepts_non_utf8)
        }
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => Err(format!(
            "crabcheck counterexample: ({})",
            arguments.join(" ")
        )),
        crabcheck_qc::ResultStatus::TimedOut => Err("crabcheck timed out".into()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "crabcheck gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => {
            Err(format!("crabcheck aborted: {error}"))
        }
    };
    (status, metrics)
}

// ───────────── hegel (hegeltest 0.3.7) ─────────────
static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new().test_cases(200).seed(Some(0x0C5F_A7E7))
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "ResetClearsOutputPosition" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let v = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(24));
                if let PropertyResult::Fail(m) = property_reset_clears_output_position(v) {
                    panic!("{m}");
                }
            })
            .settings(settings.clone())
            .run();
        }
        "TrimAllAppliesWithoutHeaders" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let v = tc.draw(
                    hgen::vecs(hgen::vecs(hgen::integers::<u8>()).max_size(6)).max_size(5),
                );
                if let PropertyResult::Fail(m) = property_trim_all_applies_without_headers(v) {
                    panic!("{m}");
                }
            })
            .settings(settings.clone())
            .run();
        }
        "WriterCommentCharAutoQuote" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let v = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(24));
                if let PropertyResult::Fail(m) = property_writer_comment_char_auto_quote(v) {
                    panic!("{m}");
                }
            })
            .settings(settings.clone())
            .run();
        }
        "ByteRecordEqMatchesFields" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let base = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(16));
                let splits_a = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(4));
                let splits_b = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(4));
                let trunc_b = tc.draw(hgen::integers::<u8>());
                if let PropertyResult::Fail(m) =
                    property_byte_record_eq_matches_fields(base, splits_a, splits_b, trunc_b)
                {
                    panic!("{m}");
                }
            })
            .settings(settings.clone())
            .run();
        }
        "CommentOnlyAtRecordStart" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let v = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(24));
                if let PropertyResult::Fail(m) = property_comment_only_at_record_start(v) {
                    panic!("{m}");
                }
            })
            .settings(settings.clone())
            .run();
        }
        "DeserializeByteBufAcceptsNonUtf8" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let v = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(24));
                if let PropertyResult::Fail(m) =
                    property_deserialize_byte_buf_accepts_non_utf8(v)
                {
                    panic!("{m}");
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("__unknown_property:{property}"),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "hegel panicked with non-string payload".to_string()
            };
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {rest}")),
                    Metrics::default(),
                );
            }
            Err(format!("hegel found counterexample: {msg}"))
        }
    };
    (status, metrics)
}

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (Err(format!("Unknown tool: {tool}")), Metrics::default()),
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!(
            "Properties: ResetClearsOutputPosition | TrimAllAppliesWithoutHeaders | WriterCommentCharAutoQuote | ByteRecordEqMatchesFields | CommentOnlyAtRecordStart | DeserializeByteBufAcceptsNonUtf8 | All"
        );
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    // Silence library-under-test panic noise (frameworks catch panics internally
    // but the default hook still prints "thread 'main' panicked at ..." to stderr).
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "panic with non-string payload".to_string()
            };
            emit_json(
                tool,
                property,
                "aborted",
                Metrics::default(),
                None,
                Some(&format!("adapter panic: {msg}")),
            );
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(msg) => emit_json(tool, property, "failed", metrics, Some(&msg), None),
    }
}
