# Plan 1: Lint Rule AST Refactor for Anchor/Alias Rules

## Status: COMPLETE

## Context

Three lint rules (`noDuplicateAnchors`, `noUndeclaredAliases`, `noUnusedAnchors`) use manual token scanning via `root.syntax().preorder_with_tokens()` to find anchors and aliases. Now that the parser creates proper `YamlAnchorProperty` and `YamlAliasNode` AST nodes (commit `a24af92f6f`), these rules should use AST queries instead.

Note: All three rules use `Ast<YamlRoot>` as query type and walk the entire document. This is because they need to correlate anchors and aliases across the whole document. This pattern is acceptable — the refactor improves how they find nodes (AST nodes vs raw tokens) but keeps the same root-level query.

## Changes

### 1. `no_duplicate_anchors.rs`
- **Current**: Walks tokens, checks `ANCHOR_PROPERTY_LITERAL`, strips `&` prefix
- **Target**: Use `root.syntax().descendants().filter_map(YamlAnchorProperty::cast)` to find anchor nodes, use `value_token()` to get the token text
- **File**: `crates/biome_yaml_analyze/src/lint/nursery/no_duplicate_anchors.rs`

### 2. `no_undeclared_aliases.rs`
- **Current**: Walks tokens, collects `ANCHOR_PROPERTY_LITERAL` and `ALIAS_LITERAL`, strips prefixes
- **Target**: Use `descendants().filter_map(YamlAnchorProperty::cast)` for anchors, `descendants().filter_map(YamlAliasNode::cast)` for aliases
- **File**: `crates/biome_yaml_analyze/src/lint/nursery/no_undeclared_aliases.rs`

### 3. `no_unused_anchors.rs`
- **Current**: Same token walking pattern
- **Target**: Same AST node approach
- **File**: `crates/biome_yaml_analyze/src/lint/nursery/no_unused_anchors.rs`

## Test Files
- `crates/biome_yaml_analyze/tests/specs/nursery/noDuplicateAnchors/{valid,invalid}.yaml`
- `crates/biome_yaml_analyze/tests/specs/nursery/noUndeclaredAliases/{valid,invalid}.yaml`
- `crates/biome_yaml_analyze/tests/specs/nursery/noUnusedAnchors/{valid,invalid}.yaml`

## Verification
1. `cargo test -p biome_yaml_analyze` — all tests pass
2. Accept any snapshot updates (diagnostics may change slightly due to different range reporting)
