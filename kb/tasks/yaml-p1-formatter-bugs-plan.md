# Fix P1 Formatter Bugs: Block Scalar Indicators & Anchor Properties

## Status: P1A FIXED, P1B/P1C BLOCKED (parser bug)

## Bug 1A — Block Scalar Indicator (FIXED)

**Bug**: `content: |` formats as `content:\n\t|` — the `|`/`>` indicator is pushed to a new indented line.

**Root cause**: `block_map_implicit_entry.rs` treats ALL non-flow block values with `hard_line_break() + block_indent(&value.format())`. Block scalars need their indicator on the same line as the colon.

**Fix**: Added variant-specific matching in `block_map_implicit_entry.rs`. For `YamlLiteralScalar`/`YamlFoldedScalar`, use `colon, space, value.format()` to keep the indicator inline. Applied the same pattern to `block_sequence_entry.rs`.

Additional fix needed: When bypassing a node's formatter (decomposing block mapping/sequence), must call `f.comments().mark_suppression_checked(node.syntax())` to satisfy the formatter's suppression comment tracking.

### Files Changed

| File | Change |
|------|--------|
| `crates/biome_yaml_formatter/src/yaml/auxiliary/block_map_implicit_entry.rs` | Variant-specific block value handling |
| `crates/biome_yaml_formatter/src/yaml/auxiliary/block_sequence_entry.rs` | Same pattern for sequence entries |
| `crates/biome_yaml_formatter/tests/specs/yaml/scalar/literal_block.yaml.snap` | Updated — `content: |` on same line |
| `crates/biome_yaml_formatter/tests/specs/yaml/scalar/folded_block.yaml.snap` | Updated — `content: >` on same line |

### Test Results
- 5 unit tests: PASS
- 17 snapshot tests: PASS (2 snapshots updated)
- 41 parser spec tests: PASS (unaffected)

---

## Bugs 1B & 1C — Anchor Property Placement (BLOCKED)

**Bug**: `defaults: &defaults\n  timeout: 30` formats as `defaults:\n&defaults\n  timeout: 30` — the anchor gets pushed to a new line.

**Actual root cause**: This is a **parser bug**, not a formatter bug. The YAML parser does not correctly handle `&anchor` syntax before block values. The AST dump shows:
- `defaults:` entry has an **empty value** (no value node at all)
- `&defaults` is parsed as `ERROR_TOKEN`, creating a `YAML_BOGUS_BLOCK_MAP_ENTRY`
- `defaults\n  timeout` is parsed as a single plain literal (wrong)
- `retries: 3` ends up as a **separate document** entirely

The formatter code for handling properties on block mappings/sequences is ready (in the entry formatters), but the parser never creates the proper AST structure with properties on block nodes.

**To fix**: The YAML parser needs to be enhanced to:
1. Recognize `&anchor` and `!tag` as properties of block values
2. Build proper `YamlBlockMapping { properties: Some(...), entries: [...] }` nodes
3. This is a non-trivial parser change in the block value parsing logic

This is consistent with the known limitation documented in `yaml-remaining-work-plan.md` item 3C: "Anchor/alias parser support — currently text-based scanning; proper AST support would improve lint rules."
