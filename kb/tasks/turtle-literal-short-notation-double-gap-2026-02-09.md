# Literal Short Notation: Missing `xsd:double` Support

**Date:** 2026-02-09
**Status:** Not implemented
**Location:** `crates/biome_turtle_formatter/src/turtle/value/rdf_literal.rs`

## Overview

The formatter's `try_short_notation()` function converts explicitly-typed RDF literals to their bare shorthand form. Three of four XSD types are implemented; `xsd:double` is missing.

## Current State

| Type | Long Form | Short Form | Status |
|------|-----------|------------|--------|
| `xsd:boolean` | `"true"^^xsd:boolean` | `true` | Done |
| `xsd:integer` | `"42"^^xsd:integer` | `42` | Done |
| `xsd:decimal` | `"3.14"^^xsd:decimal` | `3.14` | Done |
| `xsd:double` | `"4.2E9"^^xsd:double` | `4.2E9` | **Missing** |

## Gap Details

- The **parser/lexer** already fully support reading bare doubles (`TURTLE_DOUBLE_LITERAL` token)
- The **formatter** does not convert `"4.2E9"^^xsd:double` to `4.2E9`
- Both turtlefmt and prttl (comparable tools) implement double short notation
- The `try_short_notation()` function returns `None` for all datatypes beyond boolean, integer, and decimal

## Turtle Spec Reference

The W3C Turtle spec defines the `DOUBLE` production ([section 6.5](https://www.w3.org/TR/turtle/#grammar-production-DOUBLE)):

```
[21] DOUBLE   ::= [+-]? ([0-9]+ '.' [0-9]* EXPONENT | '.' [0-9]+ EXPONENT | [0-9]+ EXPONENT)
[154s] EXPONENT ::= [eE] [+-]? [0-9]+
```

## Implementation

Add an `xsd:double` branch to `try_short_notation()` in `rdf_literal.rs`:
- Recognize both `xsd:double` (prefixed) and `<http://www.w3.org/2001/XMLSchema#double>` (full IRI)
- Validate that the string value matches the `DOUBLE` production (contains `e` or `E` exponent)
- If valid, return the bare value; if invalid, return `None` (preserve long form)
- Add test cases to `crates/biome_turtle_formatter/tests/specs/turtle/literal_short_notation.ttl`

## Files to Modify

| File | Change |
|------|--------|
| `crates/biome_turtle_formatter/src/turtle/value/rdf_literal.rs` | Add `xsd:double` branch in `try_short_notation()` |
| `crates/biome_turtle_formatter/tests/specs/turtle/literal_short_notation.ttl` | Add double test cases |
