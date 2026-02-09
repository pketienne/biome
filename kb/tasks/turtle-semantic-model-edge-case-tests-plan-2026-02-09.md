# Plan: Add Semantic Model Edge-Case Tests

**Date:** 2026-02-09
**Status:** To implement
**Effort:** Quick win (< 1 hour)

## Context

The semantic model has 11 unit tests covering core functionality. Missing edge cases: blank nodes as subjects, collections, nested blank node property lists, semicolon notation (multiple predicates), and triples_for_subject index.

## Tests to Add

1. **Blank nodes as subjects** — `_:b1 foaf:name "Bob" .` should appear in triples with subject `_:b1`
2. **Blank node property lists** — `[ foaf:name "Alice" ]` creates triples with a blank node subject
3. **Semicolon notation** — `ex:alice foaf:name "Alice" ; foaf:age "30" .` produces 2 triples with same subject
4. **Comma notation (object lists)** — `ex:alice foaf:knows ex:bob, ex:carol .` produces 2 triples
5. **`triples_for_subject` index** — Verify lookup returns correct indices
6. **Empty document** — No triples, no prefixes, no errors
7. **Prefix references tracking** — Verify `prefix_references()` returns all prefixed name usages

## File to Modify

| File | Change |
|------|--------|
| `crates/biome_turtle_semantic/src/lib.rs` | Add 7 unit tests |
