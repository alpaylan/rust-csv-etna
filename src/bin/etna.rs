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
use crabcheck::quickcheck::Arbitrary as CcArbitrary;
use csv::etna::{
    property_byte_record_eq_matches_fields, property_comment_only_at_record_start,
    property_deserialize_byte_buf_accepts_non_utf8, property_reset_clears_output_position,
    property_trim_all_applies_without_headers, property_writer_comment_char_auto_quote,
    PropertyResult,
};
use hegel::{generators as hgen, Hegel, Settings as HegelSettings};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestError, TestRunner};
use quickcheck::{Arbitrary as QcArbitrary, Gen, QuickCheck, ResultStatus, TestResult};
use rand::Rng;
use std::fmt;
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
            to_err(property_writer_comment_char_auto_quote(
                b" comment".to_vec(),
                b"after".to_vec(),
                0u8,
            ))
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
            to_err(property_comment_only_at_record_start(
                b"first".to_vec(),
                b"bar".to_vec(),
                0u8,
            ))
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

// ───────────── shared generators ─────────────
//
// Match proptest shapes exactly:
//   Bytes24:    vec(any::<u8>(), 0..24)            → len 0..=23
//   Splits5:    vec(any::<u8>(), 0..5)             → len 0..=4
//   ByteFields: vec(vec(any::<u8>(), 0..6), 0..5)  → outer 0..=4, inner 0..=5

#[derive(Clone)]
struct Bytes24(Vec<u8>);

#[derive(Clone)]
struct Splits5(Vec<u8>);

#[derive(Clone)]
struct ByteFields(Vec<Vec<u8>>);

macro_rules! impl_debug_display_delegate {
    ($ty:ty) => {
        impl fmt::Debug for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }
        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }
    };
}

impl_debug_display_delegate!(Bytes24);
impl_debug_display_delegate!(Splits5);
impl_debug_display_delegate!(ByteFields);

fn gen_bytes_qc(g: &mut Gen, max_len_exclusive: u32) -> Vec<u8> {
    let len = g.random_range(0..max_len_exclusive) as usize;
    let mut v = Vec::with_capacity(len);
    for _ in 0..len {
        v.push(g.random_range(0..=u8::MAX));
    }
    v
}

fn gen_bytes_cc<R: Rng>(rng: &mut R, max_len_exclusive: u32) -> Vec<u8> {
    let len = rng.random_range(0..max_len_exclusive) as usize;
    let mut v = Vec::with_capacity(len);
    for _ in 0..len {
        v.push(rng.random_range(0..=u8::MAX));
    }
    v
}

impl QcArbitrary for Bytes24 {
    fn arbitrary(g: &mut Gen) -> Self {
        Bytes24(gen_bytes_qc(g, 24))
    }
}
impl<R: Rng> CcArbitrary<R> for Bytes24 {
    fn generate(rng: &mut R, _n: usize) -> Self {
        Bytes24(gen_bytes_cc(rng, 24))
    }
}

impl QcArbitrary for Splits5 {
    fn arbitrary(g: &mut Gen) -> Self {
        Splits5(gen_bytes_qc(g, 5))
    }
}
impl<R: Rng> CcArbitrary<R> for Splits5 {
    fn generate(rng: &mut R, _n: usize) -> Self {
        Splits5(gen_bytes_cc(rng, 5))
    }
}

impl QcArbitrary for ByteFields {
    fn arbitrary(g: &mut Gen) -> Self {
        let outer = g.random_range(0..5u32) as usize;
        let mut out = Vec::with_capacity(outer);
        for _ in 0..outer {
            out.push(gen_bytes_qc(g, 6));
        }
        ByteFields(out)
    }
}
impl<R: Rng> CcArbitrary<R> for ByteFields {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let outer = rng.random_range(0..5u32) as usize;
        let mut out = Vec::with_capacity(outer);
        for _ in 0..outer {
            out.push(gen_bytes_cc(rng, 6));
        }
        ByteFields(out)
    }
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
    let mut runner = TestRunner::new(ProptestConfig { cases: 40_000_000, ..ProptestConfig::default() });
    let result: Result<(), String> = match property {
        "ResetClearsOutputPosition" => {
            let c = counter.clone();
            runner
                .run(&bytes_strategy(), move |v| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let v_cex = v.clone();
                    match property_reset_clears_output_position(v) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?})", v_cex))),
                    }
                })
                .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() })
        }
        "TrimAllAppliesWithoutHeaders" => {
            let c = counter.clone();
            runner
                .run(&vec_bytes_strategy(), move |v| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let v_cex = v.clone();
                    match property_trim_all_applies_without_headers(v) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?})", v_cex))),
                    }
                })
                .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() })
        }
        "WriterCommentCharAutoQuote" => {
            let c = counter.clone();
            runner
                .run(
                    &(bytes_strategy(), bytes_strategy(), any::<u8>()),
                    move |(tail, after, comment)| {
                        c.fetch_add(1, Ordering::Relaxed);
                        let tail_cex = tail.clone();
                        let after_cex = after.clone();
                        match property_writer_comment_char_auto_quote(tail, after, comment) {
                            PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                            PropertyResult::Fail(_) => Err(TestCaseError::fail(format!(
                                "({:?} {:?} {})",
                                tail_cex, after_cex, comment
                            ))),
                        }
                    },
                )
                .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() })
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
                        let base_cex = base.clone();
                        let sa_cex = sa.clone();
                        let sb_cex = sb.clone();
                        match property_byte_record_eq_matches_fields(base, sa, sb, trunc) {
                            PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                            PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?} {:?} {:?} {})", base_cex, sa_cex, sb_cex, trunc))),
                        }
                    },
                )
                .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() })
        }
        "CommentOnlyAtRecordStart" => {
            let c = counter.clone();
            runner
                .run(
                    &(bytes_strategy(), bytes_strategy(), any::<u8>()),
                    move |(first, tail, comment)| {
                        c.fetch_add(1, Ordering::Relaxed);
                        let first_cex = first.clone();
                        let tail_cex = tail.clone();
                        match property_comment_only_at_record_start(first, tail, comment) {
                            PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                            PropertyResult::Fail(_) => Err(TestCaseError::fail(format!(
                                "({:?} {:?} {})",
                                first_cex, tail_cex, comment
                            ))),
                        }
                    },
                )
                .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() })
        }
        "DeserializeByteBufAcceptsNonUtf8" => {
            let c = counter.clone();
            runner
                .run(&bytes_strategy(), move |v| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let v_cex = v.clone();
                    match property_deserialize_byte_buf_accepts_non_utf8(v) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?})", v_cex))),
                    }
                })
                .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() })
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
static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_reset_clears_output_position(Bytes24(v): Bytes24) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_reset_clears_output_position(v) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_trim_all_applies_without_headers(ByteFields(v): ByteFields) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_trim_all_applies_without_headers(v) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_writer_comment_char_auto_quote(
    Bytes24(tail): Bytes24,
    Bytes24(after): Bytes24,
    comment: u8,
) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_writer_comment_char_auto_quote(tail, after, comment) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_byte_record_eq_matches_fields(
    Bytes24(base): Bytes24,
    Splits5(sa): Splits5,
    Splits5(sb): Splits5,
    trunc: u8,
) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_byte_record_eq_matches_fields(base, sa, sb, trunc) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_comment_only_at_record_start(
    Bytes24(first): Bytes24,
    Bytes24(tail): Bytes24,
    comment: u8,
) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_comment_only_at_record_start(first, tail, comment) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_deserialize_byte_buf_accepts_non_utf8(Bytes24(v): Bytes24) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_deserialize_byte_buf_accepts_non_utf8(v) {
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
    let mut qc = QuickCheck::new().tests(40_000_000).max_tests(80_000_000);
    let result = match property {
        "ResetClearsOutputPosition" => {
            qc.quicktest(qc_reset_clears_output_position as fn(Bytes24) -> TestResult)
        }
        "TrimAllAppliesWithoutHeaders" => {
            qc.quicktest(qc_trim_all_applies_without_headers as fn(ByteFields) -> TestResult)
        }
        "WriterCommentCharAutoQuote" => qc.quicktest(
            qc_writer_comment_char_auto_quote as fn(Bytes24, Bytes24, u8) -> TestResult,
        ),
        "ByteRecordEqMatchesFields" => qc.quicktest(
            qc_byte_record_eq_matches_fields as fn(Bytes24, Splits5, Splits5, u8) -> TestResult,
        ),
        "CommentOnlyAtRecordStart" => qc.quicktest(
            qc_comment_only_at_record_start as fn(Bytes24, Bytes24, u8) -> TestResult,
        ),
        "DeserializeByteBufAcceptsNonUtf8" => qc.quicktest(
            qc_deserialize_byte_buf_accepts_non_utf8 as fn(Bytes24) -> TestResult,
        ),
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
            "({})",
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

fn cc_reset_clears_output_position(Bytes24(v): Bytes24) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_reset_clears_output_position(v) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_trim_all_applies_without_headers(ByteFields(v): ByteFields) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_trim_all_applies_without_headers(v) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_writer_comment_char_auto_quote(
    (Bytes24(tail), Bytes24(after), comment): (Bytes24, Bytes24, u8),
) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_writer_comment_char_auto_quote(tail, after, comment) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_byte_record_eq_matches_fields(
    (Bytes24(base), Splits5(sa), Splits5(sb), trunc): (Bytes24, Splits5, Splits5, u8),
) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_byte_record_eq_matches_fields(base, sa, sb, trunc) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_comment_only_at_record_start(
    (Bytes24(first), Bytes24(tail), comment): (Bytes24, Bytes24, u8),
) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_comment_only_at_record_start(first, tail, comment) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_deserialize_byte_buf_accepts_non_utf8(Bytes24(v): Bytes24) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_deserialize_byte_buf_accepts_non_utf8(v) {
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
        crabcheck_qc::ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        },
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
    HegelSettings::new().test_cases(40_000_000)
}

// Hegel's max_size is inclusive, proptest's 0..N is exclusive,
// so proptest 0..24 corresponds to max_size(23), etc.
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
                let v = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(23));
                let v_cex = v.clone();
                if let PropertyResult::Fail(_) = property_reset_clears_output_position(v) {
                    panic!("({:?})", v_cex);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "TrimAllAppliesWithoutHeaders" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let v = tc.draw(
                    hgen::vecs(hgen::vecs(hgen::integers::<u8>()).max_size(5)).max_size(4),
                );
                let v_cex = v.clone();
                if let PropertyResult::Fail(_) = property_trim_all_applies_without_headers(v) {
                    panic!("({:?})", v_cex);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "WriterCommentCharAutoQuote" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let tail = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(23));
                let after = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(23));
                let comment = tc.draw(hgen::integers::<u8>());
                let tail_cex = tail.clone();
                let after_cex = after.clone();
                if let PropertyResult::Fail(_) =
                    property_writer_comment_char_auto_quote(tail, after, comment)
                {
                    panic!("({:?} {:?} {})", tail_cex, after_cex, comment);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "ByteRecordEqMatchesFields" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let base = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(23));
                let splits_a = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(4));
                let splits_b = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(4));
                let trunc_b = tc.draw(hgen::integers::<u8>());
                let base_cex = base.clone();
                let splits_a_cex = splits_a.clone();
                let splits_b_cex = splits_b.clone();
                if let PropertyResult::Fail(_) =
                    property_byte_record_eq_matches_fields(base, splits_a, splits_b, trunc_b)
                {
                    panic!("({:?} {:?} {:?} {})", base_cex, splits_a_cex, splits_b_cex, trunc_b);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "CommentOnlyAtRecordStart" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let first = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(23));
                let tail = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(23));
                let comment = tc.draw(hgen::integers::<u8>());
                let first_cex = first.clone();
                let tail_cex = tail.clone();
                if let PropertyResult::Fail(_) =
                    property_comment_only_at_record_start(first, tail, comment)
                {
                    panic!("({:?} {:?} {})", first_cex, tail_cex, comment);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "DeserializeByteBufAcceptsNonUtf8" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let v = tc.draw(hgen::vecs(hgen::integers::<u8>()).max_size(23));
                let v_cex = v.clone();
                if let PropertyResult::Fail(_) =
                    property_deserialize_byte_buf_accepts_non_utf8(v)
                {
                    panic!("({:?})", v_cex);
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
            Err(msg.strip_prefix("Property test failed: ").unwrap_or(&msg).to_string())
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
