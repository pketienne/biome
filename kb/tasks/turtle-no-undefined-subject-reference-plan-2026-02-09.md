# Plan: `noUndefinedSubjectReference` Lint Rule

## Date: 2026-02-09
## Prerequisite: Semantic model (Step 1–4 minimum)
## Status: NEEDS REVIEW — High false-positive risk

---

## Overview

Detect when a triple references a subject (as object or predicate target) that is never defined as a subject of any triple in the current document. This was previously skipped as a P3 item due to false-positive risk.

**Example flagged case:**
```turtle
@prefix ex: <http://example.org/> .
ex:alice ex:knows ex:bob .
# ex:bob is referenced but never defined as a subject in this document
```

---

## Why This Needs Careful Review

### False-Positive Scenarios

1. **External vocabulary references are valid**
   ```turtle
   @prefix foaf: <http://xmlns.com/foaf/0.1/> .
   ex:alice foaf:name "Alice" .
   ```
   `foaf:name` is defined externally — flagging it would be wrong.

2. **Cross-file references are valid**
   ```turtle
   # file: people.ttl
   ex:alice ex:knows ex:bob .

   # file: bob.ttl (separate file)
   ex:bob foaf:name "Bob" .
   ```
   Biome analyzes single files. `ex:bob` is defined elsewhere.

3. **Blank nodes are self-defining**
   ```turtle
   ex:alice ex:address [
       ex:street "123 Main St" ;
       ex:city "Anytown"
   ] .
   ```
   The blank node `[]` is both defined and used inline.

4. **Well-known vocabulary IRIs are always valid**
   - `rdf:type`, `rdfs:label`, `owl:Class`, `xsd:integer`, etc.
   - Any predicate from a standard vocabulary shouldn't be flagged.

5. **Objects that are literals are never "undefined"**
   ```turtle
   ex:alice ex:age 30 .
   ```
   `30` is a literal, not a subject reference.

6. **Collections contain references**
   ```turtle
   ex:alice ex:knows ( ex:bob ex:carol ) .
   ```
   `ex:bob` and `ex:carol` in collections are valid external references.

7. **OWL/RDFS patterns reference external classes**
   ```turtle
   ex:Person a owl:Class ;
       rdfs:subClassOf ex:Agent .
   ```
   `ex:Agent` may be defined externally.

---

## Design Options

### Option A: Only flag IRI objects that share a prefix with a defined subject

**Logic:** If the document defines `ex:alice` and `ex:bob` as subjects, and `ex:carol` appears only as an object, flag `ex:carol` — because the `ex:` prefix is "local" to this document.

**Pros:** Reduces false positives for external vocabularies (foaf:, rdf:, etc.)
**Cons:** Still false-positive for cross-file references within the same namespace.

### Option B: Only flag when subject and object share the same prefix AND no other file could define it

**Logic:** Impossible without cross-file analysis. Skip this option.

### Option C: Advisory-only rule at Info severity with `allowedPrefixes` option

**Logic:** Flag potential undefined references at Info severity (not error/warning). Provide an `allowedPrefixes` option to whitelist external vocabulary prefixes.

```jsonc
{
  "linter": { "rules": { "nursery": {
    "noUndefinedSubjectReference": {
      "level": "info",
      "options": {
        "allowedPrefixes": ["rdf:", "rdfs:", "owl:", "xsd:", "foaf:", "dc:", "dcterms:", "skos:"]
      }
    }
  }}}
}
```

**Pros:** User controls false-positive suppression. Info severity means it's a suggestion, not an error.
**Cons:** Requires non-trivial configuration for useful results.

### Option D: Only flag subjects referenced in `rdf:type` / `a` position that aren't defined

**Logic:** If `ex:alice a ex:Person .` but `ex:Person` is never defined as a subject, flag it — because types are more likely to be locally defined.

**Pros:** Very narrow, fewer false positives.
**Cons:** Too narrow to be broadly useful.

### Option E: Don't implement — use `useGroupedSubjects` as a softer alternative

**Logic:** The existing `useGroupedSubjects` rule already flags scattered subject definitions. Combined with the semantic model's triple index, this provides similar value without false-positive risk.

**Pros:** No new false-positive surface area.
**Cons:** Doesn't catch truly undefined references.

---

## Recommended Approach: Option C

If implemented, use **Option C** — advisory Info-level rule with `allowedPrefixes`:

### Implementation

**Requires:** Semantic model with `triples_by_subject` index.

**Algorithm:**
1. Collect all subjects defined in the document: `model.triples_by_subject().keys()`
2. Collect all IRI/prefixed-name objects across all triples
3. Filter out:
   - Literal objects (strings, numbers, booleans)
   - Blank nodes (always self-defining)
   - Objects whose prefix is in `allowedPrefixes`
   - Objects that ARE defined as subjects
4. Flag remaining objects as "potentially undefined"

**Rule configuration:**
```rust
declare_lint_rule! {
    pub NoUndefinedSubjectReference {
        version: "next",
        name: "noUndefinedSubjectReference",
        language: "turtle",
        recommended: false,       // NOT recommended by default
        severity: Severity::Information,
    }
}
```

**Options struct:**
```rust
pub struct NoUndefinedSubjectReferenceOptions {
    /// Prefixes to ignore (external vocabularies).
    /// Default includes: rdf:, rdfs:, owl:, xsd:
    pub allowed_prefixes: Option<Box<[Box<str>]>>,
}
```

**Built-in defaults** (always allowed, even without configuration):
```rust
const BUILTIN_ALLOWED: &[&str] = &[
    "rdf:", "rdfs:", "owl:", "xsd:",
    "dc:", "dcterms:", "skos:", "foaf:",
    "schema:", "sh:", "prov:", "dcat:",
];
```

### Files to create/modify

| File | Action |
|------|--------|
| `crates/biome_rule_options/src/no_undefined_subject_reference.rs` | New — options struct |
| `crates/biome_rule_options/src/lib.rs` | Add module |
| `crates/biome_turtle_analyze/src/lint/nursery/no_undefined_subject_reference.rs` | New — rule implementation |
| `crates/biome_diagnostics_categories/src/categories.rs` | Add category |
| `crates/biome_turtle_analyze/tests/specs/nursery/noUndefinedSubjectReference/` | Test fixtures |

### Test Fixtures

**valid.ttl** (no diagnostics):
```turtle
@prefix ex: <http://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
ex:alice foaf:name "Alice" .
ex:alice ex:knows ex:bob .
ex:bob foaf:name "Bob" .
```

**invalid.ttl** (flags `ex:carol` as undefined):
```turtle
@prefix ex: <http://example.org/> .
ex:alice ex:knows ex:bob .
ex:alice ex:knows ex:carol .
ex:bob ex:name "Bob" .
```

---

## Decision Points for Review

1. **Should we implement this at all?** The false-positive risk is real. Option E (don't implement) is valid.

2. **If yes, what severity?** Info (suggestion) vs Warning?

3. **What default allowed prefixes?** The built-in list above covers common vocabularies. Should it be longer?

4. **Should it require the semantic model?** If so, it's gated behind Step 5 of the semantic model plan. If not, it duplicates triple extraction (which we're trying to eliminate).

5. **Should predicates also be checked?** Currently only objects are checked. Predicates are almost always from external vocabularies, so checking them would create many false positives.

6. **Should `a` / `rdf:type` objects (classes) be treated differently?** Classes referenced via `a ex:Person` are more likely to need local definition than arbitrary object references.

---

## Dependencies

```
Semantic model (Steps 1-5)
    └── noUndefinedSubjectReference rule
            └── Uses: model.triples_by_subject(), model.prefix_map()
```

This rule should NOT be implemented until:
1. The semantic model is built and working
2. The approach above has been reviewed and a decision made on the design options
3. Test cases have been agreed upon to validate false-positive behavior
