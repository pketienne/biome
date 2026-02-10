# Plan 19: YAML Semantic Model (`biome_yaml_semantic`)

## Goal

Create a `biome_yaml_semantic` crate that pre-computes anchor/alias bindings from a single syntax tree traversal. This eliminates redundant traversals across the 4 anchor-related lint rules and enables rename and go-to-definition capabilities.

## Architecture

Follow `biome_graphql_semantic` as template. Three-phase pipeline:

```
preorder walk → SemanticEventExtractor → SemanticModelBuilder → SemanticModel
```

### Phase 1: Event Extraction (`events.rs`)

`SemanticEventExtractor` walks the syntax tree and emits events:

```rust
pub enum SemanticEvent {
    /// Anchor declaration: &name
    AnchorDeclaration {
        name: String,
        range: TextRange,      // range of the ANCHOR_PROPERTY_LITERAL token
        document_index: usize, // which YAML document (for multi-doc scoping)
    },
    /// Alias reference: *name
    AliasReference {
        name: String,
        range: TextRange,      // range of the ALIAS_LITERAL token
        document_index: usize,
    },
    /// New document boundary (---)
    DocumentStart { index: usize },
}
```

The extractor tracks document boundaries (`---` / `...`) to scope anchors per-document.

### Phase 2: Model Building (`semantic_model/builder.rs`)

`SemanticModelBuilder` consumes events and populates indexed data structures:

```rust
pub struct SemanticModelBuilder {
    root: YamlRoot,
    node_by_range: FxHashMap<TextRange, YamlSyntaxNode>,
    anchors: Vec<AnchorBinding>,
    anchors_by_name: FxHashMap<String, Vec<usize>>,
    anchors_by_start: FxHashMap<TextSize, usize>,
    aliases: Vec<AliasReference>,
    aliases_by_start: FxHashMap<TextSize, usize>,
    anchor_to_aliases: Vec<Vec<usize>>,
    alias_to_anchor: Vec<Option<usize>>,
    unresolved_aliases: Vec<UnresolvedAlias>,
    duplicate_anchors: Vec<DuplicateAnchor>,
    documents: Vec<DocumentScope>,
}
```

Resolution: for each alias, find matching anchor in the same document. Record unresolved aliases and duplicate anchors.

### Phase 3: Model (`semantic_model/model.rs`)

`SemanticModel` wraps `SemanticModelData` in `Rc` for zero-copy sharing:

```rust
pub struct SemanticModel {
    data: Rc<SemanticModelData>,
}

impl SemanticModel {
    pub fn all_anchors(&self) -> impl Iterator<Item = Anchor>;
    pub fn all_aliases(&self) -> impl Iterator<Item = Alias>;
    pub fn anchor_by_name(&self, name: &str, doc_index: usize) -> Option<Anchor>;
    pub fn aliases_for_anchor(&self, anchor: &Anchor) -> Vec<Alias>;
    pub fn anchor_for_alias(&self, alias: &Alias) -> Option<Anchor>;
    pub fn all_unresolved_aliases(&self) -> impl Iterator<Item = UnresolvedAlias>;
    pub fn all_duplicate_anchors(&self) -> impl Iterator<Item = DuplicateAnchor>;
    pub fn as_anchor(&self, node: &YamlAnchorProperty) -> Option<Anchor>;
}
```

### Supporting Types

**`semantic_model/binding.rs`** — `AnchorBinding` (internal) and `Anchor` (public, holds `Rc<Data>`):
```rust
pub(crate) struct AnchorBinding {
    pub index: usize,
    pub name: String,
    pub range: TextRange,
    pub document_index: usize,
}

pub struct Anchor {
    pub(crate) data: Rc<SemanticModelData>,
    pub(crate) index: usize,
}
```

**`semantic_model/reference.rs`** — `AliasRef` (internal) and `Alias` (public):
```rust
pub(crate) struct AliasRef {
    pub index: usize,
    pub name: String,
    pub range: TextRange,
    pub document_index: usize,
}

pub struct Alias {
    pub(crate) data: Rc<SemanticModelData>,
    pub(crate) index: usize,
}
```

### Top-level function (`semantic_model/mod.rs`)

```rust
pub fn semantic_model(root: &YamlRoot) -> SemanticModel {
    let mut extractor = SemanticEventExtractor::default();
    let mut builder = SemanticModelBuilder::new(root.clone());
    for node in root.syntax().preorder() {
        match node {
            WalkEvent::Enter(node) => {
                builder.push_node(&node);
                extractor.enter(&node);
            }
            WalkEvent::Leave(node) => extractor.leave(&node),
        }
    }
    while let Some(e) = extractor.pop() {
        builder.push_event(e);
    }
    builder.build()
}
```

## Crate Setup

**`crates/biome_yaml_semantic/Cargo.toml`:**
```toml
[package]
name = "biome_yaml_semantic"
version = "0.0.0"
# ... workspace fields ...

[dependencies]
biome_yaml_syntax = { workspace = true }
biome_rowan = { workspace = true }
rustc-hash = { workspace = true }

[dev-dependencies]
biome_yaml_parser = { path = "../biome_yaml_parser" }

[lints]
workspace = true
```

**`crates/biome_yaml_semantic/src/lib.rs`:**
```rust
mod events;
mod semantic_model;
pub use events::*;
pub use semantic_model::*;
```

## Files to Create

| File | Purpose |
|------|---------|
| `crates/biome_yaml_semantic/Cargo.toml` | Crate manifest |
| `crates/biome_yaml_semantic/src/lib.rs` | Re-exports |
| `crates/biome_yaml_semantic/src/events.rs` | SemanticEventExtractor |
| `crates/biome_yaml_semantic/src/semantic_model/mod.rs` | Top-level `semantic_model()` fn |
| `crates/biome_yaml_semantic/src/semantic_model/model.rs` | SemanticModel + SemanticModelData |
| `crates/biome_yaml_semantic/src/semantic_model/builder.rs` | SemanticModelBuilder |
| `crates/biome_yaml_semantic/src/semantic_model/binding.rs` | Anchor types |
| `crates/biome_yaml_semantic/src/semantic_model/reference.rs` | Alias types |

## Integration with Lint Rules

After the crate is built, refactor these 4 rules to use the semantic model instead of manual tree traversal:
- `noDuplicateAnchors` — use `model.all_duplicate_anchors()`
- `noUndeclaredAliases` — use `model.all_unresolved_aliases()`
- `noUnusedAnchors` — use `model.all_anchors()` + check `model.aliases_for_anchor()` is empty
- `useValidMergeKeys` — use `model.anchor_for_alias()` to validate merge key targets

**Note:** Integrating the semantic model as an analyzer service (like GraphQL does) requires registering it via `ServiceBag`. This is deferred — the initial implementation provides the crate and a standalone `semantic_model()` function. Rules can call it directly from `run()` as they currently traverse the tree anyway. Service integration can follow as a separate step.

## Verification

1. `cargo check -p biome_yaml_semantic` compiles
2. Unit tests for event extraction and model building
3. Existing lint rule tests still pass after refactoring
4. `cargo clippy -p biome_yaml_semantic` — zero warnings
