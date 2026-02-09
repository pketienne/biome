# Turtle Lint Rule Migration to Semantic Model

**Date:** 2026-02-09
**Status:** Complete
**Depends on:** `turtle-semantic-model-implementation-summary-2026-02-09.md`

## Overview

Migrated 7 lint rules from `Ast<TurtleRoot>` to `Semantic<TurtleRoot>`, leveraging the `biome_turtle_semantic` model to eliminate redundant AST walks, manual HashMap/HashSet tracking, and duplicated extraction logic. Rules now access pre-computed data via `ctx.model()` (provided through `RuleContext`'s `Deref` to `SemanticServices`).

## How It Works

When a rule declares `type Query = Semantic<TurtleRoot>`:
- `ctx.query()` returns the `TurtleRoot` AST node (full AST access preserved)
- `ctx.model()` returns the `SemanticModel` (via `Deref` to `SemanticServices`)
- The analyzer framework calls `SemanticServices::from_services()` to extract the model from the `ServiceBag`

## Rules Migrated

### 1. NoDuplicatePrefixDeclaration (has action)
- **Before:** Walk statements, extract namespace tokens, track in HashMap, detect duplicates, store directive node
- **After:** `model.duplicate_prefixes()` for detection; `find_prefix_directive(root, range)` to locate AST node for the removal action
- **Eliminated:** HashMap, namespace extraction logic

### 2. NoUnusedPrefix (has action)
- **Before:** Two passes — collect declarations into HashMap, walk all descendants to find usages into HashSet
- **After:** `model.unused_prefixes()` for detection (pre-computed); `find_prefix_directive(root, range)` for AST node
- **Eliminated:** Full descendant walk, HashMap + HashSet tracking

### 3. NoUndefinedPrefix (diagnostic only)
- **Before:** Two passes — collect declared prefixes into HashSet, walk all descendants to check prefixed names
- **After:** `model.prefix_map()` + `model.prefix_references()` — cross-reference in a single loop
- **Eliminated:** Both tree walks, HashSet

### 4. NoDuplicateTriple (diagnostic only)
- **Before:** Walk statements, call `extract_triples()` helper (complex function expanding predicate-object pairs), track in HashSet
- **After:** `model.duplicate_triples()` returns `&[(usize, usize)]` pairs; `model.triples()` for text formatting
- **Eliminated:** `extract_triples()` function, HashSet

### 5. UsePrefixedNames (diagnostic only)
- **Before:** Walk statements to build expansion→namespace HashMap, walk all tokens to find IRIREF literals, check against prefixes
- **After:** `model.expandable_iris()` — pre-computed with `suggested_prefixed` field
- **Eliminated:** Both walks, HashMap, local name validation

### 6. UseSortedPrefixes (diagnostic only)
- **Before:** Walk statements, extract namespace tokens, track previous namespace string
- **After:** `model.prefix_declarations()` returns ordered list; compare adjacent entries
- **Eliminated:** Statement walk, namespace extraction
- **Note:** Skips `is_duplicate` bindings to avoid comparing duplicate declarations

### 7. UseGroupedSubjects (diagnostic only)
- **Before:** Walk statements, call `extract_subject_text()` helper, track in HashMap
- **After:** `model.triples()` iterated once; track subjects by `statement_range` to distinguish different TurtleTriples blocks
- **Eliminated:** `extract_subject_text()` function, statement walk
- **Key fix:** Triples in the same block (using `;` notation) share a `statement_range` — must not be flagged as ungrouped

## Rules NOT Migrated

| Rule | Reason |
|------|--------|
| UseConsistentDirectiveStyle | Requires distinguishing `@prefix`/`PREFIX` syntax — model abstracts this away |
| UseConsistentQuotes | Quote style is purely syntactic |
| UseShorthandRdfType | Operates on `Ast<TurtleVerb>`, not `TurtleRoot`; would need query type change |
| NoInvalidIri | Character-level IRI validation; model stores strings but doesn't validate characters |
| NoInvalidLanguageTag | Language tags not captured by semantic model |
| NoMalformedDatatype | Datatype validation is syntax-level |
| NoLiteralTrimIssues | Whitespace detection is syntax-level |

## Test Infrastructure Changes

Updated all test entry points to build and pass the semantic model:

- **`tests/spec_tests.rs`** — Both `run_test()` and `run_suppression_test()` build `semantic_model(&root)` and pass `TurtleAnalyzerServices { semantic_model: Some(&model) }`
- **`tests/quick_test.rs`** — Same pattern
- **`src/lib.rs` unit tests** — Added `services_from()` helper that returns `(TurtleRoot, SemanticModel)`

## Files Modified

| File | Change |
|------|--------|
| `src/lint/nursery/no_duplicate_prefix_declaration.rs` | `Ast` → `Semantic`, use `model.duplicate_prefixes()` |
| `src/lint/nursery/no_unused_prefix.rs` | `Ast` → `Semantic`, use `model.unused_prefixes()` |
| `src/lint/nursery/no_undefined_prefix.rs` | `Ast` → `Semantic`, use `model.prefix_map()` + `prefix_references()` |
| `src/lint/nursery/no_duplicate_triple.rs` | `Ast` → `Semantic`, use `model.duplicate_triples()` |
| `src/lint/nursery/use_prefixed_names.rs` | `Ast` → `Semantic`, use `model.expandable_iris()` |
| `src/lint/nursery/use_sorted_prefixes.rs` | `Ast` → `Semantic`, use `model.prefix_declarations()` |
| `src/lint/nursery/use_grouped_subjects.rs` | `Ast` → `Semantic`, use `model.triples()` |
| `tests/spec_tests.rs` | Build and pass semantic model |
| `tests/quick_test.rs` | Build and pass semantic model |
| `src/lib.rs` | Update unit test helpers |

All paths above are relative to `crates/biome_turtle_analyze/`.

## Test Results

- **52** spec tests pass (all snapshots unchanged)
- **3** unit tests pass
- **11** semantic model unit tests pass
- **15** formatter tests pass (unaffected)

## Implementation Pattern

For rules **with actions** (need AST node for mutations):
```rust
fn run(ctx: &RuleContext<Self>) -> Self::Signals {
    let root = ctx.query();
    let model = ctx.model();
    for binding in model.duplicate_prefixes() {
        let directive = find_prefix_directive(root, binding.range);
        // store directive for action()
    }
}
```

For rules **without actions** (diagnostic only):
```rust
fn run(ctx: &RuleContext<Self>) -> Self::Signals {
    let model = ctx.model();
    model.expandable_iris()
        .filter_map(|iri_ref| { /* build signal from model data */ })
        .collect()
}
```
