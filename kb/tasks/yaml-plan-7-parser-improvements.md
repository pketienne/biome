# Plan 7: Parser Improvements

## Status: COMPLETE (7B error messages; 7A multiline plain scalars deferred — significant parser feature)

## Context

The YAML parser has a TODO for multiline plain scalar support and could benefit from improved error messages. The multiline plain scalar limitation is the most impactful parser improvement remaining.

## Changes

### 7A. Multiline Plain Scalar Support

**File**: `crates/biome_yaml_parser/src/lexer/mod.rs` (line ~484)

The `consume_plain_literal()` function has an explicit TODO. Currently stops at first line break. Needs to continue consuming lines at same or greater indentation.

Key challenges:
- Track indentation level of first scalar line
- Continue consuming subsequent lines at same or greater indent
- Stop when indent decreases or YAML indicator found
- Handle flow vs block context differences

### 7B. Improved Error Messages

**File**: `crates/biome_yaml_parser/src/parser/block.rs` (line ~180)

The `parse_block_map_implicit_entry()` uses generic `p.expect(T![:])` — could provide a more specific error when the colon is missing after a key.

**File**: `crates/biome_yaml_parser/src/parser/block.rs` (line ~59)

Properties without a following value produce a bogus node silently — could add a diagnostic.

## Verification
1. `cargo test -p biome_yaml_parser` — all tests pass
2. Existing test snapshots may need updating
3. Test with multiline plain scalar YAML files
