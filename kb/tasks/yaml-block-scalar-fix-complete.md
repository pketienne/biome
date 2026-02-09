# YAML Block Scalar Indicator Fix — Complete

## Status: COMPLETE

## Problem

Block scalar indicators (`|` and `>`) were incorrectly pushed to a new indented line instead of staying inline with the colon:

- Input: `content: |` → Formatted: `content:\n\t|`
- Input: `content: >` → Formatted: `content:\n\t>`

## Root Cause

In `block_map_implicit_entry.rs`, ALL non-flow block values were handled identically with `hard_line_break() + block_indent(&value.format())`. This pushed the entire block scalar node (including the `|`/`>` indicator) to a new indented line. The same pattern existed in `block_sequence_entry.rs`.

## Fix

Added variant-specific matching on `AnyYamlBlockInBlockNode` in both entry formatters:

- **`YamlLiteralScalar` / `YamlFoldedScalar`**: Use `colon, space, value.format()` — keeps the indicator on the same line as the colon. The scalar's content token already contains its own newlines and indentation.
- **`YamlBlockMapping` / `YamlBlockSequence`**: Decompose the node — properties stay on the colon's line, entries get `block_indent` separately. Must call `f.comments().mark_suppression_checked(node.syntax())` to satisfy the formatter's suppression comment tracking when bypassing a node's formatter.
- **`YamlBogusBlockNode`**: Falls through to original behavior (hard_line_break + block_indent).

## Files Changed

| File | Change |
|------|--------|
| `crates/biome_yaml_formatter/src/yaml/auxiliary/block_map_implicit_entry.rs` | Variant-specific block value handling |
| `crates/biome_yaml_formatter/src/yaml/auxiliary/block_sequence_entry.rs` | Same pattern for sequence entries |
| `crates/biome_yaml_formatter/tests/specs/yaml/scalar/literal_block.yaml.snap` | Updated — `content: |` on same line |
| `crates/biome_yaml_formatter/tests/specs/yaml/scalar/folded_block.yaml.snap` | Updated — `content: >` on same line |
| `kb/tasks/yaml-p1-formatter-bugs-plan.md` | Updated plan with findings |

## Test Results

- 5 unit tests: PASS
- 17 snapshot tests: PASS (2 snapshots updated)
- 41 parser spec tests: PASS (unaffected)

## Related: Anchor Property Bugs (P1B/P1C) — BLOCKED

During investigation, discovered that the anchor/alias formatting bugs (`defaults: &defaults` → `defaults:\n&defaults`) are actually **parser bugs**, not formatter bugs. The parser does not recognize `&anchor` before block values — it creates `ERROR_TOKEN` and `YAML_BOGUS_BLOCK_MAP_ENTRY` nodes instead of proper `YamlBlockMapping { properties: &anchor }` AST structure.

The formatter-side code to handle properties on block mappings/sequences is already in place and will work once the parser is enhanced to support anchor/tag properties on block values.

## Commit

`ed8dcdbcac` — fix(yaml): keep block scalar indicators on same line as colon
