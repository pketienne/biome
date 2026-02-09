# Plan: Add Literal Short Notation Edge-Case Tests

**Date:** 2026-02-09
**Status:** To implement
**Effort:** Quick win (< 1 hour)

## Context

The literal short notation test fixture covers basic cases for all 4 types. Missing edge cases: positive sign prefix, zero values, negative exponents for doubles, full IRI datatype form, and decimal edge cases.

## Test Cases to Add

```turtle
# Positive sign prefix
ex:p ex:count "+42"^^xsd:integer .
ex:q ex:value "+3.14"^^xsd:decimal .
ex:r ex:value "+1.5E2"^^xsd:double .

# Zero values
ex:s ex:count "0"^^xsd:integer .
ex:t ex:value "0.0"^^xsd:decimal .

# Double edge cases
ex:u ex:value "1.0E-5"^^xsd:double .
ex:v ex:value "42E0"^^xsd:double .
ex:w ex:value "1.E3"^^xsd:double .

# Full IRI datatype form (should also be shortened)
ex:x ex:active "true"^^<http://www.w3.org/2001/XMLSchema#boolean> .
ex:y ex:count "99"^^<http://www.w3.org/2001/XMLSchema#integer> .

# Invalid: empty string for numeric types
ex:z1 ex:count ""^^xsd:integer .
ex:z2 ex:value ""^^xsd:double .
```

## File to Modify

| File | Change |
|------|--------|
| `crates/biome_turtle_formatter/tests/specs/turtle/literal_short_notation.ttl` | Add edge cases |
