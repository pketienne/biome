# YAML Implementation — Next Phase Status

## Completed (Previous Phases)
- Parser (YAML 1.2.2 with anchors/tags/aliases, multiline plain scalars, directives)
- Formatter (58 per-node formatters, 24 snapshot tests, 7 stress tests, all bugs fixed)
- Linter (28 lint rules, all with docs and tests)
- Lint rules registered in CLI config system
- CLI integration tests (format, format --write, lint, check)
- Lint rule AST refactor
- Inline documentation on all rules
- Compiler warnings fixed (zero warnings across all YAML crates)

## All Plans

| Plan | Description | Status |
|------|------------|--------|
| 1-5 | Parser, formatter, linter, CLI integration, lint rule AST refactor | COMPLETE |
| 6 | YAML-specific config options + per-language overrides | COMPLETE |
| 7 | Parser improvements (error messages improved; multiline scalars confirmed working) | COMPLETE |
| 8 | Advanced formatter features (range formatting improved; quote_style deferred to 13) | COMPLETE |
| 9 | Override settings (per-path YAML configuration) | COMPLETE |
| 10 | Cleanup: stale TODO removed, warnings fixed, multiline plain scalar tests added | COMPLETE |
| 11 | Default YAML indent style to spaces (YAML spec compliance) | COMPLETE |
| 12 | Compact block sequence form (`- key: value` via `align(2)`) | COMPLETE |
| 13 | `quote_style` formatter option (single/double with safe conversion) | COMPLETE |
| 14 | JSON Schema validation (lint rule + jsonschema crate) | PENDING (future phase) |
| 15 | Additional formatter polish (flow collection spacing, test expansion) | COMPLETE |
| 16 | Lint rule expansion (5 new rules: noEmptyKeys, noEmptySequenceEntries, useConsistentIndentation, noAnchorReferences, useQuotedStrings) | COMPLETE |
| 17 | Multi-document support hardening (directive lexing, edge case tests) | COMPLETE |
| 18 | `useConsistentSequenceIndentation` lint rule (yamllint `indent-sequences: consistent`) | COMPLETE |
| 19 | Semantic model (`biome_yaml_semantic` crate) | COMPLETE |
| 20 | Rename capability (anchor/alias LSP rename) | COMPLETE |
| 21 | Lexer `rewind()` / `checkpoint()` implementation | COMPLETE |

## Remaining Work

### Major Feature

- **JSON Schema validation (Plan 14)** — Validate YAML against JSON schemas (e.g., Kubernetes manifests, CI configs). Requires new `jsonschema` crate dependency, YAML-to-JSON converter, and error range mapping. Estimated 8-15 days.

### Infrastructure Gaps

#### Semantic Model — COMPLETE (Plan 19)

`biome_yaml_semantic` crate implemented. A pre-computed data structure built from a single syntax tree traversal that maps anchor declarations (`&name`) to alias references (`*name`), scoped per YAML document. Other Biome languages have dedicated crates: `biome_js_semantic` (scope chains, variable bindings, hoisting, closures), `biome_css_semantic` (rule hierarchy, custom properties), `biome_graphql_semantic` (fragment/type bindings).

**Current problem:** Each anchor/alias lint rule independently walks the entire syntax tree:
- `noDuplicateAnchors` — full traversal, collects anchors into `FxHashMap`
- `noUndeclaredAliases` — two full traversals (anchors + aliases)
- `noUnusedAnchors` — two full traversals (anchors + aliases), then compares sets
- `useValidMergeKeys` — full traversal to find `<<` keys and check alias values

Running all 4 rules = ~8 full tree traversals with redundant anchor/alias collection.

**What a semantic model would provide:**
- Single traversal to build model, then O(1) hash lookups per rule
- Anchor bindings — name, range, document scope, referenced node
- Alias references — name, range, resolved anchor (or unresolved)
- Document scoping — anchors per-document in multi-doc YAML; aliases can't cross `---` boundaries
- Pre-computed maps — anchor→aliases, alias→anchor, unresolved aliases, duplicate anchors

**Proposed structure** (following GraphQL's simpler pattern):
```rust
pub struct YamlSemanticModelData {
    pub root: YamlRoot,
    pub anchors: Vec<YamlAnchor>,
    pub anchors_by_name: FxHashMap<String, Vec<AnchorId>>,
    pub aliases: Vec<YamlAlias>,
    pub anchor_to_aliases: FxHashMap<AnchorId, Vec<AliasId>>,
    pub alias_to_anchor: FxHashMap<AliasId, Option<AnchorId>>,
    pub unresolved_aliases: Vec<UnresolvedAlias>,
    pub documents: Vec<DocumentScope>,
}
```

**What it unlocks:** rename capability, go-to-definition (alias→anchor), future rules (`noCircularAliases`, `noForwardAliasReferences`, `preferInlineValues`), ~4x fewer tree traversals when running all anchor-related rules.

**Effort:** Medium. Follow GraphQL semantic crate as template. New crate, event-based builder, service integration, refactor 4 existing lint rules.

**Key reference files:**
- `crates/biome_graphql_semantic/src/semantic_model/model.rs` — simplest existing model
- `crates/biome_graphql_semantic/src/events.rs` — event extraction pattern
- `crates/biome_css_semantic/src/semantic_model/builder.rs` — builder pattern
- `crates/biome_yaml_analyze/src/lint/nursery/no_unused_anchors.rs` — existing traversal logic to replace

---

#### Rename Capability — COMPLETE (Plan 20)

LSP "rename symbol" support for YAML anchors and aliases. Place cursor on an anchor or alias, rename it, and all related references update together.

**Implemented:** `rename: Some(rename)` at `crates/biome_service/src/file_handlers/yaml.rs`. Finds token at cursor, extracts anchor/alias name, collects all matching tokens in same YAML document, builds `TextEdit` via string replacement.

**Type signature** (from `crates/biome_service/src/file_handlers/mod.rs:986`):
```rust
type Rename = fn(&BiomePath, AnyParse, TextSize, String) -> Result<RenameResult, WorkspaceError>;
```

**Example:**
```yaml
defaults: &default_config    # anchor declaration
  timeout: 30
production:
  <<: *default_config        # alias reference (must also rename)
staging:
  <<: *default_config        # alias reference (must also rename)
```
Renaming `default_config` → `defaults` produces 3 text edits (1 anchor + 2 aliases).

**Two implementation paths:**
1. With semantic model — query `anchor_to_aliases` map, get all ranges instantly
2. Without semantic model — traverse tree to collect anchors/aliases (reuse logic from existing lint rules)

**Why simpler than JS:** YAML anchors have flat, document-scoped semantics — no lexical scoping, closures, hoisting, or module imports. Just find all `&name` and `*name` with matching names in the same document.

**What's needed:**
- A `rename` function in `yaml.rs`
- Find token at cursor, determine if anchor or alias, extract name
- Find all anchors/aliases with that name in same document
- Build `TextEdit` list (strip `&`/`*` prefix, replace name portion)
- Register as `rename: Some(rename)` in capabilities
- Validation: new name doesn't conflict, is valid YAML identifier

**Key reference files:**
- `crates/biome_service/src/file_handlers/javascript.rs:1069-1108` — JS rename implementation
- `crates/biome_service/src/workspace.rs:1126-1141` — `RenameParams` and `RenameResult` types
- `crates/biome_js_analyze/src/utils/rename.rs` — JS rename infrastructure
- `crates/biome_yaml_analyze/src/lint/nursery/no_unused_anchors.rs` — existing anchor/alias collection

---

#### Search Capability (GritQL)

Biome's `biome search` command uses GritQL — a structural pattern matching language that operates on ASTs, not text. It enables queries like "find all mappings where key is `apiVersion`" and returns precise text ranges.

**Current state:** `search: SearchCapabilities { search: None }` at `yaml.rs:238`. The path-level check returns `true` (YAML files are "searchable"), but no search function is wired — it's a placeholder.

**Who has it:** Only JavaScript and CSS. JSON also has `search: None`.

**Architecture:**
```
biome search '<pattern>'
  → CLI parses pattern into GritQuery (with target language)
  → Checks is_file_compatible_with_pattern()
  → Calls search(path, parse, query, settings)
  → GritQuery.execute() matches pattern against AST
  → Returns Vec<TextRange>
```

**What's missing — 4 components:**
1. `YamlTargetLanguage` — add to `generate_target_language!` macro in `crates/biome_grit_patterns/src/grit_target_language.rs:207-210` (currently only JS and CSS)
2. `GritYamlParser` — new parser converting YAML's rowan AST into Grit's internal tree representation (bulk of the work)
3. CLI compatibility check — add YAML arm to `is_file_compatible_with_pattern()` in `crates/biome_cli/src/execute/process_file/search.rs:79-89`
4. Wire up handler — change `search: None` to `search: Some(search)` in YAML file handler

**Why it's blocked:** The Grit pattern engine needs to understand YAML's AST node types. This requires either upstream Grit support for YAML or building a complete YAML-to-Grit AST mapping — significant work with unclear external dependency status.

---

#### Lexer `rewind()` — COMPLETE (Plan 21)

The `Lexer` trait (`crates/biome_parser/src/lexer.rs:50-51`) requires a `rewind()` method that restores the lexer to a previously saved checkpoint, enabling speculative parsing.

**Implemented:** `checkpoint()` and `rewind()` via `SavedLexerState` struct + `RefCell<Vec<SavedLexerState>>`. Captures/restores `current_coordinate`, `scopes` (block scope stack), `tokens` (VecDeque buffer), and diagnostics length. `LexerWithCheckpoint` trait implemented.

**Who uses it:** JS and CSS parsers rely on it for disambiguation (e.g., `()` could be grouping or arrow function start). HTML uses it internally in the lexer.

**Who else doesn't implement it:** GraphQL and Grit parsers also have `unimplemented!()`.

**Why YAML doesn't need it:** The YAML lexer uses a different architecture — it maintains a `VecDeque<LexToken>` token buffer and eagerly disambiguates tokens during lexing. The parser never calls `checkpoint()` or `rewind()` anywhere.

**What implementing it would require:** Capturing and restoring `current_coordinate` (offset + column), `scopes: Vec<BlockScope>` (indentation tracking), `tokens: VecDeque<LexToken>` (token buffer), and `diagnostics` state. Non-trivial due to stateful scope tracking.

**Impact:** None currently. Would only matter if speculative parsing patterns (`BufferedLexer` wrapping) or advanced error recovery via backtracking were ever needed. Lowest priority item.

### Cleanup

- ~~**Unused compact notation syntax kinds**~~ — Removed. Four ghost syntax kinds deleted from codegen and generated kind.rs.
