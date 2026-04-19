# rust-csv — Injected Bugs

Total mutations: 7

All variants are patch-based; apply the listed patch to a clean HEAD to reproduce the buggy build. Each `etna/<variant>` branch is a pre-applied snapshot.

## Bug Index

| # | Name | Variant | File | Injection | Fix Commit |
|---|------|---------|------|-----------|------------|
| 1 | `csv_core::Reader::reset` leaves `output_pos` stale | `core_reader_reset_output_pos_zero_066de4a_1` | `patches/core_reader_reset_output_pos_zero_066de4a_1.patch` | patch | `066de4aaf5dbf2c82bdb03fb574d905dde172d8a` |
| 2 | `Trim::All` skipped when `has_headers(false)` | `reader_trim_all_without_headers_ce01ae7_1` | `patches/reader_trim_all_without_headers_ce01ae7_1.patch` | patch | `ce01ae7fe4cf7938a22ca565c33678e7422da6c0` |
| 3 | `Writer` does not auto-quote fields starting with comment char | `writer_comment_char_auto_quote_0f64d3f_1` | `patches/writer_comment_char_auto_quote_0f64d3f_1.patch` | patch | `0f64d3f3322b30af7a38e222bd7dad18eac38b2b` |
| 4 | `ByteRecord::eq` ignores field boundaries | `byte_record_eq_field_boundaries_efc4a51_1` | `patches/byte_record_eq_field_boundaries_efc4a51_1.patch` | patch | `efc4a51224dd6ccb1b1c4e2254a1ea94b9067b17` |
| 5 | `ByteRecord::eq` skips length check | `byte_record_eq_length_check_23fb0cd_1` | `patches/byte_record_eq_length_check_23fb0cd_1.patch` | patch | `23fb0cd676bf71c23fc8de45856cbf0187627e45` |
| 6 | Comment character honored mid-record | `core_reader_comment_only_at_record_start_a5745ba_1` | `patches/core_reader_comment_only_at_record_start_a5745ba_1.patch` | patch | `a5745baa172d50679e34b33d6dba3d063eb40cd4` |
| 7 | `deserialize_byte_buf` UTF-8-validates raw bytes | `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | `patches/deserialize_byte_buf_bypasses_utf8_9e644e6_1.patch` | patch | `9e644e66db0aa0b931758de1c2b7da555fb632b7` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `core_reader_reset_output_pos_zero_066de4a_1` | `property_reset_clears_output_position` | `witness_reset_clears_output_position_case_short_field` |
| `reader_trim_all_without_headers_ce01ae7_1` | `property_trim_all_applies_without_headers` | `witness_trim_all_applies_without_headers_case_three_fields` |
| `writer_comment_char_auto_quote_0f64d3f_1` | `property_writer_comment_char_auto_quote` | `witness_writer_comment_char_auto_quote_case_hash_prefix` |
| `byte_record_eq_field_boundaries_efc4a51_1` | `property_byte_record_eq_matches_fields` | `witness_byte_record_eq_matches_fields_case_boundary_shift` |
| `byte_record_eq_length_check_23fb0cd_1` | `property_byte_record_eq_matches_fields` | `witness_byte_record_eq_matches_fields_case_length_mismatch` |
| `core_reader_comment_only_at_record_start_a5745ba_1` | `property_comment_only_at_record_start` | `witness_comment_only_at_record_start_case_mid_record_hash` |
| `deserialize_byte_buf_bypasses_utf8_9e644e6_1` | `property_deserialize_byte_buf_accepts_non_utf8` | `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle` |

Note: bugs 4 and 5 share a single `property_byte_record_eq_matches_fields` — a single invariant (`r1 == r2 ⇔ fields(r1) == fields(r2)`) covers both the boundary regression and the length regression.

## Framework Coverage

| Property | etna | proptest | quickcheck | crabcheck | hegel |
|----------|:----:|:--------:|:----------:|:---------:|:-----:|
| `property_reset_clears_output_position` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_trim_all_applies_without_headers` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_writer_comment_char_auto_quote` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_byte_record_eq_matches_fields` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_comment_only_at_record_start` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_deserialize_byte_buf_accepts_non_utf8` | ✓ | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. `csv_core::Reader::reset` leaves `output_pos` stale

- **Variant**: `core_reader_reset_output_pos_zero_066de4a_1`
- **Location**: `patches/core_reader_reset_output_pos_zero_066de4a_1.patch` (applies to `csv-core/src/reader.rs`)
- **Property**: `property_reset_clears_output_position`
- **Witness**: `witness_reset_clears_output_position_case_short_field`
- **Fix commit**: `066de4aaf5dbf2c82bdb03fb574d905dde172d8a` — `csv-core: fix Reader::reset not resetting output_pos`
- **Invariant violated**: After `Reader::reset()`, a subsequent `read_record` into a fresh record must place `ends[0]` at exactly the length of the first field — `reset` should wipe all parser state including the running `output_pos`.
- **How the mutation triggers**: Removes the `self.output_pos = 0;` assignment from `Reader::reset`. After priming the reader (a complete record followed by a partial field with no terminator advances `output_pos`), `reset()` leaves the stale offset; the next complete parse reports `ends[0] = stale_output_pos + field.len()` instead of `field.len()`.

### 2. `Trim::All` skipped when `has_headers(false)`

- **Variant**: `reader_trim_all_without_headers_ce01ae7_1`
- **Location**: `patches/reader_trim_all_without_headers_ce01ae7_1.patch` (applies to `src/reader.rs`)
- **Property**: `property_trim_all_applies_without_headers`
- **Witness**: `witness_trim_all_applies_without_headers_case_three_fields`
- **Fix commit**: `ce01ae7fe4cf7938a22ca565c33678e7422da6c0` — `reader: tweak record trimming logic` (fixes #237)
- **Invariant violated**: `ReaderBuilder::has_headers(false).trim(Trim::All)` must trim whitespace off every field of every record, not just records read after the header branch.
- **How the mutation triggers**: Reverts the trim block to the buggy `} else if self.state.trim.should_trim_fields() { record.trim(); }` so trimming only runs inside the `!has_headers` branch path. With `has_headers(false)` and `Trim::All`, the record retains its surrounding whitespace.

### 3. `Writer` does not auto-quote fields starting with comment char

- **Variant**: `writer_comment_char_auto_quote_0f64d3f_1`
- **Location**: `patches/writer_comment_char_auto_quote_0f64d3f_1.patch` (applies to `csv-core/src/writer.rs`)
- **Property**: `property_writer_comment_char_auto_quote`
- **Witness**: `witness_writer_comment_char_auto_quote_case_hash_prefix`
- **Fix commit**: `0f64d3f3322b30af7a38e222bd7dad18eac38b2b` — `api: automatically escape fields that contain the comment character` (closes #283)
- **Invariant violated**: A field written with `WriterBuilder::comment(Some(c))` and `QuoteStyle::Necessary` must round-trip through a reader configured with the same comment character. When the field starts with `c`, the writer must auto-quote it.
- **How the mutation triggers**: Removes the `if let Some(comment) = self.wtr.comment { wtr.requires_quotes[comment as usize] = true; }` block from `WriterBuilder::build`. Fields beginning with `#` serialize unquoted; on read-back with `comment = Some(b'#')`, the row is dropped as a comment line.

### 4. `ByteRecord::eq` ignores field boundaries

- **Variant**: `byte_record_eq_field_boundaries_efc4a51_1`
- **Location**: `patches/byte_record_eq_field_boundaries_efc4a51_1.patch` (applies to `src/byte_record.rs`)
- **Property**: `property_byte_record_eq_matches_fields`
- **Witness**: `witness_byte_record_eq_matches_fields_case_boundary_shift`
- **Fix commit**: `efc4a51224dd6ccb1b1c4e2254a1ea94b9067b17` — `csv: fix equality check for raw records` (fixes #138)
- **Invariant violated**: Two `ByteRecord`s must only compare equal when their field lists compare equal — the field *boundaries*, not just the concatenated byte buffer, are part of the record's identity.
- **How the mutation triggers**: Reverts the field-wise zip-compare back to `self.as_slice() == other.as_slice()`, which compares concatenated field bytes. The length-check guard from commit 23fb0cd is preserved, so only the boundary regression is exposed; e.g. `["12","34"] == ["123","4"]` incorrectly returns `true`.

### 5. `ByteRecord::eq` skips length check

- **Variant**: `byte_record_eq_length_check_23fb0cd_1`
- **Location**: `patches/byte_record_eq_length_check_23fb0cd_1.patch` (applies to `src/byte_record.rs`)
- **Property**: `property_byte_record_eq_matches_fields`
- **Witness**: `witness_byte_record_eq_matches_fields_case_length_mismatch`
- **Fix commit**: `23fb0cd676bf71c23fc8de45856cbf0187627e45` — `csv: fix record equality, redux`
- **Invariant violated**: Records of different field counts must never compare equal, even when the shorter one is a prefix of the longer.
- **How the mutation triggers**: Removes the `if self.len() != other.len() { return false; }` guard, leaving the zip-based field comparison which silently truncates at the shorter iterator; e.g. `["12","34","56"] == ["12","34"]` incorrectly returns `true`.

### 6. Comment character honored mid-record

- **Variant**: `core_reader_comment_only_at_record_start_a5745ba_1`
- **Location**: `patches/core_reader_comment_only_at_record_start_a5745ba_1.patch` (applies to `csv-core/src/reader.rs`)
- **Property**: `property_comment_only_at_record_start`
- **Witness**: `witness_comment_only_at_record_start_case_mid_record_hash`
- **Fix commit**: `a5745baa172d50679e34b33d6dba3d063eb40cd4` — `csv-core: fix comment handling` (fixes #137)
- **Invariant violated**: The configured comment character only starts a comment at the beginning of a record, not at the beginning of any field.
- **How the mutation triggers**: Moves the comment NFA transition from `StartRecord` back to `StartField`. Parsing `first,#tail\n` with `comment = Some(b'#')` causes the parser to discard the second field as a comment and emit only a single field.

### 7. `deserialize_byte_buf` UTF-8-validates raw bytes

- **Variant**: `deserialize_byte_buf_bypasses_utf8_9e644e6_1`
- **Location**: `patches/deserialize_byte_buf_bypasses_utf8_9e644e6_1.patch` (applies to `src/deserializer.rs`)
- **Property**: `property_deserialize_byte_buf_accepts_non_utf8`
- **Witness**: `witness_deserialize_byte_buf_accepts_non_utf8_case_invalid_middle`
- **Fix commit**: `9e644e66db0aa0b931758de1c2b7da555fb632b7` — `serde: fix bug in handling of invalid UTF-8`
- **Invariant violated**: A struct field annotated with `#[serde(with = "serde_bytes")]` takes arbitrary bytes — deserializing into it must not perform UTF-8 validation on the source field.
- **How the mutation triggers**: Reroutes `deserialize_byte_buf` through `next_field()` (which runs UTF-8 validation) instead of `next_field_bytes()` (which returns raw bytes). Deserializing a record whose middle field contains invalid UTF-8 now returns a decode error.
