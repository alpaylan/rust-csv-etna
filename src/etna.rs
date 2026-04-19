//! ETNA framework-neutral property functions for rust-csv.
//!
//! Each `property_<name>` is a pure function taking concrete, owned inputs and
//! returning `PropertyResult`. Framework adapters (proptest/quickcheck/crabcheck/hegel)
//! in `src/bin/etna.rs` and deterministic witness tests in `tests/etna_witnesses.rs`
//! both call these functions directly — there is no re-implementation of the
//! invariant inside any adapter.

#![allow(missing_docs)]

use crate::{ByteRecord, ReaderBuilder, Trim, WriterBuilder};

pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

fn normalize_field(src: &[u8]) -> Vec<u8> {
    src.iter()
        .copied()
        .filter(|&b| {
            b.is_ascii_alphanumeric()
                || b == b' '
                || b == b'_'
                || b == b'-'
                || b == b'.'
                || b == b'@'
        })
        .take(24)
        .collect()
}

// ──────────────────────────────────────────────────────────────────────────
// Property 1: csv_core::Reader::reset clears output_pos.
//
// Regression for 066de4a — `Reader::reset` was missing `self.output_pos = 0;`.
// Priming the reader (so `output_pos` advances), calling reset, then parsing a
// known single-record payload must produce `ends[0] == len(first field)`.
// Under the bug, `ends[0]` is the stale `output_pos` plus the field length.
// ──────────────────────────────────────────────────────────────────────────
pub fn property_reset_clears_output_position(payload: Vec<u8>) -> PropertyResult {
    let field = normalize_field(&payload);
    if field.is_empty() {
        return PropertyResult::Discard;
    }
    let mut csv = field.clone();
    csv.extend_from_slice(b",z\n");

    let mut r = csv_core::Reader::new();
    let mut trash_out = [0u8; 256];
    let mut trash_ends = [0usize; 16];
    // Priming: read a full record to consume its bytes, then a partial tail
    // with no terminator. This leaves `output_pos` advanced (the parser wrote
    // the partial bytes into `out` and is awaiting more input). After reset(),
    // `output_pos` must go back to 0 — if the bug is present, ends[0] below
    // picks up the stale offset.
    let _ = r.read_record(b"foo,bar\n", &mut trash_out, &mut trash_ends);
    let _ = r.read_record(b"baz", &mut trash_out, &mut trash_ends);
    r.reset();

    let mut out = [0u8; 512];
    let mut ends = [0usize; 16];
    let (res, _nin, _nout, nend) = r.read_record(&csv, &mut out, &mut ends);
    if res != csv_core::ReadRecordResult::Record {
        return PropertyResult::Discard;
    }
    if nend < 1 {
        return PropertyResult::Fail(format!("nend={}, expected >= 1", nend));
    }
    if ends[0] != field.len() {
        return PropertyResult::Fail(format!(
            "ends[0]={} expected={} (stale output_pos not reset)",
            ends[0],
            field.len()
        ));
    }
    PropertyResult::Pass
}

// ──────────────────────────────────────────────────────────────────────────
// Property 2: `Trim::All` trims fields even when `has_headers(false)`.
//
// Regression for ce01ae7 — the trim branch was gated behind an `else` arm that
// only ran when the reader was already past header setup, so `has_headers(false)`
// with `Trim::All` left fields padded with whitespace.
// ──────────────────────────────────────────────────────────────────────────
pub fn property_trim_all_applies_without_headers(fields: Vec<Vec<u8>>) -> PropertyResult {
    if fields.is_empty() || fields.len() > 6 {
        return PropertyResult::Discard;
    }
    let cleaned: Vec<Vec<u8>> = fields.iter().map(|f| normalize_field(f)).collect();
    for f in &cleaned {
        // Trim::All strips leading/trailing whitespace from every field; skip
        // fields whose expected value would itself change under trim so the
        // equality check below stays meaningful.
        if f.is_empty()
            || f.first().map_or(true, |b| b.is_ascii_whitespace())
            || f.last().map_or(true, |b| b.is_ascii_whitespace())
        {
            return PropertyResult::Discard;
        }
    }

    let mut data = Vec::new();
    for (i, f) in cleaned.iter().enumerate() {
        if i > 0 {
            data.push(b',');
        }
        data.push(b' ');
        data.extend_from_slice(f);
        data.push(b'\t');
    }
    data.push(b'\n');

    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .from_reader(data.as_slice());
    let mut rec = ByteRecord::new();
    let ok = match rdr.read_byte_record(&mut rec) {
        Ok(v) => v,
        Err(e) => return PropertyResult::Fail(format!("read error: {e}")),
    };
    if !ok {
        return PropertyResult::Fail("no record read".into());
    }
    if rec.len() != cleaned.len() {
        return PropertyResult::Fail(format!(
            "expected {} fields, got {}",
            cleaned.len(),
            rec.len()
        ));
    }
    for (i, expected) in cleaned.iter().enumerate() {
        if rec.get(i) != Some(&expected[..]) {
            return PropertyResult::Fail(format!(
                "field {i}: got {:?}, expected {:?}",
                rec.get(i),
                expected
            ));
        }
    }
    PropertyResult::Pass
}

// ──────────────────────────────────────────────────────────────────────────
// Property 3: Writer auto-quotes fields containing the comment character.
//
// Regression for 0f64d3f — without the `requires_quotes[comment] = true` entry
// in `WriterBuilder::build`, a field starting with `#` serialized unquoted and
// round-tripped as a comment (lost).
// ──────────────────────────────────────────────────────────────────────────
pub fn property_writer_comment_char_auto_quote(tail: Vec<u8>) -> PropertyResult {
    let tail = normalize_field(&tail);
    // Build a field `#...` that starts with the comment char.
    let mut field = vec![b'#'];
    field.extend_from_slice(&tail);

    let mut wtr = WriterBuilder::new()
        .comment(Some(b'#'))
        .from_writer(Vec::new());
    if let Err(e) = wtr.write_record(&[&field[..], b"after".as_slice()]) {
        return PropertyResult::Fail(format!("write error: {e}"));
    }
    let buf = match wtr.into_inner() {
        Ok(b) => b,
        Err(e) => return PropertyResult::Fail(format!("flush error: {e}")),
    };

    // Round-trip: read back with the same comment char configured. If the field
    // was not quoted, the reader treats the `#...` row as a comment and skips it.
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(buf.as_slice());
    let mut rec = ByteRecord::new();
    let ok = match rdr.read_byte_record(&mut rec) {
        Ok(v) => v,
        Err(e) => {
            return PropertyResult::Fail(format!("read error: {e} (raw={:?})", buf));
        }
    };
    if !ok {
        return PropertyResult::Fail(format!(
            "no record read back (raw={:?}) — field starting with '#' was not quoted",
            buf
        ));
    }
    if rec.len() != 2 {
        return PropertyResult::Fail(format!(
            "expected 2 fields, got {} (raw={:?})",
            rec.len(),
            buf
        ));
    }
    if rec.get(0) != Some(&field[..]) {
        return PropertyResult::Fail(format!(
            "round-trip mismatch on field 0: {:?} vs {:?}",
            rec.get(0),
            field
        ));
    }
    PropertyResult::Pass
}

// ──────────────────────────────────────────────────────────────────────────
// Property 4/5: ByteRecord equality respects field boundaries and length.
//
// Regression for efc4a51 (field boundaries) and 23fb0cd (length check). Input
// is shaped so random PBT can reach both bug patterns:
//   left  = split(base, splits_a)
//   right = split(base[..base.len()-trunc_b], splits_b)
//
// Shared `base` means `r1.as_slice() == r2.as_slice()` is common, which
// exercises the boundary bug when splits differ. Non-zero `trunc_b` yields
// records of differing lengths that still share a common prefix of fields,
// which exercises the length-guard bug.
//
// Invariant: `r1 == r2  ⇔  fields(r1) == fields(r2)` (vec equality).
// ──────────────────────────────────────────────────────────────────────────
fn split_bytes(bytes: &[u8], splits: &[u8]) -> Vec<Vec<u8>> {
    if bytes.is_empty() {
        return vec![Vec::new()];
    }
    let mut positions: Vec<usize> = splits
        .iter()
        .take(4)
        .map(|&s| (s as usize) % (bytes.len() + 1))
        .filter(|&p| p > 0 && p < bytes.len())
        .collect();
    positions.sort();
    positions.dedup();
    let mut out = Vec::with_capacity(positions.len() + 1);
    let mut prev = 0usize;
    for p in &positions {
        out.push(bytes[prev..*p].to_vec());
        prev = *p;
    }
    out.push(bytes[prev..].to_vec());
    out
}

pub fn property_byte_record_eq_matches_fields(
    base: Vec<u8>,
    splits_a: Vec<u8>,
    splits_b: Vec<u8>,
    trunc_b: u8,
) -> PropertyResult {
    if base.len() > 16 {
        return PropertyResult::Discard;
    }
    let base_a = &base[..];
    let trunc = (trunc_b as usize) % (base.len() + 1);
    let base_b = &base[..base.len() - trunc];
    let left = split_bytes(base_a, &splits_a);
    let right = split_bytes(base_b, &splits_b);
    let r1 = ByteRecord::from(left.clone());
    let r2 = ByteRecord::from(right.clone());
    let expected = left == right;
    let actual = r1 == r2;
    if expected != actual {
        return PropertyResult::Fail(format!(
            "base={:?} splits_a={:?} splits_b={:?} trunc_b={} left={:?} right={:?} expected_eq={} got_eq={}",
            base, splits_a, splits_b, trunc_b, left, right, expected, actual
        ));
    }
    PropertyResult::Pass
}

// ──────────────────────────────────────────────────────────────────────────
// Property 6: Comment character is only honored at the start of a record.
//
// Regression for a5745ba — the NFA transition for the comment character was
// present in `StartField`, so a field starting with `#` (mid-record) was also
// treated as a comment. The fix moved the transition to `StartRecord`.
// ──────────────────────────────────────────────────────────────────────────
pub fn property_comment_only_at_record_start(tail: Vec<u8>) -> PropertyResult {
    let comment: u8 = b'#';
    let tail = normalize_field(&tail);

    // Build `first,#<tail>\n` — the second field starts with `#` but that's
    // mid-record, so `#` must be treated as a normal byte.
    let mut second = vec![comment];
    second.extend_from_slice(&tail);
    let mut data = b"first,".to_vec();
    data.extend_from_slice(&second);
    data.push(b'\n');

    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .comment(Some(comment))
        .from_reader(data.as_slice());
    let mut rec = ByteRecord::new();
    let ok = match rdr.read_byte_record(&mut rec) {
        Ok(v) => v,
        Err(e) => return PropertyResult::Fail(format!("read error: {e}")),
    };
    if !ok {
        return PropertyResult::Fail(format!(
            "no record read (data={:?})",
            data
        ));
    }
    if rec.len() != 2 {
        return PropertyResult::Fail(format!(
            "expected 2 fields, got {} (data={:?})",
            rec.len(),
            data
        ));
    }
    if rec.get(0) != Some(b"first".as_slice()) {
        return PropertyResult::Fail(format!(
            "field 0 mismatch: {:?}",
            rec.get(0)
        ));
    }
    if rec.get(1) != Some(second.as_slice()) {
        return PropertyResult::Fail(format!(
            "field 1 mismatch: {:?} vs expected {:?}",
            rec.get(1),
            second
        ));
    }
    PropertyResult::Pass
}

// ──────────────────────────────────────────────────────────────────────────
// Property 7: Deserializing a ByteRecord into a `#[serde(with = "serde_bytes")]`
// field must not run UTF-8 validation on that field.
//
// Regression for 9e644e6 — `deserialize_byte_buf` went through `next_field()`
// (UTF-8 validated) instead of `next_field_bytes()`.
// ──────────────────────────────────────────────────────────────────────────
pub fn property_deserialize_byte_buf_accepts_non_utf8(h2_bytes: Vec<u8>) -> PropertyResult {
    // The bytes can be arbitrary — including invalid UTF-8. We skip cases that
    // happen to collide with CSV special bytes; construction from `ByteRecord`
    // bypasses the parser so this is actually unnecessary, but keeps the input
    // space smaller.
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Row {
        h1: String,
        #[serde(with = "serde_bytes")]
        h2: Vec<u8>,
        h3: String,
    }
    let headers = ByteRecord::from(vec![b"h1".as_slice(), b"h2", b"h3"]);
    let record = ByteRecord::from(vec![
        b"baz".as_slice(),
        h2_bytes.as_slice(),
        b"quux".as_slice(),
    ]);
    match record.deserialize::<Row>(Some(&headers)) {
        Ok(row) => {
            if row.h1 == "baz" && row.h2 == h2_bytes && row.h3 == "quux" {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!("round-trip mismatch: {:?}", row))
            }
        }
        Err(e) => PropertyResult::Fail(format!(
            "deserialize rejected non-UTF-8 byte buf: {e} (bytes={:?})",
            h2_bytes
        )),
    }
}
