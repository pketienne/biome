# Plan 36: Fix Code Action Bugs, Add Missing Tests

## Gap 1: `useCommentSpacing` action() returns None

**Root cause:** `covering_element()` at line 107 never returns COMMENT tokens because COMMENT is trivia in YAML (`YamlSyntaxKind::is_trivia()` returns true for COMMENT). Trivia tokens are attached to adjacent non-trivia tokens and `covering_element()` returns the non-trivia token instead.

**Fix:** Instead of `covering_element()`, iterate through all tokens in the tree and search their leading/trailing trivia for the COMMENT token whose range overlaps `state`. Once found, rebuild the trivia piece with the space inserted, then replace the parent token's trivia.

**File:** `crates/biome_yaml_analyze/src/lint/nursery/use_comment_spacing.rs` lines 100-138

**Approach:** Walk `root.syntax().descendants_with_tokens()`, filter for tokens, then for each token iterate `leading_trivia()` and `trailing_trivia()` pieces looking for COMMENT kind where `piece.text_range()` overlaps `state`. Once found, build new trivia with the modified comment text and create a new detached token with updated trivia.

---

## Gap 2: `useConsistentQuoteStyle` action() returns None

**Root cause:** The `action()` method at line 125 searches `root.syntax().descendants()` for a node matching `state` (a `text_trimmed_range`), then does `YamlSingleQuotedScalar::cast(node)?`. The issue is that `descendants()` returns nodes but the state range was computed from `scalar.syntax().text_trimmed_range()`, and when iterating descendants the range comparison with `text_trimmed_range()` may not match due to trivia differences.

**Fix:** Replace the descendants loop with a direct `covering_element()` lookup using `state.start()`, then navigate to the parent scalar node. Alternatively, use `SyntaxNode::covering_element(state)` → cast to scalar.

**File:** `crates/biome_yaml_analyze/src/lint/nursery/use_consistent_quote_style.rs` lines 120-191

---

## Gap 3: `useBlockStyle` fix appends block text but retains flow text

**Root cause:** The action at line 127-158 only replaces the first token (`covering_token`) of the flow node. The flow node contains multiple tokens (`{`, keys, values, commas, `}`), and replacing only the first one leaves the rest in place, causing duplication like `config:\n  host: localhost\n  port: 8080{host: localhost, port: 8080}`.

**Fix:** After replacing the first token with the block text, iterate remaining tokens in the node and replace each with an empty detached token to effectively remove them.

**File:** `crates/biome_yaml_analyze/src/assist/source/use_block_style.rs` lines 111-162

---

## Gap 4: `useFlowStyle` false positive on leaf mappings inside nested structures

**Root cause:** The rule at line 56-109 checks if a block mapping has nested values, but does not check whether the mapping *itself* is nested inside another block mapping/sequence. This causes `key: value` inside `nested: child:` to be flagged even though it's a deeply nested leaf.

**Fix:** Add a parent context check: skip any `YAML_BLOCK_MAPPING` or `YAML_BLOCK_SEQUENCE` whose parent chain contains another `YAML_BLOCK_MAPPING` or `YAML_BLOCK_SEQUENCE` (i.e., it's a child value, not a top-level collection).

**File:** `crates/biome_yaml_analyze/src/assist/source/use_flow_style.rs` lines 52-111

---

## Gap 5: `useExpandedMergeKeys` doesn't produce diagnostics

**Root cause:** At line 113, `anchor_syntax.parent().and_then(|p| p.parent())` assumes the anchor property is exactly 2 levels below the block mapping that contains the values to expand. The actual tree structure may have intermediate wrapper nodes (e.g., `YAML_ANCHOR_PROPERTY → YAML_BLOCK_MAP_IMPLICIT_ENTRY → ... → YAML_BLOCK_MAPPING`), so the 2-parent assumption fails.

**Fix:** Replace the fixed-depth parent traversal with a loop that walks up from `anchor_syntax` looking for `YAML_INDENTED_BLOCK` or the first ancestor whose children include a `YAML_BLOCK_MAPPING`.

**File:** `crates/biome_yaml_analyze/src/assist/source/use_expanded_merge_keys.rs` lines 107-126

---

## Gap 6: Workspace-level YAML tests (deferred — low priority)

Add basic workspace integration tests in `crates/biome_service/tests/` that exercise `format()`, `lint()`, `pull_diagnostics()` for YAML files through `WorkspaceServer`.

**Deferred:** These are integration tests that require significant setup and are lower priority than the bug fixes.

---

## Gap 7: CLI integration tests for YAML-specific commands (deferred — low priority)

Extend `crates/biome_cli/tests/cases/yaml.rs` with tests for:
- `lint --write` (auto-fix)
- `search` with GritQL pattern on YAML files

**Deferred:** Existing 4 CLI tests provide adequate coverage for now.

---

## Gap 8: Bidirectional quote style tests

Add test spec files for `useConsistentQuoteStyle` with `PreferredQuote::Single` option, verifying double→single conversion works.

**File:** `crates/biome_yaml_analyze/tests/specs/nursery/useConsistentQuoteStyle/single_preferred.yaml`
**Options file:** `crates/biome_yaml_analyze/tests/specs/nursery/useConsistentQuoteStyle/single_preferred.options.json`

---

## Gap 9: `biome_yaml_semantic` unit tests

Add unit tests covering all public API methods of `SemanticModel`, `Anchor`, `Alias`, `UnresolvedAlias`, `DuplicateAnchor`.

**File:** `crates/biome_yaml_semantic/src/semantic_model/tests.rs` (new module)

**Test cases:**
- Single anchor, single alias → resolution
- Multiple anchors, multiple aliases → correct resolution
- Unresolved alias → appears in `all_unresolved_aliases()`
- Duplicate anchor → appears in `all_duplicate_anchors()`
- Multi-document anchor scoping
- `as_anchor()` lookup
- Anchor `all_aliases()` reverse lookup
- Empty document → zero anchors/aliases

---

## Status

- [x] Gap 1: useCommentSpacing action — FIXED (trivia search instead of covering_element)
- [x] Gap 2: useConsistentQuoteStyle action — FIXED (covering_element + ancestor walk)
- [x] Gap 3: useBlockStyle duplicate text — FIXED (replace all tokens, not just first)
- [x] Gap 4: useFlowStyle false positive — FIXED (depth-2+ nesting check)
- [x] Gap 5: useExpandedMergeKeys no diagnostics — FIXED (dynamic ancestor walk for block mapping)
- [ ] Gap 6: Workspace tests — DEFERRED (low priority)
- [ ] Gap 7: CLI integration tests — DEFERRED (low priority, 4 tests exist)
- [ ] Gap 8: Bidirectional quote tests — DEFERRED (YAML rules not in global config schema)
- [x] Gap 9: Semantic model unit tests — DONE (12 tests covering all public APIs)

All tests pass: 79 analyze + 58 formatter + 80+ parser + 39 LSP + 12 semantic
