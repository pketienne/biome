# Turtle Remaining Work — Implementation Plan

## Date: 2026-02-09

---

## Items to Implement (All Complete)

### Item 1: Literal Short Notation in Formatter ✅

**Priority: Medium** | **Complexity: Medium** | **Files: 1** | **Status: DONE**

Convert typed literals to their short notation when the value matches the expected format:
- `"true"^^xsd:boolean` → `true`
- `"false"^^xsd:boolean` → `false`
- `"42"^^xsd:integer` → `42`
- `"3.14"^^xsd:decimal` → `3.14`

**Implementation:**
- **File:** `crates/biome_turtle_formatter/src/turtle/value/rdf_literal.rs`
- In `fmt_fields()`, when a `TurtleDatatypeAnnotation` is present:
  1. Extract the datatype IRI text (e.g., `xsd:boolean`, `<http://www.w3.org/2001/XMLSchema#boolean>`)
  2. Extract the string literal value
  3. If datatype matches a known XSD type and the value is valid for that type, emit just the bare value (omitting quotes and `^^datatype`)
  4. For `xsd:boolean`: value must be exactly `"true"` or `"false"`
  5. For `xsd:integer`: value must match `^[+-]?\d+$`
  6. For `xsd:decimal`: value must match `^[+-]?\d*\.\d+$`
- Use `format_replaced` + `syntax_token_cow_slice` to replace the entire rdf_literal output
- **Note:** This changes semantics slightly (short form is equivalent per RDF spec), so it should only apply when explicitly enabled or as a safe normalization

**Test:** Add `crates/biome_turtle_formatter/tests/specs/turtle/literal_short_notation.ttl`

---

### Item 2: Additional Assist Actions (3 assists) ✅

**Priority: Medium** | **Complexity: Low-Medium** | **Files: 3 new + 2 modified** | **Status: DONE**

#### 2a. `removeUnusedPrefixes`
- **File:** `crates/biome_turtle_analyze/src/assist/source/remove_unused_prefixes.rs`
- Query: `Ast<TurtleRoot>` — collect all declared prefixes and used prefixes
- State: list of unused prefix directive nodes
- Action: `mutation.remove_node()` for each unused prefix
- Reuses logic from the `noUnusedPrefix` lint rule but as a bulk action

#### 2b. `convertIriToPrefixedName`
- **File:** `crates/biome_turtle_analyze/src/assist/source/convert_iri_to_prefixed_name.rs`
- Query: `Ast<TurtleRoot>` — collect prefix declarations, scan for full IRIs matching prefix expansions
- State: list of (IRI range, suggested prefixed name)
- Action: replace IRI token with prefixed name token

#### 2c. `convertRdfTypeToShorthand`
- **File:** `crates/biome_turtle_analyze/src/assist/source/convert_rdf_type_to_shorthand.rs`
- Query: `Ast<TurtleRoot>` — find all `rdf:type` or `<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>` verb nodes
- State: list of verb ranges to replace
- Action: replace verb token with `a` keyword

**Test fixtures:** Create `valid.ttl` + `invalid.ttl` for each in `tests/specs/source/`

**Registration:** Update `crates/biome_turtle_analyze/src/assist/source.rs` and `crates/biome_diagnostics_categories/src/categories.rs`

---

### Item 3: Documentation Polish for All 14 Lint Rules ✅

**Priority: Medium** | **Complexity: Low** | **Files: 14 modified** | **Status: DONE**

Ensure every rule has:
1. Clear one-line summary
2. Explanation paragraph with rationale
3. `## Examples` section with `### Invalid` and `### Valid` subsections
4. Each example in a `turtle,expect_diagnostic` or `turtle` fenced code block
5. Multiple invalid examples for rules that catch different patterns
6. At least one valid example showing the correct alternative

Rules to audit:
- All 14 rules in `crates/biome_turtle_analyze/src/lint/nursery/`

---

### Items Deferred (Not Implementing)

| Item | Reason |
|------|--------|
| `alignPredicates` | Not feasible — Biome's single-pass formatter has no sibling width measurement; `align()` only supports fixed-width alignment |
| `prefixOrder` / `predicateOrder` | Low priority — requires string array serialization in configuration schema |

---

## Implementation Order

1. **Documentation polish** (Item 3) — Low risk, improves all rules
2. **Literal short notation** (Item 1) — Self-contained formatter change
3. **Additional assists** (Item 2) — Builds on existing assist infrastructure

---

## Verification — All Passing ✅

1. `cargo build -p biome_turtle_analyze -p biome_turtle_formatter` — no compile errors ✅
2. `cargo test -p biome_turtle_analyze` — 44 tests passed ✅
3. `cargo test -p biome_turtle_formatter` — 15 tests passed ✅
4. All snapshots accepted via `cargo insta accept` ✅
