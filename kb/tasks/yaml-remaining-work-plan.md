# YAML Remaining Work — Implementation Plan

## Priority 1: Formatter Bugs (MUST FIX)

### 1A. Block scalar indicator placed on wrong line

**Bug**: `content: |` formats as `content:\n\t|` — the `|`/`>` indicator is pushed to a new indented line instead of staying inline.

**Root cause**: In `block_map_implicit_entry.rs`, block nodes (including `YamlLiteralScalar` and `YamlFoldedScalar`) go through the `Some(value)` branch which writes `hard_line_break() + block_indent(&value)`. The `|`/`>` indicator is part of the block scalar node and gets indented.

**Fix**: Add a match arm for `AnyYamlBlockNode::YamlLiteralScalar` and `AnyYamlBlockNode::YamlFoldedScalar` (or their common parent) that keeps the indicator on the same line as the colon:
```
colon.format(), space(), indicator.format(), content.format()
```
This may require restructuring `block_map_implicit_entry.rs` to match on specific block node variants, or adjusting the literal/folded scalar formatters to handle the line break internally.

**Files to modify**:
- `crates/biome_yaml_formatter/src/yaml/auxiliary/block_map_implicit_entry.rs`
- Possibly `literal_scalar.rs` and `folded_scalar.rs`

**Tests to update**: `scalar/literal_block.yaml.snap`, `scalar/folded_block.yaml.snap`

---

### 1B. Anchor property placed on wrong line

**Bug**: `defaults: &defaults\n  timeout: 30` formats as `defaults:\n&defaults\n  timeout: 30` — the anchor gets pushed to a new line and nested mapping loses indentation.

**Root cause**: Same as 1A — `&defaults` is a property of the value node. The value is a block mapping node (not flow-in-block), so `block_map_implicit_entry.rs` routes it through `hard_line_break() + block_indent()`. The anchor property gets formatted as part of the block value on the new line, and the nested `timeout: 30` / `retries: 3` lose their relative indentation.

**Fix**: When a block value has properties (anchor/tag), the properties should stay on the same line as the colon. The block content should follow on the next line indented. The entry formatter needs to:
1. Detect if the block value has properties
2. If yes: `colon, space, properties, hard_line_break, block_indent(content)`
3. If no: `colon, hard_line_break, block_indent(value)`

This requires the entry formatter to inspect the value node and extract properties vs. content separately, rather than formatting the entire value as one block.

**Files to modify**:
- `crates/biome_yaml_formatter/src/yaml/auxiliary/block_map_implicit_entry.rs`
- Possibly `block_mapping.rs`, `block_sequence.rs` for similar patterns

**Tests to update**: `properties/anchor_alias.yaml.snap`, possibly `properties/tag.yaml.snap`

---

### 1C. Nested indentation lost in anchor/alias test

**Bug**: In the anchor_alias output, `retries: 3` appears at root level instead of indented under `defaults:`. The `production:` entry and its children also have incorrect indentation.

**Root cause**: Likely related to 1B — the block_indent wrapper is applied to the wrong scope when properties are present, causing sibling entries to lose their indentation context.

**Fix**: Will likely be resolved when 1B is fixed correctly. Verify indentation of all entries in the anchor_alias test after the 1B fix.

---

## Priority 2: Test Coverage Gaps (SHOULD DO)

### 2A. Multi-document test

**Current state**: `document/markers.yaml` only tests a single document with `---`/`...` markers.

**Add**: `document/multiple.yaml` test spec:
```yaml
---
doc1: value1
...
---
doc2: value2
...
```

**Files to create**:
- `crates/biome_yaml_formatter/tests/specs/yaml/document/multiple.yaml`

---

### 2B. Complex nested structures test

**Add**: `mapping/complex.yaml` test spec covering:
- Deeply nested mappings (3+ levels)
- Mixed block/flow collections
- Mappings containing sequences and vice versa
- Anchor with alias usage in flow context

**Files to create**:
- `crates/biome_yaml_formatter/tests/specs/yaml/mapping/complex.yaml`

---

### 2C. Edge case scalars test

**Add**: `scalar/edge_cases.yaml` test spec covering:
- Multi-line plain scalars
- Empty string values (`key: ""`, `key: ''`)
- Special characters in scalars
- Very long lines
- Unicode content

**Files to create**:
- `crates/biome_yaml_formatter/tests/specs/yaml/scalar/edge_cases.yaml`

---

## Priority 3: Enhancements (NICE TO HAVE)

### 3A. YAML-specific formatter configuration options

**Current**: Only generic options (indent_style, indent_width, line_ending, line_width).

**Potential additions**:
- `quote_style`: Prefer single or double quotes for string scalars
- `trailing_newline`: Ensure final newline
- `max_blank_lines`: Maximum consecutive blank lines (1 or 2)

**Files to modify**:
- `crates/biome_yaml_formatter/src/context.rs` — add options
- `crates/biome_configuration/src/yaml.rs` — add config fields
- `crates/biome_service/src/file_handlers/yaml.rs` — wire options through

---

### 3B. CLI integration tests

**Current**: No YAML-specific CLI tests.

**Add**: Basic CLI format and lint tests verifying end-to-end behavior:
- `biome format` on a YAML file
- `biome lint` on a YAML file
- `biome check` on a YAML file

**Files to create**:
- Tests in `crates/biome_cli/tests/` following existing patterns

---

### 3C. Lint rule AST-based anchor/alias detection

**Current**: `noDuplicateAnchors`, `noUndeclaredAliases`, `noUnusedAnchors` scan tokens directly rather than using AST nodes (`YamlAnchorProperty`, `YamlAliasNode`).

**Enhancement**: Refactor to use AST node queries via `ctx.query()` instead of manual token scanning. This would make the rules more robust and follow the same patterns as other Biome lint rules.

**Files to modify**:
- `crates/biome_yaml_analyze/src/lint/nursery/no_duplicate_anchors.rs`
- `crates/biome_yaml_analyze/src/lint/nursery/no_undeclared_aliases.rs`
- `crates/biome_yaml_analyze/src/lint/nursery/no_unused_anchors.rs`

---

### 3D. Documentation

- Rule documentation with examples for each of the 23 lint rules
- Configuration guide for YAML in `biome.json`
- Migration guide from yamllint

---

## Execution Order

1. **1A** — Fix block scalar indicator placement (small, isolated)
2. **1B + 1C** — Fix anchor property placement and indentation (related)
3. **2A-2C** — Add test coverage (fast, no code changes)
4. **3A-3D** — Enhancements (lower priority, can be done incrementally)

## Estimated Scope

| Priority | Items | Complexity |
|----------|-------|------------|
| P1 (bugs) | 3 | Medium — requires understanding block value formatting pipeline |
| P2 (tests) | 3 | Low — just creating spec files and accepting snapshots |
| P3 (enhancements) | 4 | Varies — config options are medium, CLI tests medium, docs low |
