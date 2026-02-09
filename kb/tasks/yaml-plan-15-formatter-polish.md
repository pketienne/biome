# Plan 15: Additional Formatter Polish

## Status: COMPLETE

## Goal

Polish the YAML formatter with three targeted improvements:
1. Flow collection internal spacing (`{ key: value }` with spaces inside braces/brackets)
2. Ensure proper space after `:` in flow mappings (already working, add tests)
3. Add flow collection and comment formatter snapshot tests for better coverage

## Current State

- Flow mappings: `{l_curly entries r_curly}` — no space inside braces
- Flow sequences: `[l_brack entries r_brack]` — no space inside brackets
- Flow entry lists: `join_with(space())` + comma separated
- Comment handling: preserved via trivia, `useCommentSpacing` lint rule covers `# ` normalization
- 22 snapshot tests, 6 stress tests

## Changes

### 15A: Add space inside flow collection braces/brackets

**File**: `crates/biome_yaml_formatter/src/yaml/auxiliary/flow_mapping.rs`

Add `space()` after `{` and before `}` when the collection is non-empty:

```rust
fn fmt_fields(&self, node: &YamlFlowMapping, f: &mut YamlFormatter) -> FormatResult<()> {
    let entries = node.entries();
    write!(f, [node.l_curly_token()?.format()])?;
    if !entries.is_empty() {
        write!(f, [space(), entries.format(), space()])?;
    }
    write!(f, [node.r_curly_token()?.format()])
}
```

**File**: `crates/biome_yaml_formatter/src/yaml/auxiliary/flow_sequence.rs`

Same pattern for `[` and `]`:

```rust
fn fmt_fields(&self, node: &YamlFlowSequence, f: &mut YamlFormatter) -> FormatResult<()> {
    let entries = node.entries();
    write!(f, [node.l_brack_token()?.format()])?;
    if !entries.is_empty() {
        write!(f, [space(), entries.format(), space()])?;
    }
    write!(f, [node.r_brack_token()?.format()])
}
```

### 15B: Add formatter snapshot tests for flow collections

**File**: `crates/biome_yaml_formatter/tests/specs/yaml/flow/mapping.yaml` (update)

Add cases: empty flow map `{}`, nested flow maps, flow map with many entries.

**File**: `crates/biome_yaml_formatter/tests/specs/yaml/flow/sequence.yaml` (update)

Add cases: empty flow sequence `[]`, nested flow sequences.

### 15C: Add `document/separated_by_doc_end.yaml` formatter test

Test multi-document with `...` separators to ensure formatter handles them correctly.

### 15D: Verify and update stress test expectations

Run stress tests and confirm idempotency with the new flow spacing.

## Test Plan

1. Unit tests in `lib.rs`: add `flow_mapping_spacing` and `flow_sequence_spacing` tests
2. Snapshot tests: accept updated snapshots reflecting new spacing
3. Stress tests: all 6 must remain idempotent
4. CLI tests: 5 must pass

## Files Changed

| File | Change |
|------|--------|
| `flow_mapping.rs` | Add space inside braces for non-empty maps |
| `flow_sequence.rs` | Add space inside brackets for non-empty sequences |
| `flow/mapping.yaml` | Add test cases |
| `flow/sequence.yaml` | Add test cases |
| `document/separated_by_doc_end.yaml` | New test file |
| `lib.rs` | Add unit tests |
| Snapshot files | Accept updates |
