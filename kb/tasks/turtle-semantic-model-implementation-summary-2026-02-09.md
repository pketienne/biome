# Turtle Semantic Model Implementation Summary

**Date:** 2026-02-09
**Status:** Complete (Steps 1-6)
**Plan:** `turtle-semantic-model-impl-plan-2026-02-09.md`

## What Was Built

### New Crate: `biome_turtle_semantic`

Created `crates/biome_turtle_semantic/` following the `biome_css_semantic` pattern.

#### `src/events.rs` — Event Extraction

`SemanticEvent` enum with variants:
- `PrefixDeclaration` — `@prefix` / `PREFIX` declarations
- `BaseDeclaration` — `@base` / `BASE` declarations
- `Triple` — expanded (subject, predicate, object) triples
- `PrefixReference` — prefixed name usage (e.g., `foaf:Person`)
- `IriReference` — full IRI references (e.g., `<http://...>`)

`SemanticEventExtractor` walks AST via `enter()`/`leave()` pattern, matching on `TurtleSyntaxKind`. Handles both Turtle (`@prefix`/`@base`) and SPARQL (`PREFIX`/`BASE`) forms. Expands predicate-object pairs x objects into individual Triple events.

#### `src/semantic_model/builder.rs` — Model Construction

`SemanticModelBuilder` processes events into indexed data:
- Builds `prefix_map` (namespace -> expansion) and `reverse_prefix_map`
- Detects duplicate prefix declarations (same namespace declared twice)
- Collects `TripleInfo` entries, indexes by subject, detects duplicate triples
- Tracks prefix references, marks which prefixes are used
- Checks IRI references against reverse prefix map for contractability

#### `src/semantic_model/model.rs` — Query Facade

`SemanticModel` (thread-safe via `Arc<SemanticModelData>` + `SendNode`):

| Method | Description |
|--------|-------------|
| `prefix_map()` | namespace -> expansion map |
| `resolve_prefix(ns)` | resolve a prefix to its expansion IRI |
| `contract_iri(iri)` | IRI -> prefixed name (e.g., `foaf:Person`) |
| `expand_prefixed_name(pn)` | prefixed name -> full IRI |
| `base_uri()` | the `@base` URI if declared |
| `prefix_declarations()` | all prefix declarations in document order |
| `unused_prefixes()` | prefixes declared but never referenced |
| `duplicate_prefixes()` | prefixes declared more than once |
| `is_prefix_used(ns)` | whether a prefix namespace is referenced |
| `triples()` | all extracted triples |
| `triples_for_subject(s)` | triple indices for a given subject |
| `duplicate_triples()` | pairs of (first, duplicate) triple indices |
| `expandable_iris()` | IRIs that could be contracted to prefixed names |
| `prefix_references()` | all prefixed name references |
| `iri_references()` | all full IRI references |

Key types: `PrefixBinding`, `TripleInfo`, `PrefixRef`, `IriRef`

#### `src/lib.rs` — Public API

```rust
pub fn semantic_model(root: &TurtleRoot) -> SemanticModel
```

Single AST walk, event extraction, then builder construction. Plus 11 unit tests covering all major functionality.

### Analyzer Integration

#### `crates/biome_turtle_analyze/src/services/semantic.rs`

- `SemanticServices` struct with `model: SemanticModel`
- `FromServices` impl extracting model from `ServiceBag`
- `Semantic<N>` queryable type for rules (same pattern as CSS)

#### `crates/biome_turtle_analyze/src/lib.rs`

- Added `TurtleAnalyzerServices` struct with `semantic_model: Option<&'a SemanticModel>`
- Updated `analyze()` and `analyze_with_inspect_matcher()` to accept services parameter
- Inserts semantic model into `ServiceBag` when provided

### Service Layer Integration

#### `crates/biome_service/src/workspace/document.rs`

- Added `Turtle(TurtleDocumentServices)` variant to `DocumentServices` enum
- `TurtleDocumentServices` holds `Option<SemanticModel>`
- `with_turtle_semantic_model()` builds model from `TurtleRoot`

#### `crates/biome_service/src/file_handlers/turtle.rs`

- `lint()`, `code_actions()`, `fix_all()` extract semantic model from document services and pass to analyzer
- Added `debug_semantic_model` capability

#### `crates/biome_service/src/workspace/server.rs`

- Builds semantic model when opening/parsing Turtle files (adjacent to CSS model building)

## Files Created

| File | Purpose |
|------|---------|
| `crates/biome_turtle_semantic/Cargo.toml` | Crate manifest |
| `crates/biome_turtle_semantic/src/lib.rs` | Public API + 11 unit tests |
| `crates/biome_turtle_semantic/src/events.rs` | SemanticEvent + SemanticEventExtractor |
| `crates/biome_turtle_semantic/src/semantic_model/mod.rs` | Re-exports |
| `crates/biome_turtle_semantic/src/semantic_model/model.rs` | SemanticModel + SemanticModelData |
| `crates/biome_turtle_semantic/src/semantic_model/builder.rs` | SemanticModelBuilder |
| `crates/biome_turtle_analyze/src/services/mod.rs` | Services module |
| `crates/biome_turtle_analyze/src/services/semantic.rs` | SemanticServices + Semantic<N> |

## Files Modified

| File | Change |
|------|--------|
| `Cargo.toml` (workspace root) | Added `biome_turtle_semantic` workspace dependency |
| `crates/biome_turtle_analyze/Cargo.toml` | Added `biome_turtle_semantic` dependency |
| `crates/biome_turtle_analyze/src/lib.rs` | Added services module, `TurtleAnalyzerServices`, updated `analyze()` |
| `crates/biome_turtle_analyze/tests/quick_test.rs` | Updated `analyze()` call signature |
| `crates/biome_turtle_analyze/tests/spec_tests.rs` | Updated `analyze()` call signatures (2 locations) |
| `crates/biome_service/Cargo.toml` | Added `biome_turtle_semantic` dependency |
| `crates/biome_service/src/workspace/document.rs` | Added Turtle variant + TurtleDocumentServices |
| `crates/biome_service/src/workspace.rs` | Re-exported TurtleDocumentServices |
| `crates/biome_service/src/file_handlers/turtle.rs` | Pass services to analyzer, debug_semantic_model |
| `crates/biome_service/src/workspace/server.rs` | Build semantic model on file open/parse |

## Test Results

- **11** semantic model unit tests pass (prefix collection, duplicate detection, usage tracking, triple extraction, IRI contraction, expansion, base URI, SPARQL style, rdf:type detection)
- **52** analyzer spec tests pass (unchanged behavior — rules don't use semantic model yet)
- **15** formatter tests pass (unaffected)

## Implementation Notes

- Namespace keys include trailing colon (e.g., `"foaf:"` not `"foaf"`) — matches how `namespace_token()` returns text
- `SendNode` created via `syntax().as_send()` (no `SendNode::new()` constructor)
- `AstSeparatedList` trait must be imported for `.elements()` on separated lists
- `is_rdf_type` detection checks for `"a"`, `"rdf:type"`, and full IRI form

## Next Steps (Deferred)

- **Step 7:** Migrate existing lint rules to use `Semantic<N>` queries instead of walking AST independently
- **Step 8:** LSP features (rename prefix, go-to-definition for prefixed names)
