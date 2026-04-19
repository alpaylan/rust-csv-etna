#!/usr/bin/env bash
# Validate rust-csv ETNA workload: base + 7 variants × 4 frameworks.
#
# On base: every (tool, property) pair must emit status:passed.
# On variant: the matching property via each of (proptest, quickcheck,
#             crabcheck, hegel) must emit status:failed.
#
# Exit 0 if all checks hold; exit 1 otherwise.

set -u

cd "$(dirname "$0")/.."

FAIL_COUNT=0
TASKS_OK=0

BASE_REF=$(git rev-parse master)

status_of() {
    # Extract "status":"X" from a JSON line.
    python3 -c 'import json,sys; print(json.loads(sys.argv[1]).get("status","?"))' "$1" 2>/dev/null
}

run_and_get_status() {
    local tool="$1" prop="$2"
    local out
    out=$(./target/release/etna "$tool" "$prop" 2>/dev/null | tail -n 1)
    status_of "$out"
}

build_for_branch() {
    cargo build --release --bin etna 2>&1 >/dev/null
    if [ $? -ne 0 ]; then
        echo "FATAL: build failed on $(git rev-parse --abbrev-ref HEAD)"
        exit 1
    fi
}

# Property names (must match the runner's ALL_PROPERTIES).
ALL_PROPS=(
    ResetClearsOutputPosition
    TrimAllAppliesWithoutHeaders
    WriterCommentCharAutoQuote
    ByteRecordEqMatchesFields
    CommentOnlyAtRecordStart
    DeserializeByteBufAcceptsNonUtf8
)

# (variant-branch-suffix, variant-property)
VARIANT_PROPS=(
    "core_reader_reset_output_pos_zero_066de4a_1:ResetClearsOutputPosition"
    "reader_trim_all_without_headers_ce01ae7_1:TrimAllAppliesWithoutHeaders"
    "writer_comment_char_auto_quote_0f64d3f_1:WriterCommentCharAutoQuote"
    "byte_record_eq_field_boundaries_efc4a51_1:ByteRecordEqMatchesFields"
    "byte_record_eq_length_check_23fb0cd_1:ByteRecordEqMatchesFields"
    "core_reader_comment_only_at_record_start_a5745ba_1:CommentOnlyAtRecordStart"
    "deserialize_byte_buf_bypasses_utf8_9e644e6_1:DeserializeByteBufAcceptsNonUtf8"
)

# 1) Base: all tools × all properties must pass.
echo "=== base ($BASE_REF) ==="
git checkout --quiet master
build_for_branch
for tool in etna proptest quickcheck crabcheck hegel; do
    for prop in "${ALL_PROPS[@]}"; do
        s=$(run_and_get_status "$tool" "$prop")
        if [ "$s" != "passed" ]; then
            echo "  [FAIL] base: $tool $prop -> $s"
            FAIL_COUNT=$((FAIL_COUNT+1))
        else
            TASKS_OK=$((TASKS_OK+1))
        fi
    done
done

# 2) Variants: the matching property must fail for every framework.
for vp in "${VARIANT_PROPS[@]}"; do
    variant="${vp%%:*}"
    prop="${vp##*:}"
    echo "=== variant etna/$variant (prop=$prop) ==="
    git checkout --quiet "etna/$variant"
    build_for_branch
    for tool in proptest quickcheck crabcheck hegel; do
        s=$(run_and_get_status "$tool" "$prop")
        if [ "$s" != "failed" ]; then
            echo "  [FAIL] etna/$variant $tool $prop -> $s (want failed)"
            FAIL_COUNT=$((FAIL_COUNT+1))
        else
            TASKS_OK=$((TASKS_OK+1))
        fi
    done
    # Also confirm the etna (witness) replay fails for this variant's witness.
    s=$(run_and_get_status "etna" "$prop")
    if [ "$s" != "failed" ]; then
        echo "  [WARN] etna/$variant etna $prop -> $s (expected failed, variant may share witness)"
    fi
done

git checkout --quiet master

echo
echo "Validation summary: $TASKS_OK ok, $FAIL_COUNT failures"
if [ "$FAIL_COUNT" -eq 0 ]; then
    exit 0
else
    exit 1
fi
