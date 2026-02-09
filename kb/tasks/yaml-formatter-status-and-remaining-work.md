# YAML Formatter — Status & Remaining Work

## Status: CORE COMPLETE — Polish Remaining

All core formatter functionality is implemented and tested. The remaining items are test coverage improvements, config options, and lint rule refactoring.

## Completed Work

### P1A. Block Scalar Indicator Placement ✅
- **Commit**: `ed8dcdbcac`
- **Fix**: `content: |` no longer formats as `content:\n\t|` — indicator stays inline

### P1B/P1C. Anchor/Tag/Alias Parser Support ✅
- **Commit**: `a24af92f6f`
- **Fix**: Added lexer token emission (`ANCHOR_PROPERTY_LITERAL`, `TAG_PROPERTY_LITERAL`, `ALIAS_LITERAL`), parser property/alias parsing, grammar slot reordering
- **Result**: `defaults: &defaults` and `<<: *defaults` now round-trip correctly

### Inline Comment Fix ✅
- **Commit**: `4e745691ad`
- **Fix**: `key: value # comment` no longer adds extra space on each format pass

### Formatter Implementation ✅
- **Commit**: `b2d43ea2ab`
- **Coverage**: 58 per-node formatters (30 auxiliary + 13 union + 8 list + 6 bogus + 1 CST)
- **Tests**: 17 snapshot specs, all passing

---

## Remaining Work

### Priority 2: Test Coverage (SHOULD DO)

#### 2A. Multi-document test
Add `document/multiple.yaml` covering multiple `---`/`...` sequences.
```yaml
---
doc1: value1
...
---
doc2: value2
...
```
**File**: `crates/biome_yaml_formatter/tests/specs/yaml/document/multiple.yaml`

#### 2B. Complex nested structures test
Add `mapping/complex.yaml` covering 3+ nesting levels, mixed block/flow, mappings inside sequences and vice versa.
**File**: `crates/biome_yaml_formatter/tests/specs/yaml/mapping/complex.yaml`

#### 2C. Edge-case scalars test
Add `scalar/edge_cases.yaml` covering multi-line plain scalars, empty strings, special characters, long lines, Unicode.
**File**: `crates/biome_yaml_formatter/tests/specs/yaml/scalar/edge_cases.yaml`

---

### Priority 3: Enhancements (NICE TO HAVE)

#### 3A. YAML-specific formatter config options
- `quote_style`: Prefer single or double quotes
- `trailing_newline`: Ensure final newline
- `max_blank_lines`: Maximum consecutive blank lines
- **Files**: `context.rs`, `crates/biome_configuration/src/yaml.rs`, `crates/biome_service/src/file_handlers/yaml.rs`

#### 3B. CLI integration tests
End-to-end tests with `biome format`, `biome lint`, `biome check` on YAML files.
- **Files**: `crates/biome_cli/tests/` following existing patterns

#### 3C. Lint rule AST-based anchor/alias detection
Refactor `noDuplicateAnchors`, `noUndeclaredAliases`, `noUnusedAnchors` to use AST node queries (`YamlAnchorProperty`, `YamlAliasNode`) instead of manual token scanning, now that the parser creates proper AST nodes.
- **Files**: `crates/biome_yaml_analyze/src/lint/nursery/no_duplicate_anchors.rs`, `no_undeclared_aliases.rs`, `no_unused_anchors.rs`

#### 3D. Documentation
- Rule examples for each of 23 lint rules
- YAML configuration guide for `biome.json`
- Migration guide from yamllint

---

## Architecture Reference

| Component | Files | Status |
|-----------|-------|--------|
| Formatter entry | `crates/biome_yaml_formatter/src/lib.rs` | Complete |
| Format context | `crates/biome_yaml_formatter/src/context.rs` | Complete (basic options) |
| Node formatters | `crates/biome_yaml_formatter/src/yaml/auxiliary/*.rs` | 30 files, all implemented |
| Union dispatchers | `crates/biome_yaml_formatter/src/yaml/any/*.rs` | 13 files, auto-generated |
| List formatters | `crates/biome_yaml_formatter/src/yaml/lists/*.rs` | 8 files, all customized |
| Bogus handlers | `crates/biome_yaml_formatter/src/yaml/bogus/*.rs` | 6 files, verbatim fallback |
| Generated glue | `crates/biome_yaml_formatter/src/generated.rs` | ~1700 lines, auto-generated |
| Test specs | `crates/biome_yaml_formatter/tests/specs/yaml/` | 17 specs across 6 categories |
