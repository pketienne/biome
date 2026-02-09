# Plan: Add Stress Tests for Turtle Language Support

**Date:** 2026-02-09
**Status:** To implement
**Effort:** Quick win (< 1 hour)

## Context

The Turtle semantic model has 18 unit tests and the analyzer has 54 spec tests, but none test with large inputs. Stress tests verify correctness and stability with:
- Large numbers of triples
- Deeply nested blank node property lists
- Large collections
- Many prefix declarations
- Large object lists (comma notation)

These are unit tests in the semantic model crate, not full Criterion benchmarks (which would be a separate effort).

## Tests to Add

Add to `crates/biome_turtle_semantic/src/lib.rs`:

### 1. `stress_many_triples`
Generate a document with 1000 triples. Verify the model collects all of them and `triples_for_subject` works correctly.

### 2. `stress_many_prefixes`
Generate a document with 100 prefix declarations. Verify all are tracked, unused detection works.

### 3. `stress_deep_nested_blank_nodes`
Generate nested blank node property lists 10 levels deep. Verify parsing and model construction complete without stack overflow.

### 4. `stress_large_object_list`
Generate a single subject with 500 objects via comma notation. Verify correct triple count.

### 5. `stress_large_collection`
Generate a collection with 200 elements. Verify model handles it.

### 6. `stress_duplicate_detection`
Generate 100 unique triples + 50 exact duplicates. Verify duplicate count.

## File to Modify

| File | Change |
|------|--------|
| `crates/biome_turtle_semantic/src/lib.rs` | Add 6 stress tests |
