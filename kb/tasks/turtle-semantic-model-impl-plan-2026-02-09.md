# Plan: Implement `biome_turtle_semantic` Crate

## Context

Six lint rules and multiple assists independently walk the full AST to collect prefix declarations, track usage, extract triples, and resolve IRIs. A semantic model centralizes this computation (single AST walk), eliminates redundancy, and enables future LSP features (rename, go-to-definition). This follows the established CSS semantic model pattern (`biome_css_semantic`).

This plan covers Steps 1–6 of the semantic model plan at `kb/tasks/turtle-semantic-model-plan-2026-02-09.md`. Rule migration (Step 7) and LSP features (Step 8) are deferred to follow-up work.

---

## Step 1: Create the `biome_turtle_semantic` crate

Create `crates/biome_turtle_semantic/` with this structure:

```
crates/biome_turtle_semantic/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public semantic_model() function
│   ├── events.rs           # SemanticEvent enum + SemanticEventExtractor
│   └── semantic_model/
│       ├── mod.rs           # Re-exports
│       ├── model.rs         # SemanticModel, SemanticModelData, query methods
│       └── builder.rs       # SemanticModelBuilder
```

**`Cargo.toml` dependencies** (following `biome_css_semantic/Cargo.toml`):
```toml
[dependencies]
biome_turtle_syntax = { workspace = true }
biome_rowan         = { workspace = true }
rustc-hash          = { workspace = true }

[dev-dependencies]
biome_turtle_parser = { path = "../biome_turtle_parser" }
```

---

## Step 2: Implement event extraction (`events.rs`)

**SemanticEvent** enum:
```rust
pub enum SemanticEvent {
    PrefixDeclaration { namespace: String, expansion: String, range: TextRange },
    BaseDeclaration { iri: String, range: TextRange },
    Triple { subject: String, predicate: String, object: String,
             subject_range: TextRange, predicate_range: TextRange,
             object_range: TextRange, statement_range: TextRange },
    PrefixReference { namespace: String, local_name: String, range: TextRange },
    IriReference { iri: String, range: TextRange },
}
```

**SemanticEventExtractor**: Walks AST via `enter()`/`leave()` pattern (matching on `TurtleSyntaxKind`):
- `TurtlePrefixDeclaration` / `TurtleSparqlPrefixDeclaration` → emit `PrefixDeclaration`
- `TurtleBaseDeclaration` / `TurtleSparqlBaseDeclaration` → emit `BaseDeclaration`
- `TurtleTriples` → expand predicate-object pairs × objects, emit `Triple` for each
- `TurtlePrefixedName` → emit `PrefixReference` (extract namespace and local_name from token text)
- `TurtleIri` with IRIREF token → emit `IriReference`

Reference: `crates/biome_css_semantic/src/events.rs` (same enter/leave/pop pattern)

---

## Step 3: Implement builder (`semantic_model/builder.rs`)

**SemanticModelBuilder** processes events into indexed data:
- Accumulates `PrefixBinding` entries, detects duplicates (same namespace declared twice)
- Builds `prefix_map` (namespace → expansion) and `reverse_prefix_map` (expansion → namespace)
- Collects `TripleInfo` entries, indexes by subject text, detects duplicate triples (same s/p/o text)
- Tracks prefix references, marks which prefixes are used
- Checks IRI references against reverse prefix map for contractability (IRI that could be a prefixed name)

Key types:
```rust
pub struct PrefixBinding { pub namespace: String, pub expansion: String, pub range: TextRange, pub is_duplicate: bool }
pub struct TripleInfo { pub subject: String, pub predicate: String, pub object: String,
                        pub statement_range: TextRange, pub is_rdf_type: bool }
pub struct PrefixRef { pub namespace: String, pub local_name: String, pub range: TextRange }
pub struct IriRef { pub iri: String, pub range: TextRange, pub suggested_prefixed: Option<String> }
```

---

## Step 4: Implement model queries (`semantic_model/model.rs`)

**SemanticModel** (thread-safe façade, following CSS pattern):
```rust
pub struct SemanticModel {
    pub(crate) data: Arc<SemanticModelData>,
    root: SendNode,
}
```

**Public API methods:**
- `prefix_map()` → `&FxHashMap<String, String>` (namespace → expansion)
- `resolve_prefix(namespace)` → `Option<&str>` (expansion for a prefix)
- `contract_iri(iri)` → `Option<String>` (IRI → prefixed name)
- `prefix_declarations()` → `&[PrefixBinding]`
- `unused_prefixes()` → iterator over unused `PrefixBinding`s
- `duplicate_prefixes()` → iterator over duplicate `PrefixBinding`s
- `is_prefix_used(namespace)` → `bool`
- `triples()` → `&[TripleInfo]`
- `triples_for_subject(subject)` → `&[usize]` (indices into triples)
- `duplicate_triples()` → `&[(usize, usize)]`
- `expandable_iris()` → iterator over `IriRef`s that could be prefixed

Reference: `crates/biome_css_semantic/src/semantic_model/model.rs`

---

## Step 5: Public API (`lib.rs`)

```rust
pub fn semantic_model(root: &TurtleRoot) -> SemanticModel {
    let mut extractor = SemanticEventExtractor::default();
    let mut builder = SemanticModelBuilder::new(root.clone());
    for node in root.syntax().preorder() {
        match node {
            WalkEvent::Enter(n) => extractor.enter(&n),
            WalkEvent::Leave(n) => extractor.leave(&n),
        }
    }
    while let Some(e) = extractor.pop() {
        builder.push_event(e);
    }
    builder.build()
}
```

Reference: `crates/biome_css_semantic/src/lib.rs` (line 1-9, exact same pattern)

---

## Step 6: Analyzer integration

### 6a. Create `crates/biome_turtle_analyze/src/services/semantic.rs`

Copy the CSS pattern from `crates/biome_css_analyze/src/services/semantic.rs`:
- `SemanticServices` struct with `model: SemanticModel`
- `FromServices` impl: extract model from `ServiceBag`
- `Phase` impl: returns `Phases::Syntax`
- `Semantic<N>` queryable type for rules to use

### 6b. Modify `crates/biome_turtle_analyze/src/lib.rs`

- Add `mod services;` module
- Add `TurtleAnalyzerServices` struct:
  ```rust
  #[derive(Debug, Clone, Default)]
  pub struct TurtleAnalyzerServices<'a> {
      pub semantic_model: Option<&'a SemanticModel>,
  }
  ```
- Update `analyze()` and `analyze_with_inspect_matcher()` signatures to accept `TurtleAnalyzerServices`
- After `registry.build()`, insert semantic model into `services`:
  ```rust
  if let Some(semantic_model) = turtle_services.semantic_model {
      services.insert_service(semantic_model.clone());
  }
  ```

### 6c. Modify `crates/biome_service/src/workspace/document.rs`

- Add `Turtle(TurtleDocumentServices)` variant to `DocumentServices` enum
- Add `TurtleDocumentServices` struct with `semantic_model: Option<SemanticModel>`
- Add `with_turtle_semantic_model()`, `new_turtle()`, `as_turtle_services()` methods
- Add `From<TurtleDocumentServices>` impl

### 6d. Modify `crates/biome_service/src/file_handlers/turtle.rs`

- Update `lint()`: build `TurtleAnalyzerServices` from `params.document_services`, pass to `analyze()`
- Update `code_actions()`: same — extract semantic model from document_services, pass to `analyze()`
- Update `fix_all()`: same pattern
- Add `debug_semantic_model` capability

### 6e. Modify `crates/biome_service/src/workspace/server.rs`

- In the file opening/parsing paths where CSS builds its model (search for `CssDocumentServices`), add Turtle equivalent:
  ```rust
  if document_source.is_turtle_like()
      && (settings.is_linter_enabled() || settings.is_assist_enabled())
  {
      services = TurtleDocumentServices::default()
          .with_turtle_semantic_model(&any_parse.tree())
          .into();
  }
  ```

---

## Files to Create

| File | Purpose |
|------|---------|
| `crates/biome_turtle_semantic/Cargo.toml` | Crate manifest |
| `crates/biome_turtle_semantic/src/lib.rs` | Public `semantic_model()` function |
| `crates/biome_turtle_semantic/src/events.rs` | `SemanticEvent` + `SemanticEventExtractor` |
| `crates/biome_turtle_semantic/src/semantic_model/mod.rs` | Re-exports |
| `crates/biome_turtle_semantic/src/semantic_model/model.rs` | `SemanticModel` + `SemanticModelData` |
| `crates/biome_turtle_semantic/src/semantic_model/builder.rs` | `SemanticModelBuilder` |
| `crates/biome_turtle_analyze/src/services/mod.rs` | Services module |
| `crates/biome_turtle_analyze/src/services/semantic.rs` | `SemanticServices` + `Semantic<N>` |

## Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` (workspace root) | Add `biome_turtle_semantic` to workspace members |
| `crates/biome_turtle_analyze/Cargo.toml` | Add `biome_turtle_semantic` dependency |
| `crates/biome_turtle_analyze/src/lib.rs` | Add services module, `TurtleAnalyzerServices`, update `analyze()` signature |
| `crates/biome_service/Cargo.toml` | Add `biome_turtle_semantic` dependency |
| `crates/biome_service/src/workspace/document.rs` | Add `Turtle` variant and `TurtleDocumentServices` |
| `crates/biome_service/src/file_handlers/turtle.rs` | Pass services to analyzer, add `debug_semantic_model` |
| `crates/biome_service/src/workspace/server.rs` | Build semantic model when opening/parsing Turtle files |

---

## Verification

1. `cargo build -p biome_turtle_semantic` — crate compiles
2. Unit tests in `biome_turtle_semantic` — test prefix collection, triple extraction, duplicate detection, IRI contractability
3. `cargo build -p biome_turtle_analyze` — analyzer compiles with new services
4. `cargo test -p biome_turtle_analyze` — all 52 existing tests still pass (rules don't use semantic model yet, so services are optional)
5. `cargo build -p biome_service` — service layer compiles
6. `cargo test -p biome_turtle_formatter` — formatter unaffected (15 tests still pass)
