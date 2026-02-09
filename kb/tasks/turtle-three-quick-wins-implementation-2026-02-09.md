# Implementation: Three Quick Wins (debug_semantic_model, edge-case tests, literal short notation)

**Date:** 2026-02-09
**Status:** Done
**Commit:** (pending)

## Summary

Implemented all three quick-win plans in a single pass. All tests pass.

## Plan #1: Enhance `debug_semantic_model` Output

Added `Display` impl for `SemanticModel` in `model.rs` — outputs base URI, prefix declarations (with unused/duplicate markers), triples, duplicate triples, and expandable IRIs in human-readable format. Updated `turtle.rs` to use `model.to_string()` instead of `format!("{model:#?}")`.

### Files Modified

| File | Change |
|------|--------|
| `crates/biome_turtle_semantic/src/semantic_model/model.rs` | Added `std::fmt` import and `Display` impl |
| `crates/biome_service/src/file_handlers/turtle.rs` | Changed `format!("{model:#?}")` to `model.to_string()` |

## Plan #2: Semantic Model Edge-Case Tests (7 new, 18 total)

| Test | What it covers |
|------|----------------|
| `blank_node_as_subject` | `_:b1` as subject |
| `blank_node_property_list` | `[...]` syntax — confirmed model captures as single triple, not decomposed |
| `semicolon_notation_multiple_predicates` | `;` produces 2 triples with same subject |
| `comma_notation_object_list` | `,` produces 2 triples with same subject and predicate |
| `triples_for_subject_index` | Lookup returns correct indices, unknown returns empty |
| `empty_document` | No triples, no prefixes, no base URI |
| `prefix_references_tracking` | Verifies prefix references are collected |

### Files Modified

| File | Change |
|------|--------|
| `crates/biome_turtle_semantic/src/lib.rs` | Added 7 unit tests |

## Plan #3: Literal Short Notation Edge Cases

Added to the formatter test fixture and verified correct output:

| Category | Cases | Result |
|----------|-------|--------|
| Positive sign prefix | `+42`, `+3.14`, `+1.5E2` | Correctly shortened |
| Zero values | `0`, `0.0` | Correctly shortened |
| Double edge cases | `1.0E-5` (negative exponent), `42E0` (zero exponent) | Correctly shortened |
| Full IRI datatype form | `"true"^^<http://...#boolean>`, `"99"^^<http://...#integer>` | Correctly shortened |
| Invalid: empty strings | `""^^xsd:integer`, `""^^xsd:double` | Correctly preserved |
| Invalid: parser limitation | `"1.E3"^^xsd:double` | Correctly preserved |

### Files Modified

| File | Change |
|------|--------|
| `crates/biome_turtle_formatter/tests/specs/turtle/literal_short_notation.ttl` | Added edge cases |
| `crates/biome_turtle_formatter/tests/specs/turtle/literal_short_notation.ttl.snap` | Updated snapshot |
| `crates/biome_turtle_formatter/src/turtle/value/rdf_literal.rs` | Fixed `is_valid_double()` to require digit after dot |

## Bug Fix: `is_valid_double()` Parser Ambiguity

Discovered during implementation: the Turtle grammar allows `[0-9]+ '.' [0-9]* EXPONENT` (zero digits after dot), but the Turtle lexer greedily matches `1.` as a DECIMAL token, making forms like `1.E3` unparseable in short notation. Fixed `is_valid_double()` to require at least one digit after the decimal point when a dot is present.

## Test Results

- **Semantic model**: 18 passed (7 new)
- **Formatter**: 15 passed, 1 ignored
- **Analyzer**: 54 passed, 1 ignored
