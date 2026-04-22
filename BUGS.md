# rust-csv — Injected Bugs

Total mutations: 7

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `byte_record_eq_field_boundaries_efc4a51_1` | `byte_record_eq_field_boundaries` | `src/byte_record.rs` | `patch` | `efc4a51224dd6ccb1b1c4e2254a1ea94b9067b17` |
| 2 | `byte_record_eq_length_check_23fb0cd_1` | `byte_record_eq_length_check` | `src/byte_record.rs` | `patch` | `23fb0cd676bf71c23fc8de45856cbf0187627e45` |
| 3 | `core_reader_comment_only_at_record_start_a5745ba_1` | `core_reader_comment_only_at_record_start` | `csv-core/src/reader.rs` | `patch` | `a5745baa172d50679e34b33d6dba3d063eb40cd4` |
| 4 | `core_reader_reset_output_pos_zero_066de4a_1` | `core_reader_reset_output_pos_zero` | `csv-core/src/reader.rs` | `patch` | `066de4aaf5dbf2c82bdb03fb574d905dde172d8a` |
| 5 | `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | `deserialize_byte_buf_bypasses_utf8` | `src/deserializer.rs` | `patch` | `9e644e66db0aa0b931758de1c2b7da555fb632b7` |
| 6 | `reader_trim_all_without_headers_ce01ae7_1` | `reader_trim_all_without_headers` | `src/reader.rs` | `patch` | `ce01ae7fe4cf7938a22ca565c33678e7422da6c0` |
| 7 | `writer_comment_char_auto_quote_0f64d3f_1` | `writer_comment_char_auto_quote` | `csv-core/src/writer.rs` | `patch` | `0f64d3f3322b30af7a38e222bd7dad18eac38b2b` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `byte_record_eq_field_boundaries_efc4a51_1` | `ByteRecordEqMatchesFields` | `witness_byte_record_eq_matches_fields_case_boundary_shift` |
| `byte_record_eq_length_check_23fb0cd_1` | `ByteRecordEqMatchesFields` | `witness_byte_record_eq_matches_fields_case_length_mismatch` |
| `core_reader_comment_only_at_record_start_a5745ba_1` | `CommentOnlyAtRecordStart` | `witness_comment_only_at_record_start_case_mid_record_hash` |
| `core_reader_reset_output_pos_zero_066de4a_1` | `ResetClearsOutputPosition` | `witness_reset_clears_output_position_case_short_field` |
| `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | `DeserializeByteBufAcceptsNonUtf8` | `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` |
| `reader_trim_all_without_headers_ce01ae7_1` | `TrimAllAppliesWithoutHeaders` | `witness_trim_all_applies_without_headers_case_three_fields` |
| `writer_comment_char_auto_quote_0f64d3f_1` | `WriterCommentCharAutoQuote` | `witness_writer_comment_char_auto_quote_case_hash_prefix` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `ByteRecordEqMatchesFields` | ✓ | ✓ | ✓ | ✓ |
| `CommentOnlyAtRecordStart` | ✓ | ✓ | ✓ | ✓ |
| `ResetClearsOutputPosition` | ✓ | ✓ | ✓ | ✓ |
| `DeserializeByteBufAcceptsNonUtf8` | ✓ | ✓ | ✓ | ✓ |
| `TrimAllAppliesWithoutHeaders` | ✓ | ✓ | ✓ | ✓ |
| `WriterCommentCharAutoQuote` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. byte_record_eq_field_boundaries

- **Variant**: `byte_record_eq_field_boundaries_efc4a51_1`
- **Location**: `src/byte_record.rs`
- **Property**: `ByteRecordEqMatchesFields`
- **Witness(es)**:
  - `witness_byte_record_eq_matches_fields_case_boundary_shift`
- **Source**: csv: fix equality check for raw records
  > `ByteRecord: PartialEq` compared the concatenated byte buffer via `self.as_slice() == other.as_slice()`, which missed field-boundary differences entirely — so `["12","34"] == ["123","4"]` returned `true` (issue #138). The fix zips field-by-field so boundaries are part of equality.
- **Fix commit**: `efc4a51224dd6ccb1b1c4e2254a1ea94b9067b17` — csv: fix equality check for raw records
- **Invariant violated**: Two `ByteRecord`s must only compare equal when their field lists compare equal — the field *boundaries*, not just the concatenated byte buffer, are part of the record's identity.
- **How the mutation triggers**: Reverts the field-wise zip-compare back to `self.as_slice() == other.as_slice()`, which compares concatenated field bytes. The length-check guard from commit 23fb0cd is preserved, so only the boundary regression is exposed; e.g. `["12","34"] == ["123","4"]` incorrectly returns `true`.

### 2. byte_record_eq_length_check

- **Variant**: `byte_record_eq_length_check_23fb0cd_1`
- **Location**: `src/byte_record.rs`
- **Property**: `ByteRecordEqMatchesFields`
- **Witness(es)**:
  - `witness_byte_record_eq_matches_fields_case_length_mismatch`
- **Source**: csv: fix record equality, redux
  > After the field-wise fix (`efc4a51`), `ByteRecord: PartialEq` relied on a `zip` which silently truncated at the shorter iterator, so `["12","34","56"] == ["12","34"]` returned `true`. The fix adds an explicit `if self.len() != other.len() { return false; }` guard before the zip.
- **Fix commit**: `23fb0cd676bf71c23fc8de45856cbf0187627e45` — csv: fix record equality, redux
- **Invariant violated**: Records of different field counts must never compare equal, even when the shorter one is a prefix of the longer.
- **How the mutation triggers**: Removes the `if self.len() != other.len() { return false; }` guard, leaving the zip-based field comparison which silently truncates at the shorter iterator; e.g. `["12","34","56"] == ["12","34"]` incorrectly returns `true`.

### 3. core_reader_comment_only_at_record_start

- **Variant**: `core_reader_comment_only_at_record_start_a5745ba_1`
- **Location**: `csv-core/src/reader.rs`
- **Property**: `CommentOnlyAtRecordStart`
- **Witness(es)**:
  - `witness_comment_only_at_record_start_case_mid_record_hash`
- **Source**: csv-core: fix comment handling
  > The NFA transition for the configured comment character was wired from `StartField`, so any field beginning with the comment byte — not just records starting with it — was treated as a comment (issue #137). Parsing `first,#tail\n` under `comment = Some(b'#')` therefore dropped the second field. The fix moves the transition to `StartRecord`.
- **Fix commit**: `a5745baa172d50679e34b33d6dba3d063eb40cd4` — csv-core: fix comment handling
- **Invariant violated**: The configured comment character only starts a comment at the beginning of a record, not at the beginning of any field.
- **How the mutation triggers**: Moves the comment NFA transition from `StartRecord` back to `StartField`. Parsing `first,#tail\n` with `comment = Some(b'#')` causes the parser to discard the second field as a comment and emit only a single field.

### 4. core_reader_reset_output_pos_zero

- **Variant**: `core_reader_reset_output_pos_zero_066de4a_1`
- **Location**: `csv-core/src/reader.rs`
- **Property**: `ResetClearsOutputPosition`
- **Witness(es)**:
  - `witness_reset_clears_output_position_case_short_field`
- **Source**: csv-core: fix Reader::reset not resetting output_pos
  > `csv_core::Reader::reset` cleared most parser state but forgot to zero `output_pos`, so any partial field parsed before the reset leaked into the next record — `ends[0]` after the reset came out as `stale + field.len()` instead of `field.len()`. The fix adds `self.output_pos = 0`.
- **Fix commit**: `066de4aaf5dbf2c82bdb03fb574d905dde172d8a` — csv-core: fix Reader::reset not resetting output_pos
- **Invariant violated**: After `Reader::reset()`, a subsequent `read_record` into a fresh record must place `ends[0]` at exactly the length of the first field — `reset` should wipe all parser state including the running `output_pos`.
- **How the mutation triggers**: Removes the `self.output_pos = 0;` assignment from `Reader::reset`. After priming the reader (a complete record followed by a partial field with no terminator advances `output_pos`), `reset()` leaves the stale offset; the next complete parse reports `ends[0] = stale_output_pos + field.len()` instead of `field.len()`.

### 5. deserialize_byte_buf_bypasses_utf8

- **Variant**: `deserialize_byte_buf_bypasses_utf8_9e644e6_1`
- **Location**: `src/deserializer.rs`
- **Property**: `DeserializeByteBufAcceptsNonUtf8`
- **Witness(es)**:
  - `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle`
- **Source**: serde: fix bug in handling of invalid UTF-8
  > The serde deserializer's `deserialize_byte_buf` was implemented via `next_field()`, which performs UTF-8 validation, so `#[serde(with = "serde_bytes")]` fields rejected perfectly valid raw-byte payloads with a decode error. The fix routes through `next_field_bytes()` instead, honoring the byte-buffer contract.
- **Fix commit**: `9e644e66db0aa0b931758de1c2b7da555fb632b7` — serde: fix bug in handling of invalid UTF-8
- **Invariant violated**: A struct field annotated with `#[serde(with = "serde_bytes")]` takes arbitrary bytes — deserializing into it must not perform UTF-8 validation on the source field.
- **How the mutation triggers**: Reroutes `deserialize_byte_buf` through `next_field()` (which runs UTF-8 validation) instead of `next_field_bytes()` (which returns raw bytes). Deserializing a record whose middle field contains invalid UTF-8 now returns a decode error.

### 6. reader_trim_all_without_headers

- **Variant**: `reader_trim_all_without_headers_ce01ae7_1`
- **Location**: `src/reader.rs`
- **Property**: `TrimAllAppliesWithoutHeaders`
- **Witness(es)**:
  - `witness_trim_all_applies_without_headers_case_three_fields`
- **Source**: reader: tweak record trimming logic
  > The trim block was nested inside the `has_headers(true)` branch, so `ReaderBuilder::has_headers(false).trim(Trim::All)` silently skipped trimming and returned records with their surrounding whitespace intact (issue #237). The fix lifts the `record.trim()` call out to an unconditional sibling branch.
- **Fix commit**: `ce01ae7fe4cf7938a22ca565c33678e7422da6c0` — reader: tweak record trimming logic
- **Invariant violated**: `ReaderBuilder::has_headers(false).trim(Trim::All)` must trim whitespace off every field of every record, not just records read after the header branch.
- **How the mutation triggers**: Reverts the trim block to the buggy `} else if self.state.trim.should_trim_fields() { record.trim(); }` so trimming only runs inside the `!has_headers` branch path. With `has_headers(false)` and `Trim::All`, the record retains its surrounding whitespace.

### 7. writer_comment_char_auto_quote

- **Variant**: `writer_comment_char_auto_quote_0f64d3f_1`
- **Location**: `csv-core/src/writer.rs`
- **Property**: `WriterCommentCharAutoQuote`
- **Witness(es)**:
  - `witness_writer_comment_char_auto_quote_case_hash_prefix`
- **Source**: api: automatically escape fields that contain the comment character
  > When the writer was configured with `comment(Some(c))` and `QuoteStyle::Necessary`, a field beginning with `c` was serialized unquoted, so reading it back under the same comment character silently dropped the row as a comment line (issue #283). The fix marks the comment byte as requires-quotes when the writer is built.
- **Fix commit**: `0f64d3f3322b30af7a38e222bd7dad18eac38b2b` — api: automatically escape fields that contain the comment character
- **Invariant violated**: A field written with `WriterBuilder::comment(Some(c))` and `QuoteStyle::Necessary` must round-trip through a reader configured with the same comment character. When the field starts with `c`, the writer must auto-quote it.
- **How the mutation triggers**: Removes the `if let Some(comment) = self.wtr.comment { wtr.requires_quotes[comment as usize] = true; }` block from `WriterBuilder::build`. Fields beginning with `#` serialize unquoted; on read-back with `comment = Some(b'#')`, the row is dropped as a comment line.
