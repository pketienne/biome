# Plan 24: Advanced Lint Rules, Schema Phase 2 & Code Actions

## Status: IMPLEMENTED

## Context

Following Plan 23, this phase adds more lint rules, enhances schema validation with alias resolution and auto-discovery, and introduces the first code actions for YAML lint rules.

---

## Part A: New Lint Rules

### A1. `noDeepNesting` — detect deeply nested structures

**Implementation:** `crates/biome_yaml_analyze/src/lint/nursery/no_deep_nesting.rs`
- Walks all descendants, counts nesting depth for `YAML_BLOCK_MAPPING`, `YAML_BLOCK_SEQUENCE`, `YAML_FLOW_MAPPING`, `YAML_FLOW_SEQUENCE`
- Default max depth: 4 (flags at depth > 4)
- `recommended: false`, `severity: Warning`

### A2. `useConsistentAnchorNaming` — enforce camelCase anchor names

**Implementation:** `crates/biome_yaml_analyze/src/lint/nursery/use_consistent_anchor_naming.rs`
- Uses semantic model to iterate all anchors
- Checks each anchor name against camelCase convention (no `_`, no `-`, starts with lowercase)
- `recommended: false`, `severity: Warning`

---

## Part B: Schema Validation Enhancements

### B1. Alias resolution in YAML-to-JSON converter

**File:** `crates/biome_yaml_analyze/src/utils/yaml_to_json.rs`
- Added `ConvertCtx` struct that builds an anchor name → value node map from the syntax tree
- All conversion functions now accept `&mut ConvertCtx`
- `YamlAliasNode` resolution: looks up anchor name, finds value node, recursively converts
- Circular reference guard via `resolving: FxHashSet<String>` — circular aliases return `Value::Null`

### B2. Schema associations by glob pattern

**File:** `crates/biome_rule_options/src/use_valid_schema.rs`
- Added `schema_associations: Option<BTreeMap<String, String>>` option
- Maps glob patterns to schema file paths

**File:** `crates/biome_yaml_analyze/src/lint/nursery/use_valid_schema.rs`
- Added `find_schema_by_association()` — matches file path against configured glob patterns
- Added simple glob matching (`glob_matches`, `glob_match_str`, `simple_glob_match`) supporting `*` and `**`
- Schema resolution order: explicit `schemaPath` → glob associations → inline `# yaml-language-server:` comment

---

## Part C: Code Actions

### C1. `noUnusedAnchors` — Safe fix to remove unused anchor

**File:** `crates/biome_yaml_analyze/src/lint/nursery/no_unused_anchors.rs`
- Added `fix_kind: FixKind::Safe`
- `action()` method finds the `YamlAnchorProperty` node matching the diagnostic range and removes it via `mutation.remove_node()`
- Snapshot shows `FIXABLE` tag and diff of anchor removal

### C2. `noEmptyKeys` — Unsafe fix to remove entry with empty key

**File:** `crates/biome_yaml_analyze/src/lint/nursery/no_empty_keys.rs`
- Added `fix_kind: FixKind::Unsafe`
- `action()` method removes the entire `YamlBlockMapImplicitEntry` node

---

## Files Modified/Created

### New files:
- `crates/biome_yaml_analyze/src/lint/nursery/no_deep_nesting.rs`
- `crates/biome_yaml_analyze/src/lint/nursery/use_consistent_anchor_naming.rs`
- `crates/biome_yaml_analyze/tests/specs/nursery/noDeepNesting/{valid,invalid}.yaml`
- `crates/biome_yaml_analyze/tests/specs/nursery/useConsistentAnchorNaming/{valid,invalid}.yaml`
- `crates/biome_yaml_analyze/tests/specs/nursery/noEmptyKeys/{valid,invalid}.yaml`

### Modified files:
- `crates/biome_diagnostics_categories/src/categories.rs` — added `noDeepNesting`, `useConsistentAnchorNaming`
- `crates/biome_yaml_analyze/src/utils/yaml_to_json.rs` — alias resolution with ConvertCtx
- `crates/biome_yaml_analyze/src/lint/nursery/use_valid_schema.rs` — glob-based schema associations
- `crates/biome_yaml_analyze/src/lint/nursery/no_unused_anchors.rs` — code action
- `crates/biome_yaml_analyze/src/lint/nursery/no_empty_keys.rs` — code action
- `crates/biome_rule_options/src/use_valid_schema.rs` — `schemaAssociations` option

## Verification

- `cargo build -p biome_yaml_analyze` ✓
- `cargo test -p biome_yaml_analyze` — 65 passed ✓
- `cargo test -p biome_yaml_parser` — 136 passed ✓
- `cargo test -p biome_yaml_formatter` — 55 passed ✓
- `cargo build -p biome_cli` ✓
