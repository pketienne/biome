# Implementation Plan: Remaining Feasible P3 Items

## Context

The Turtle implementation is at ~78% gap coverage (100% P0-P2). Nine P3 items remain: 3 are architecturally infeasible (alignment), 1 is skipped (noUndefinedSubjectReference — false-positive risk), and 5 are feasible. This plan covers those 5 feasible items, implemented as 4 new assist actions + 1 rule option.

**Design decision**: Ordering/expansion/merging features are implemented as **assist actions** (not formatter options) because they are one-time AST transformations, avoid complex config serialization, and align with the existing assist pattern (`sortPrefixDeclarations`, `removeUnusedPrefixes`, etc.).

---

## Item 1: `ignoredPrefixes` Option on `noUnusedPrefix`

**Complexity: Low** — Add rule option so users can whitelist prefixes that should not trigger the unused-prefix warning.

### Files to modify

1. **`crates/biome_rule_options/src/no_unused_prefix.rs`** (new) — Options struct with `ignored_prefixes: Option<Box<[Box<str>]>>` field. Follow the `NoLabelWithoutControlOptions` pattern from `crates/biome_rule_options/src/no_label_without_control.rs`.

2. **`crates/biome_rule_options/src/lib.rs`** — Add `pub mod no_unused_prefix;`

3. **`crates/biome_turtle_analyze/Cargo.toml`** — Add `biome_rule_options = { workspace = true }` dependency

4. **`crates/biome_turtle_analyze/src/lint/nursery/no_unused_prefix.rs`** — Change `type Options = ()` to `type Options = NoUnusedPrefixOptions`. In `run()`, read `ctx.options().ignored_prefixes` and skip matching namespaces in the filter.

### Config example
```json
{ "linter": { "rules": { "nursery": {
  "noUnusedPrefix": { "level": "warn", "options": { "ignoredPrefixes": ["owl:", "skos:"] } }
}}}}
```

### Test
Update `tests/specs/nursery/noUnusedPrefix/valid.ttl` or add a new test that uses options.

---

## Item 2: `sortPredicates` Assist Action

**Complexity: Medium** — Sort predicate-object pairs alphabetically within each subject block.

### File: `crates/biome_turtle_analyze/src/assist/source/sort_predicates.rs` (new)

Follow the `sortPrefixDeclarations` pattern exactly — same `mutation.replace_element()` approach but on predicate-object pairs instead of prefix directives.

### Algorithm
1. Query `Ast<TurtleRoot>`, iterate all `TurtleTriples`
2. For each triple with 2+ predicate-object pairs:
   - Extract verb text for each pair via `pair.verb()?.syntax().text_trimmed()`
   - Treat `a` as `rdf:type` for sorting
   - Check if already sorted (case-insensitive)
3. If any triple is unsorted, return state with range
4. In `action()`: collect pairs with sort keys, sort, replace each original position with sorted pair using `mutation.replace_element()`

### Test fixture (`tests/specs/source/sortPredicates/invalid.ttl`)
```turtle
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
<http://example.org/alice>
    foaf:knows <http://example.org/bob> ;
    foaf:age 30 ;
    foaf:name "Alice" .
```

### Register
- Add category `"assist/source/sortPredicates"` in `categories.rs`
- The `source.rs` module auto-discovers via `declare_group_from_fs!`

---

## Item 3: `sortTriples` Assist Action

**Complexity: Medium** — Sort triple statement blocks by subject text.

### File: `crates/biome_turtle_analyze/src/assist/source/sort_triples.rs` (new)

Same pattern as `sortPrefixDeclarations` but on `TurtleTriples` nodes.

### Algorithm
1. Query `Ast<TurtleRoot>`, collect all `TurtleTriples` statements (skip directives)
2. Extract subject text for each via `triples.subject()?.syntax().text_trimmed()`
3. Check if already sorted (case-insensitive)
4. If unsorted, in `action()` sort and replace each position using `mutation.replace_element()`

### Test fixture (`tests/specs/source/sortTriples/invalid.ttl`)
```turtle
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
<http://example.org/carol> foaf:name "Carol" .
<http://example.org/alice> foaf:name "Alice" .
<http://example.org/bob> foaf:name "Bob" .
```

### Register
- Add category `"assist/source/sortTriples"` in `categories.rs`

---

## Item 4: `expandTriples` Assist Action

**Complexity: High** — Expand compound triples into one-triple-per-line form (diff-optimized).

### File: `crates/biome_turtle_analyze/src/assist/source/expand_triples.rs` (new)

### Algorithm
1. Query `Ast<TurtleRoot>`, find triples with multiple predicates OR multiple objects
2. For each expandable triple, build replacement text: for every (subject, verb, object) combination, emit `subject verb object .` on its own line
3. Use `mutation.replace_element()` to replace the original triple node with a new parsed subtree

**Key challenge**: Constructing new nodes. Since we need to produce N separate `TurtleTriples` from 1, the simplest approach is:
- Build the replacement text as a string
- Parse it with `parse_turtle()`
- Extract the resulting statement nodes
- Replace the original node

However, Biome's mutation API doesn't directly support replacing one node with multiple sibling nodes. Alternative approach: **replace the entire statement list** — collect all statements, expand the ones that need expanding, rebuild the full statement list.

**Simpler alternative**: Since `mutation.replace_element()` only does 1:1 replacement, we can:
- For triples with multiple predicates but single objects each: keep as-is (`;` form is fine for diffs)
- Only expand object lists: replace `ex:s ex:p ex:o1, ex:o2 .` with multiple triples
- Actually, the real diff-optimized form needs complete expansion

**Revised approach**: Build the expanded text for the entire document (all triples expanded), parse it, and replace the root. This is the most reliable approach given the mutation API constraints.

### Test fixture (`tests/specs/source/expandTriples/invalid.ttl`)
```turtle
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
<http://example.org/alice>
    foaf:name "Alice" ;
    foaf:knows <http://example.org/bob>,
        <http://example.org/carol> .
```

Expected output:
```turtle
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
<http://example.org/alice> foaf:name "Alice" .
<http://example.org/alice> foaf:knows <http://example.org/bob> .
<http://example.org/alice> foaf:knows <http://example.org/carol> .
```

### Register
- Add category `"assist/source/expandTriples"` in `categories.rs`

---

## Item 5: `mergeTriples` Assist Action

**Complexity: High** — Inverse of expand: merge triples sharing subject+predicate into object lists.

### File: `crates/biome_turtle_analyze/src/assist/source/merge_triples.rs` (new)

### Algorithm
1. Query `Ast<TurtleRoot>`, collect all `TurtleTriples` with single predicate-object pair
2. Group by (subject_text, verb_text)
3. If any group has 2+ triples, these are mergeable
4. Build merged replacement: `subject verb obj1, obj2, obj3 .`
5. Also merge same-subject triples with different predicates into `;` form

**Same mutation challenge as expandTriples**: Need to replace N nodes with 1. Approach: replace first triple in each group with the merged form, remove the rest with `mutation.remove_node()`.

### Test fixture (`tests/specs/source/mergeTriples/invalid.ttl`)
```turtle
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
<http://example.org/alice> foaf:name "Alice" .
<http://example.org/alice> foaf:knows <http://example.org/bob> .
<http://example.org/alice> foaf:knows <http://example.org/carol> .
```

### Register
- Add category `"assist/source/mergeTriples"` in `categories.rs`

---

## Implementation Order

1. **`ignoredPrefixes` option** (Item 1) — Low complexity, standalone
2. **`sortPredicates` assist** (Item 2) — Directly follows `sortPrefixDeclarations` pattern
3. **`sortTriples` assist** (Item 3) — Same pattern as Item 2
4. **`mergeTriples` assist** (Item 5) — Before expand, since merge is more common
5. **`expandTriples` assist** (Item 4) — Most complex, last

---

## Key Files

| File | Purpose |
|------|---------|
| `crates/biome_rule_options/src/no_unused_prefix.rs` | New options struct |
| `crates/biome_rule_options/src/lib.rs` | Register options module |
| `crates/biome_turtle_analyze/Cargo.toml` | Add rule_options dep |
| `crates/biome_turtle_analyze/src/lint/nursery/no_unused_prefix.rs` | Wire options |
| `crates/biome_turtle_analyze/src/assist/source/sort_predicates.rs` | New assist |
| `crates/biome_turtle_analyze/src/assist/source/sort_triples.rs` | New assist |
| `crates/biome_turtle_analyze/src/assist/source/expand_triples.rs` | New assist |
| `crates/biome_turtle_analyze/src/assist/source/merge_triples.rs` | New assist |
| `crates/biome_diagnostics_categories/src/categories.rs` | Register 4 new categories |
| `crates/biome_turtle_analyze/src/assist/source/sort_prefix_declarations.rs` | Reference pattern |
| `crates/biome_rule_options/src/no_label_without_control.rs` | Reference for options with arrays |

---

## Verification

1. `cargo build -p biome_turtle_analyze` — no compile errors
2. `cargo test -p biome_turtle_analyze` — all tests pass
3. `cargo insta accept` for new snapshots
4. Verify assist snapshots show correct transformations
5. Verify `noUnusedPrefix` with `ignoredPrefixes` option doesn't flag whitelisted prefixes
