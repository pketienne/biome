# Plan 23: Lint Rules, Schema Enhancements & Testing Hardening

## Status: IMPLEMENTED

## Context

All 22 implementation plans and 4 feature additions for YAML support are complete. The next phase focuses on three areas: (1) new lint rules leveraging the semantic model, (2) improvements to the JSON Schema validation rule, and (3) expanded test coverage for edge cases and reliability.

---

## Part A: New Lint Rules

### A1. `noCircularAliases` — detect anchor/alias cycles

**Problem:** YAML allows constructing circular reference chains (`&a` → value containing `*b`, `&b` → value containing `*a`). These cause infinite loops in consumers.

**Approach:** Build an alias-follows-anchor graph from the semantic model, detect cycles using DFS with a visited set. The semantic model already tracks anchor→alias and alias→anchor relationships — we need to follow the chain: for each alias, find its anchor, then find the parent node of that anchor's value, check if that parent contains more aliases, and continue.

**Implementation:**

Since the current semantic model resolves aliases to anchors by name only (not by structural containment), we need a simpler approach: walk the YAML AST looking for anchors whose value subtree contains aliases that eventually reference back to themselves.

1. **New file:** `crates/biome_yaml_analyze/src/lint/nursery/no_circular_aliases.rs`
2. **Query type:** `Ast<YamlRoot>` (need full document for graph analysis)
3. **Algorithm:**
   - Build `semantic_model(root)`
   - For each anchor, walk its value's subtree collecting aliases
   - For each alias found, resolve to its anchor, then walk that anchor's subtree
   - Track visited anchors; if we revisit one, report a cycle
4. **Diagnostic:** Points at the alias that completes the cycle, with `.detail()` pointing at the anchor it references
5. **Register:** Auto-discovered via `declare_group_from_fs!` in nursery — just add the file
6. **Register category:** Add `"lint/nursery/noCircularAliases"` to `categories.rs`

**Key files:**
- `crates/biome_yaml_analyze/src/lint/nursery/no_circular_aliases.rs` (NEW)
- `crates/biome_diagnostics_categories/src/categories.rs` (add category)
- `crates/biome_yaml_analyze/tests/specs/nursery/noCircularAliases/` (test dir)

### A2. `noForwardAliasReferences` — aliases must appear after their anchors

**Problem:** YAML spec doesn't require anchors to be declared before aliases, but forward references are confusing and many tools don't handle them correctly.

**Approach:** Compare text positions — if an alias's range starts before its anchor's range within the same document, flag it.

**Implementation:**

1. **New file:** `crates/biome_yaml_analyze/src/lint/nursery/no_forward_alias_references.rs`
2. **Query type:** `Ast<YamlRoot>`
3. **Algorithm:**
   - Build `semantic_model(root)`
   - For each alias via `model.all_aliases()`, get its resolved anchor via `alias.anchor()`
   - If `alias.range().start() < anchor.range().start()`, it's a forward reference
4. **Diagnostic:** Primary range on the alias, `.detail()` on the anchor showing where it's later declared
5. **Severity:** Warning (not an error per YAML spec, but a best practice)

**Key files:**
- `crates/biome_yaml_analyze/src/lint/nursery/no_forward_alias_references.rs` (NEW)
- `crates/biome_diagnostics_categories/src/categories.rs` (add category)
- `crates/biome_yaml_analyze/tests/specs/nursery/noForwardAliasReferences/` (test dir)

---

## Part B: Schema Validation Enhancements

### B1. Per-property error ranges instead of whole-document fallback

**Current behavior:** When a schema error references a specific path like `/name`, the rule tries to resolve it via `resolve_path_range()`. But for root-level errors (missing required property, additional properties), the fallback is the entire document range — not helpful.

**Enhancement:** For root-level errors, resolve to the range of the mapping that contains all the entries (all entries from first to last), which gives a more focused highlight.

**File:** `crates/biome_yaml_analyze/src/lint/nursery/use_valid_schema.rs`
- When `instance_path` is empty and error is about a specific property (e.g., "extra" is additional), attempt to find that property's entry in the mapping and use its range
- Parse error message with regex to extract property names from common patterns like `'extra' was unexpected`, `"name" is a required property`

### B2. Schema comment with URL support

**Current behavior:** `find_schema_comment()` only supports file paths.

**Enhancement:** If the schema path starts with `http://` or `https://`, skip it gracefully (return `None` or add a future note about URL support). Currently it would try to read a file called `https://...` which fails silently. We should at minimum log a meaningful non-crash.

**File:** `crates/biome_yaml_analyze/src/lint/nursery/use_valid_schema.rs`
- In `find_schema_comment()` or `resolve_schema_path()`, detect URL-like paths and return early

### B3. Alias resolution in YAML-to-JSON converter

**Current behavior:** `flow_node_to_json` returns `Value::Null` for alias nodes. This means schemas validating documents with aliases get incorrect results.

**Enhancement:** Accept the semantic model as a parameter, resolve aliases to their anchor's value during conversion.

**Files:**
- `crates/biome_yaml_analyze/src/utils/yaml_to_json.rs` — add optional `SemanticModel` parameter to conversion functions; resolve `YamlAliasNode` by looking up its anchor's value subtree and recursively converting
- `crates/biome_yaml_analyze/src/lint/nursery/use_valid_schema.rs` — pass `None` for now (alias resolution is opt-in; can be enabled later if needed)

---

## Part C: Testing Hardening

### C1. Expand anchor/alias lint rule tests

Current tests are minimal (1 invalid case each). Add:

**`noDuplicateAnchors/`:**
- `invalid.yaml`: Add multi-document case (anchors with same name in different docs is OK), 3+ duplicates, nested anchors
- `valid.yaml`: Add multi-document with same anchor name (should be valid), nested mappings

**`noUndeclaredAliases/`:**
- `invalid.yaml`: Add aliases in flow context, aliases in sequences, multiple unresolved aliases
- `valid.yaml`: Add flow-context anchor/alias pairs, nested usage

**`noUnusedAnchors/`:**
- `invalid.yaml`: Add multiple unused anchors, anchor in flow context
- `valid.yaml`: Add merge key usage (`<<: *anchor`), multiple aliases to same anchor

### C2. Schema validation edge case tests

Add new test files in `useValidSchema/`:
- `nested_schema.yaml` + `nested-schema.json`: Test nested object validation with arrays
- `types.yaml` + `types-schema.json`: Test type coercion (numbers, booleans, null)
- `empty_doc.yaml`: Empty document against a schema
- `multi_doc.yaml`: Multi-document file (only first doc validated)

### C3. Parser edge case tests

Add to `crates/biome_yaml_parser/tests/yaml_test_suite/`:

**ok/block/:**
- `anchor_alias_basic.yaml`: Basic anchor/alias parsing
- `multi_document.yaml`: Multiple documents with `---` and `...`
- `empty_values.yaml`: Keys with empty/null values
- `complex_keys.yaml`: Multi-line and flow keys in block context

**ok/flow/:**
- `nested_flow.yaml`: Deeply nested flow collections
- `mixed_flow_block.yaml`: Flow collections inside block context

**err/block/:**
- `duplicate_key_warning.yaml`: Duplicate keys (parser should handle gracefully)
- `tab_indent.yaml`: Tab indentation (should produce diagnostic)

### C4. Formatter stress tests with anchors/aliases

Add to `crates/biome_yaml_formatter/tests/specs/yaml/`:
- `anchor_alias.yaml`: Formatting preserves anchor/alias syntax
- `complex_anchors.yaml`: Anchors on different node types (mappings, sequences, scalars)

---

## Execution Order

1. **A1 + A2** — New lint rules (can be done in parallel)
2. **B1 + B2 + B3** — Schema enhancements
3. **C1 + C2 + C3 + C4** — Testing hardening
4. **Build + test verification**

## Verification

1. `cargo build -p biome_yaml_analyze` — compiles with new rules
2. `cargo test -p biome_yaml_analyze` — all existing + new tests pass
3. `cargo test -p biome_yaml_parser` — parser tests pass
4. `cargo test -p biome_yaml_formatter` — formatter tests pass
5. `cargo insta accept --workspace` — accept new snapshots
6. `cargo build -p biome_cli` — full CLI builds clean
