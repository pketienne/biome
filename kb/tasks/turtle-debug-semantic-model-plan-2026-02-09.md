# Plan: Enhance `debug_semantic_model` Output

**Date:** 2026-02-09
**Status:** To implement
**Effort:** Quick win (< 2 hours)

## Context

The Turtle `debug_semantic_model` handler currently uses `format!("{model:#?}")` (Rust Debug derive), which dumps raw internal struct fields. CSS and JS implementations use a custom `Display` impl via `biome_formatter` for readable output. For Turtle, a simpler approach — a hand-written `Display` impl — provides clear, human-readable output without the overhead of a full formatter integration.

## Implementation

Add a `Display` impl for `SemanticModel` in `crates/biome_turtle_semantic/src/semantic_model/model.rs` that outputs:

```
=== Turtle Semantic Model ===

Base URI: <http://example.org/>

Prefix Declarations (3):
  foaf: -> http://xmlns.com/foaf/0.1/
  dc:   -> http://purl.org/dc/elements/1.1/  [unused]
  ex:   -> http://example.org/               [duplicate]

Triples (5):
  <http://example.org/alice> a foaf:Person
  <http://example.org/alice> foaf:name "Alice"
  ...

Duplicate Triples (1):
  #0 == #3: <http://example.org/alice> a foaf:Person

Expandable IRIs (2):
  <http://xmlns.com/foaf/0.1/Person> -> foaf:Person
  ...
```

Then update `turtle.rs` to use `model.to_string()` instead of `format!("{model:#?}")`.

## Files to Modify

| File | Change |
|------|--------|
| `crates/biome_turtle_semantic/src/semantic_model/model.rs` | Add `Display` impl |
| `crates/biome_service/src/file_handlers/turtle.rs` | Change `format!("{model:#?}")` to `model.to_string()` |
