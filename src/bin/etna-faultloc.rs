use std::fmt;

use crabcheck::profiling::quickcheck;
use crabcheck::quickcheck::{Arbitrary, Mutate};
use csv::etna::{
    property_byte_record_eq_matches_fields, property_comment_only_at_record_start,
    property_deserialize_byte_buf_accepts_non_utf8, property_reset_clears_output_position,
    property_trim_all_applies_without_headers, property_writer_comment_char_auto_quote,
    PropertyResult,
};
use rand::Rng;

// Mirror src/bin/etna.rs: Bytes24 = Vec<u8> of 0..24 random bytes,
// Splits5 = Vec<u8> of 0..5, ByteFields = Vec<Vec<u8>> outer 0..5 x inner 0..6.
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
        // Flip a random bit so the byte stays close to the previous value.
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

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        return;
    }
    let result = match (args[1].as_str(), args[2].as_str()) {
        ("crabcheck", "ResetClearsOutputPosition") => quickcheck(|Bytes24(v)| {
            to_opt(property_reset_clears_output_position(v))
        }),
        ("crabcheck", "TrimAllAppliesWithoutHeaders") => quickcheck(|ByteFields(v)| {
            to_opt(property_trim_all_applies_without_headers(v))
        }),
        ("crabcheck", "WriterCommentCharAutoQuote") => quickcheck(
            |(Bytes24(tail), Bytes24(after), ByteVal(comment)): (Bytes24, Bytes24, ByteVal)| {
                to_opt(property_writer_comment_char_auto_quote(tail, after, comment))
            },
        ),
        ("crabcheck", "ByteRecordEqMatchesFields") => {
            quickcheck(
                |(Bytes24(base), (Splits5(sa), Splits5(sb), trunc)): (
                    Bytes24,
                    (Splits5, Splits5, usize),
                )| {
                    {
                        to_opt(property_byte_record_eq_matches_fields(base, sa, sb, trunc as u8))
                    }
                },
            )
        }
        ("crabcheck", "CommentOnlyAtRecordStart") => quickcheck(
            |(Bytes24(first), Bytes24(tail), ByteVal(comment)): (Bytes24, Bytes24, ByteVal)| {
                to_opt(property_comment_only_at_record_start(first, tail, comment))
            },
        ),
        ("crabcheck", "DeserializeByteBufAcceptsNonUtf8") => quickcheck(|Bytes24(v)| {
            to_opt(property_deserialize_byte_buf_accepts_non_utf8(v))
        }),
        (a, b) => panic!("Unknown: {a} {b}"),
    };
    println!("Result: {:?}", result);
}
