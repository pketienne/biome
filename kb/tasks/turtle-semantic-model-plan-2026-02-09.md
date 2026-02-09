# Plan: Turtle Semantic Model (`biome_turtle_semantic`)

## Date: 2026-02-09
## Prerequisite: None
## Depends on: Existing parser + syntax crates

---

## Motivation

Six lint rules and three assists independently walk the full AST to collect prefix declarations, track prefix usage, extract triples, and resolve IRIs. A semantic model centralizes this computation, eliminates redundancy, and enables LSP features (rename, go-to-definition, find-references) that are currently impossible.

### Redundant Computations Today

| Computation | Performed by (6+ locations) |
|-------------|----------------------------|
| Collect prefix declarations | `noUndefinedPrefix`, `noDuplicatePrefixDeclaration`, `noUnusedPrefix`, `usePrefixedNames`, `convertIriToPrefixedName`, `removeUnusedPrefixes` |
| Track prefix usage | `noUndefinedPrefix`, `noUnusedPrefix`, `removeUnusedPrefixes` |
| IRI ↔ prefixed name resolution | `usePrefixedNames`, `convertIriToPrefixedName` |
| Extract triples as tuples | `noDuplicateTriple`, `mergeTriples` |
| Group triples by subject | `useGroupedSubjects`, `mergeTriples` |
| Resolve rdf:type variants | `useShorthandRdfType`, `convertRdfTypeToShorthand` |

---

## Architecture

Follow the established Biome pattern (CSS/GraphQL semantic models):

```
biome_turtle_semantic/
├── src/
│   ├── lib.rs                    # Public API: semantic_model(&TurtleRoot) -> SemanticModel
│   ├── events.rs                 # SemanticEvent enum + SemanticEventExtractor
│   └── semantic_model/
│       ├── mod.rs                # Re-exports
│       ├── model.rs              # SemanticModel + SemanticModelData
│       └── builder.rs            # SemanticModelBuilder
├── Cargo.toml
└── tests/
```

### Phase 1: Event Extraction

Walk the AST once, emit events:

```rust
pub enum SemanticEvent {
    /// @prefix or PREFIX declaration
    PrefixDeclaration {
        namespace: String,      // e.g., "foaf:"
        expansion: String,      // e.g., "http://xmlns.com/foaf/0.1/"
        range: TextRange,
    },
    /// @base or BASE declaration
    BaseDeclaration {
        iri: String,
        range: TextRange,
    },
    /// A complete (subject, predicate, object) triple
    Triple {
        subject: String,        // text_trimmed of subject node
        predicate: String,      // text_trimmed of verb node
        object: String,         // text_trimmed of object node
        subject_range: TextRange,
        predicate_range: TextRange,
        object_range: TextRange,
        statement_range: TextRange,
    },
    /// A prefixed name reference (e.g., foaf:name)
    PrefixReference {
        namespace: String,      // "foaf:"
        local_name: String,     // "name"
        range: TextRange,
    },
    /// A full IRI reference (e.g., <http://...>)
    IriReference {
        iri: String,
        range: TextRange,
    },
}
```

### Phase 2: Model Building

Process events into indexed data:

```rust
pub struct SemanticModelData {
    // Prefix resolution
    pub(crate) prefix_declarations: Vec<PrefixBinding>,
    pub(crate) prefix_map: FxHashMap<String, String>,        // namespace -> expansion
    pub(crate) reverse_prefix_map: FxHashMap<String, String>, // expansion -> namespace
    pub(crate) base_uri: Option<String>,

    // Prefix usage tracking
    pub(crate) prefix_references: Vec<PrefixRef>,
    pub(crate) used_prefixes: FxHashSet<String>,
    pub(crate) unused_prefixes: Vec<usize>,                  // indices into prefix_declarations

    // Triple index
    pub(crate) triples: Vec<TripleInfo>,
    pub(crate) triples_by_subject: FxHashMap<String, Vec<usize>>,
    pub(crate) duplicate_triples: Vec<(usize, usize)>,       // pairs of duplicate triple indices

    // IRI references
    pub(crate) iri_references: Vec<IriRef>,
    pub(crate) expandable_iris: Vec<usize>,                  // indices into iri_references that could be prefixed

    // Lookups
    pub(crate) node_by_range: FxHashMap<TextRange, TurtleSyntaxNode>,
}
```

### Key Types

```rust
pub struct PrefixBinding {
    pub namespace: String,
    pub expansion: String,
    pub range: TextRange,
    pub is_duplicate: bool,
}

pub struct PrefixRef {
    pub namespace: String,
    pub local_name: String,
    pub range: TextRange,
    pub binding_index: Option<usize>,  // into prefix_declarations
}

pub struct TripleInfo {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub statement_range: TextRange,
    pub is_rdf_type: bool,             // predicate is rdf:type or `a`
}

pub struct IriRef {
    pub iri: String,
    pub range: TextRange,
    pub suggested_prefixed: Option<String>,  // e.g., "foaf:name"
}
```

### Public API

```rust
pub struct SemanticModel {
    data: Arc<SemanticModelData>,
    root: SendNode,
}

impl SemanticModel {
    // Prefix resolution
    pub fn prefix_map(&self) -> &FxHashMap<String, String>;
    pub fn resolve_prefix(&self, namespace: &str) -> Option<&str>;
    pub fn contract_iri(&self, iri: &str) -> Option<String>;
    pub fn expand_prefixed_name(&self, prefixed: &str) -> Option<String>;
    pub fn base_uri(&self) -> Option<&str>;

    // Prefix analysis
    pub fn prefix_declarations(&self) -> &[PrefixBinding];
    pub fn unused_prefixes(&self) -> impl Iterator<Item = &PrefixBinding>;
    pub fn duplicate_prefixes(&self) -> impl Iterator<Item = &PrefixBinding>;
    pub fn is_prefix_used(&self, namespace: &str) -> bool;

    // Triple access
    pub fn triples(&self) -> &[TripleInfo];
    pub fn triples_for_subject(&self, subject: &str) -> &[usize];
    pub fn duplicate_triples(&self) -> &[(usize, usize)];

    // IRI analysis
    pub fn expandable_iris(&self) -> impl Iterator<Item = &IriRef>;

    // Reference resolution (for LSP)
    pub fn binding_for_reference(&self, range: TextRange) -> Option<&PrefixBinding>;
    pub fn references_for_binding(&self, binding_range: TextRange) -> Vec<&PrefixRef>;
}
```

---

## Implementation Steps

### Step 1: Create the crate

```
cargo new crates/biome_turtle_semantic --lib
```

**`Cargo.toml` dependencies:**
```toml
[dependencies]
biome_rowan         = { workspace = true }
biome_turtle_syntax = { workspace = true }
rustc-hash          = { workspace = true }
```

**Files to create:**
- `src/lib.rs` — public `semantic_model()` function
- `src/events.rs` — `SemanticEvent` enum, `SemanticEventExtractor`
- `src/semantic_model/mod.rs` — re-exports
- `src/semantic_model/model.rs` — `SemanticModel`, `SemanticModelData`, query methods
- `src/semantic_model/builder.rs` — `SemanticModelBuilder`

### Step 2: Implement event extraction

`SemanticEventExtractor` walks the AST via `enter()`/`leave()` pattern:
- On `TurtlePrefixDeclaration` / `TurtleSparqlPrefixDeclaration`: emit `PrefixDeclaration`
- On `TurtleBaseDeclaration` / `TurtleSparqlBaseDeclaration`: emit `BaseDeclaration`
- On `TurtleTriples`: expand predicate-object pairs × objects, emit `Triple` for each
- On `TurtlePrefixedName`: emit `PrefixReference`
- On `TurtleIri` with IRIREF literal: emit `IriReference`

### Step 3: Implement builder

`SemanticModelBuilder` processes events:
- Accumulates prefix declarations, detects duplicates
- Builds prefix map and reverse map
- Collects triples, indexes by subject, detects duplicates
- Resolves prefix references to bindings
- Checks IRIs against reverse prefix map for contractability

### Step 4: Implement model queries

Public API methods on `SemanticModel` providing read-only access to computed data.

### Step 5: Integration — Analyzer services

**New file: `crates/biome_turtle_analyze/src/services/semantic.rs`**

```rust
pub struct SemanticServices {
    model: SemanticModel,
}

impl FromServices for SemanticServices { ... }
impl Phase for SemanticServices { ... }

pub struct Semantic<N>(pub N);
impl<N> Queryable for Semantic<N> { ... }
```

**Modify: `crates/biome_turtle_analyze/src/lib.rs`**
- Add `TurtleAnalyzerServices` struct with optional `&SemanticModel`
- Update `analyze()` to accept services

### Step 6: Integration — Service layer

**Modify: `crates/biome_service/src/workspace/document.rs`**
- Add `TurtleDocumentServices` with `semantic_model: Option<SemanticModel>`
- Add `with_turtle_semantic_model()` builder method

**Modify: `crates/biome_service/src/file_handlers/turtle.rs`**
- Build semantic model in lint/code_actions paths
- Pass to analyzer via services
- Implement `debug_semantic_model` for debugging

### Step 7: Migrate existing rules

Migrate rules one at a time to use `Semantic<N>` queries and `ctx.model()`:

| Rule | Migration | Benefit |
|------|-----------|---------|
| `noUndefinedPrefix` | Use `model.is_prefix_used()` | Eliminate AST walk |
| `noDuplicatePrefixDeclaration` | Use `model.duplicate_prefixes()` | Eliminate AST walk |
| `noUnusedPrefix` | Use `model.unused_prefixes()` | Eliminate 2 AST walks |
| `noDuplicateTriple` | Use `model.duplicate_triples()` | Eliminate triple extraction |
| `usePrefixedNames` | Use `model.expandable_iris()` | Eliminate prefix collection + IRI scan |
| `useGroupedSubjects` | Use `model.triples_for_subject()` | Eliminate subject grouping |
| `useShorthandRdfType` | Use `triple.is_rdf_type` | Minor simplification |

**Assists to migrate:**
| Assist | Migration |
|--------|-----------|
| `convertIriToPrefixedName` | Use `model.expandable_iris()` |
| `removeUnusedPrefixes` | Use `model.unused_prefixes()` |
| `mergeTriples` | Use `model.triples_for_subject()` |
| `convertRdfTypeToShorthand` | Use `model.triples()` + `is_rdf_type` filter |

### Step 8: LSP features (optional, after model works)

With bindings and references tracked:
- **Go-to-definition**: From prefixed name → prefix declaration
- **Find references**: From prefix declaration → all usages
- **Rename**: Rename prefix namespace across all usages

Requires implementing `rename()` and related methods in `turtle.rs` file handler.

---

## Testing Strategy

1. **Unit tests in `biome_turtle_semantic`**: Test model building from sample documents
   - Prefix resolution correctness
   - Triple extraction and deduplication
   - IRI contractability
   - Unused/duplicate prefix detection
2. **Migration tests**: Ensure existing lint rule snapshots don't change after migration
3. **LSP integration tests**: Test go-to-definition, find-references (if implemented)

---

## Estimated Scope

| Step | Files | Complexity |
|------|-------|------------|
| 1. Create crate | 5 new | Low |
| 2. Event extraction | 1 file | Medium |
| 3. Builder | 1 file | Medium |
| 4. Model queries | 1 file | Low |
| 5. Analyzer services | 2 files (1 new, 1 modified) | Medium |
| 6. Service layer | 2 files modified | Low |
| 7. Migrate rules | 10+ files modified | Medium (per rule) |
| 8. LSP features | 1-2 files modified | High |

---

## Reference Files

| Biome Pattern | Location |
|---------------|----------|
| CSS semantic model | `crates/biome_css_semantic/` |
| GraphQL semantic model | `crates/biome_graphql_semantic/` |
| CSS analyzer services | `crates/biome_css_analyze/src/services/semantic.rs` |
| CSS document services | `crates/biome_service/src/workspace/document.rs` |
| CSS file handler (model usage) | `crates/biome_service/src/file_handlers/css.rs` |
| Turtle file handler | `crates/biome_service/src/file_handlers/turtle.rs` |
