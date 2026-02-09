# Plan: Implement `noUndefinedSubjectReference` Lint Rule (Option C)

## Context

Implement a new Turtle lint rule that detects when an IRI used as an object in a triple is never defined as a subject anywhere in the document. This is an advisory-only rule (Info severity, not recommended by default) with an `allowedPrefixes` option to suppress false positives from external vocabularies. Only [rdflint](https://github.com/imas/rdflint) implements this check among existing tools — most RDF tools skip it due to the Open World Assumption.

The semantic model is already built and provides `triples()`, `triples_for_subject()`, and `prefix_map()` — all the data needed.

---

## Step 1: Create options struct

**Create** `crates/biome_rule_options/src/no_undefined_subject_reference.rs`

Follow the `NoUnusedPrefixOptions` pattern at `crates/biome_rule_options/src/no_unused_prefix.rs`:

```rust
#[derive(Default, Clone, Debug, Deserialize, Deserializable, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoUndefinedSubjectReferenceOptions {
    /// Additional prefixes to allow (external vocabularies, e.g., `["ex:", "org:"]`).
    /// Common vocabulary prefixes (rdf:, rdfs:, owl:, xsd:, foaf:, dc:, etc.)
    /// are always allowed by default.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub allowed_prefixes: Option<Box<[Box<str>]>>,
}
```

**Modify** `crates/biome_rule_options/src/lib.rs` — add module and re-export.

---

## Step 2: Register diagnostic category

**Modify** `crates/biome_diagnostics_categories/src/categories.rs`

Add in lexicographic order (after `noUndefinedPrefix`):
```
"lint/nursery/noUndefinedSubjectReference": "https://biomejs.dev/linter/rules/no-undefined-subject-reference",
```

---

## Step 3: Implement the rule

**Create** `crates/biome_turtle_analyze/src/lint/nursery/no_undefined_subject_reference.rs`

Key design:
- `type Query = Semantic<TurtleRoot>`
- `type Options = NoUndefinedSubjectReferenceOptions`
- `recommended: false`, `severity: Severity::Information`
- No fix action (diagnostic only)

**Algorithm:**
1. Collect all defined subjects from `model.triples()` into a `HashSet<&str>`
2. Build an allowed-prefixes set: built-in defaults + user-configured `allowedPrefixes`
3. Iterate all triples, for each object:
   - Skip literals (starts with `"`, or is a bare number/boolean)
   - Skip blank nodes (starts with `_:` or `[`)
   - Skip full IRIs in angle brackets (starts with `<`) — these are typically external resources
   - For prefixed-name objects (contains `:`): extract the namespace prefix
   - Skip if prefix is in the allowed set
   - Skip if object text is already a defined subject
   - Otherwise flag it

**Built-in allowed prefixes** (always allowed, hardcoded):
```rust
const BUILTIN_ALLOWED: &[&str] = &[
    "rdf:", "rdfs:", "owl:", "xsd:",
    "dc:", "dcterms:", "skos:", "foaf:",
    "schema:", "sh:", "prov:", "dcat:",
];
```

**Diagnostic message:**
- Primary: `"'{object}' is used as an object but never defined as a subject in this document."`
- Note: `"If this resource is defined externally, add its prefix to the 'allowedPrefixes' option."`

---

## Step 4: Register the rule

**Modify** `crates/biome_turtle_analyze/src/options.rs` — add type alias:
```rust
pub type NoUndefinedSubjectReference =
    <lint::nursery::no_undefined_subject_reference::NoUndefinedSubjectReference as biome_analyze::Rule>::Options;
```

The rule file in `nursery/` is auto-discovered by the `declare_group_from_fs!` macro — no changes needed to `nursery.rs`.

---

## Step 5: Create test fixtures

**Create** `crates/biome_turtle_analyze/tests/specs/nursery/noUndefinedSubjectReference/valid.ttl`:
```turtle
@prefix ex: <http://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
ex:alice foaf:name "Alice" .
ex:alice ex:knows ex:bob .
ex:bob foaf:name "Bob" .
```
All objects are either literals, built-in-prefix references (foaf:), or defined subjects (ex:bob).

**Create** `crates/biome_turtle_analyze/tests/specs/nursery/noUndefinedSubjectReference/invalid.ttl`:
```turtle
@prefix ex: <http://example.org/> .
ex:alice ex:knows ex:bob .
ex:alice ex:knows ex:carol .
ex:bob ex:name "Bob" .
```
`ex:carol` is used as an object but never appears as a subject. `ex:knows` and `ex:name` are predicates (not checked). `ex:bob` is defined as a subject (not flagged).

---

## Files to Create

| File | Purpose |
|------|---------|
| `crates/biome_rule_options/src/no_undefined_subject_reference.rs` | Options struct |
| `crates/biome_turtle_analyze/src/lint/nursery/no_undefined_subject_reference.rs` | Rule implementation |
| `crates/biome_turtle_analyze/tests/specs/nursery/noUndefinedSubjectReference/valid.ttl` | Valid test fixture |
| `crates/biome_turtle_analyze/tests/specs/nursery/noUndefinedSubjectReference/invalid.ttl` | Invalid test fixture |

## Files to Modify

| File | Change |
|------|---------|
| `crates/biome_rule_options/src/lib.rs` | Add `no_undefined_subject_reference` module |
| `crates/biome_diagnostics_categories/src/categories.rs` | Add diagnostic category |
| `crates/biome_turtle_analyze/src/options.rs` | Add options type alias |

---

## Verification

1. `cargo build -p biome_turtle_analyze` — compiles
2. `cargo test -p biome_turtle_analyze` — all existing 52 spec tests + 3 unit tests pass, plus 2 new spec tests for this rule
3. Review snapshots: `valid.ttl` produces no diagnostics, `invalid.ttl` flags `ex:carol`
4. Verify `ex:bob` is NOT flagged (it's defined as a subject)
5. Verify `foaf:name` is NOT flagged (built-in allowed prefix)
6. Verify `"Bob"` is NOT flagged (literal)
