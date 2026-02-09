# Implementation Plan: Turtle Remaining Work Items 1-5

## Date: 2026-02-09

## Context

The Turtle language support in Biome has a working parser, formatter (23 node formatters), and analyzer (10 nursery lint rules with snapshot tests). However, the lint rules lack auto-fixes, the formatter has no Turtle-specific configuration options, there are no suppression comment tests, and some additional rules and formatter options from the gap analysis remain unimplemented. This plan covers all 5 items from `kb/tasks/turtle-remaining-work.md`.

---

## Item 1: Auto-Fixes for 6 Existing Lint Rules

Each rule needs: `fix_kind: FixKind::Safe` in `declare_lint_rule!` + an `fn action()` method returning `Option<TurtleRuleAction>`.

### 1a. `useShorthandRdfType` — Replace `rdf:type` with `a`
**File:** `crates/biome_turtle_analyze/src/lint/nursery/use_shorthand_rdf_type.rs`
- Add `fix_kind: FixKind::Safe` to macro
- Add `fn action()`: get the first token from the verb node, create `SyntaxToken::new_detached(TurtleSyntaxKind::A_KW, "a", [], [])`, use `mutation.replace_token_transfer_trivia()`
- Add imports: `FixKind`, `BatchMutationExt`, `TurtleRuleAction`, `TurtleSyntaxToken`

### 1b. `useConsistentQuotes` — Replace `'string'` with `"string"`
**File:** `crates/biome_turtle_analyze/src/lint/nursery/use_consistent_quotes.rs`
- Add `fix_kind: FixKind::Safe`
- Store the token in state (change `InconsistentQuote` to hold the `SyntaxToken`)
- `fn action()`: take token text, replace outer `'` with `"`, create new detached token, `replace_token_transfer_trivia()`

### 1c. `noLiteralTrimIssues` — Trim whitespace from literal value
**File:** `crates/biome_turtle_analyze/src/lint/nursery/no_literal_trim_issues.rs`
- Add `fix_kind: FixKind::Unsafe` (changing literal value is potentially semantic)
- Store token in `TrimIssue` state
- `fn action()`: extract inner text between quotes, trim it, reconstruct quoted string, create new token, `replace_token_transfer_trivia()`

### 1d. `useConsistentDirectiveStyle` — Convert SPARQL-style to Turtle-style
**File:** `crates/biome_turtle_analyze/src/lint/nursery/use_consistent_directive_style.rs`
- Add `fix_kind: FixKind::Safe`
- Store the directive node reference in `InconsistentStyle`
- `fn action()`: This is a node-level replacement. For `TurtleSparqlPrefixDeclaration` → need to construct the equivalent `@prefix ns: <iri> .` text. Since node-level construction is complex, use `mutation.replace_element()` or rebuild tokens. Simplest approach: get the namespace and IRI tokens from the SPARQL node, build a string like `@prefix {ns} {iri} .`, and create a new parsed subtree. **However**, since Biome's mutation API works at the token/node level, a simpler approach is to replace individual tokens: replace `PREFIX` → `@prefix`, add trailing `.` — this requires careful handling. **Decision**: Mark as `FixKind::Unsafe` and skip for initial implementation (complex node reconstruction). Come back to this if time permits.

### 1e. `noDuplicatePrefixDeclaration` — Remove duplicate declaration
**File:** `crates/biome_turtle_analyze/src/lint/nursery/no_duplicate_prefix_declaration.rs`
- Add `fix_kind: FixKind::Safe`
- Store the directive node in `DuplicatePrefix` state
- `fn action()`: use `mutation.remove_node()` on the duplicate directive's syntax node (need to look up the node from the range, or store a reference). The `remove_node` approach requires having the actual node. Refactor `run()` to store a reference to the syntax node or use `mutation.remove_statement()`.

### 1f. `noUnusedPrefix` — Remove unused prefix declaration
**File:** `crates/biome_turtle_analyze/src/lint/nursery/no_unused_prefix.rs`
- Add `fix_kind: FixKind::Safe`
- Store the directive syntax node in `UnusedPrefix` state
- `fn action()`: use `mutation.remove_node()` on the unused prefix's syntax node

### Test updates
- Update existing `invalid.ttl` test fixtures if needed (current ones should trigger auto-fixes)
- Re-run `cargo test -p biome_turtle_analyze` and accept updated snapshots (they'll now include code fix sections)

---

## Item 2: Turtle-Specific Formatter Configuration Options

### 2a. Add `quoteStyle` to configuration
**File:** `crates/biome_configuration/src/turtle.rs`
- Add `use biome_formatter::QuoteStyle;`
- Add field to `TurtleFormatterConfiguration`:
  ```rust
  #[bpaf(long("turtle-formatter-quote-style"), argument("double|single"))]
  pub quote_style: Option<QuoteStyle>,
  ```
- Update `default_turtle_formatter` test

### 2b. Wire through settings
**File:** `crates/biome_service/src/file_handlers/turtle.rs`
- Add `pub quote_style: Option<QuoteStyle>` to `TurtleFormatterSettings`
- Update `From<TurtleFormatterConfiguration>` to pass through `quote_style`
- In `resolve_format_options()`: add `.with_quote_style(language.quote_style.unwrap_or_default())`

### 2c. Add to format options
**File:** `crates/biome_turtle_formatter/src/context.rs`
- Add `use biome_formatter::QuoteStyle;`
- Add `quote_style: QuoteStyle` field to `TurtleFormatOptions`
- Add `with_quote_style()`, `set_quote_style()`, `quote_style()` methods
- Update `new()` to initialize `quote_style: QuoteStyle::default()`
- Update `Display` impl

### 2d. Use in formatter nodes
- The `TurtleString` formatter node should check `f.options().quote_style()` and rewrite quotes accordingly. This requires modifying the string literal formatter in `crates/biome_turtle_formatter/src/turtle/` (find the node that formats `TurtleString`).

---

## Item 3: Suppression Comment Tests

### 3a. Add suppression test runner to spec_tests.rs
**File:** `crates/biome_turtle_analyze/tests/spec_tests.rs`
- Add a second `gen_tests!` macro:
  ```rust
  tests_macros::gen_tests! {"tests/suppression/**/*.ttl", crate::run_suppression_test, "module"}
  ```
- Add `run_suppression_test()` function (modeled on CSS pattern): parse file, run analyzer with rule filter, collect only suppression actions, snapshot result

### 3b. Create suppression test fixtures
**Directory:** `crates/biome_turtle_analyze/tests/suppression/nursery/`

Create test files for 2-3 representative rules:
- `noUnusedPrefix/suppress.ttl`:
  ```turtle
  # biome-ignore lint/nursery/noUnusedPrefix: intentionally unused
  @prefix unused: <http://example.org/unused/> .
  @prefix foaf: <http://xmlns.com/foaf/0.1/> .
  <http://example.org/alice> foaf:name "Alice" .
  ```
- `useShorthandRdfType/suppress.ttl`:
  ```turtle
  @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
  @prefix foaf: <http://xmlns.com/foaf/0.1/> .
  # biome-ignore lint/nursery/useShorthandRdfType: using full form
  <http://example.org/alice> rdf:type foaf:Person .
  ```
- `noDuplicatePrefixDeclaration/suppress.ttl`

### 3c. Run and accept snapshots
- `cargo test -p biome_turtle_analyze` → `cargo insta accept`

---

## Item 4: Additional P3 Lint Rules (5 new rules)

All go in `crates/biome_turtle_analyze/src/lint/nursery/`. Each needs:
- Rule file in the nursery directory
- Test fixtures (`valid.ttl` + `invalid.ttl`) in `crates/biome_turtle_analyze/tests/specs/nursery/{ruleName}/`

### 4a. `useSortedPrefixes` (style, medium complexity)
- Query: `Ast<TurtleRoot>`
- Logic: collect all prefix declarations, check if they're alphabetically sorted by namespace
- State: `Vec<UnsortedPrefix>` with range of first out-of-order declaration
- No auto-fix initially

### 4b. `useGroupedSubjects` (style, high complexity)
- Query: `Ast<TurtleRoot>`
- Logic: collect all `TurtleTriples` nodes, extract subjects, detect when same subject appears in separate triple blocks
- State: `Vec<UngroupedSubject>` with subject text and ranges
- No auto-fix (complex restructuring)

### 4c. `usePrefixedNames` (style, high complexity)
- Query: `Ast<TurtleRoot>` (needs prefix context)
- Logic: collect declared prefix IRIs, scan all `TURTLE_IRIREF_LITERAL` tokens, check if any IRI starts with a declared prefix's expansion
- State: `Vec<ExpandableIri>` with range, suggested prefixed name
- No auto-fix initially

### 4d. `noMalformedDatatype` (correctness, high complexity)
- Query: `Ast<TurtleRdfLiteral>`
- Logic: when `^^xsd:integer`, `^^xsd:boolean`, `^^xsd:decimal`, `^^xsd:date` etc. are used, validate the literal value matches the expected format
- State: `MalformedDatatype` with range, expected format, found value
- XSD types to validate: `xsd:integer` (regex `^[+-]?\d+$`), `xsd:boolean` (`true|false|0|1`), `xsd:decimal` (decimal format), `xsd:date` (ISO date)

### 4e. `noUndefinedSubjectReference` (correctness, high complexity)
- Query: `Ast<TurtleRoot>`
- Logic: collect all subjects (first element of triples), collect all objects that look like they reference subjects (prefixed names / IRIs used as objects), flag objects that are never defined as subjects
- **Decision**: Skip this rule — it's extremely complex and has high false-positive risk (objects frequently reference external resources). Replace with a simpler rule or defer.

---

## Item 5: Advanced Formatter Options

### 5a. `directiveStyle` option
**Type:** enum `DirectiveStyle { Turtle, Sparql }` — default `Turtle`

**Files to modify:**
- `crates/biome_configuration/src/turtle.rs` — add field
- `crates/biome_turtle_formatter/src/context.rs` — add to `TurtleFormatOptions`
- `crates/biome_service/src/file_handlers/turtle.rs` — wire through
- Formatter node for directives — apply style conversion during formatting

### 5b. `firstPredicateInNewLine` option
**Type:** `bool`, default `true`

Controls whether the first predicate in a subject block goes on a new line:
```turtle
# firstPredicateInNewLine: true (default)
ex:alice
    foaf:name "Alice" ;
    foaf:age 30 .

# firstPredicateInNewLine: false
ex:alice foaf:name "Alice" ;
    foaf:age 30 .
```

**Files to modify:** Same config/options chain + predicate-object-list formatter node

### 5c. `alignPredicates` option
**Type:** `bool`, default `false`

Aligns predicates vertically within a subject block. This is complex to implement in Biome's IR-based formatter (requires computing max predicate width). **Decision**: Defer — mark as future work.

### 5d. `prefixOrder` / `predicateOrder`
Custom ordering arrays. **Decision**: Defer — these require serialization of string arrays in configuration, which adds significant complexity.

---

## Implementation Order

1. **Auto-fixes** (1a-1c first — simple token replacements; then 1e-1f — node removals; defer 1d)
2. **`quoteStyle` config** (2a-2d)
3. **Suppression tests** (3a-3c)
4. **New lint rules** (4a, 4d, 4b, 4c — skip 4e)
5. **`directiveStyle` + `firstPredicateInNewLine`** formatter options (5a, 5b — defer 5c, 5d)

---

## Key Files

| File | Purpose |
|------|---------|
| `crates/biome_turtle_analyze/src/lint/nursery/*.rs` | All lint rule implementations |
| `crates/biome_turtle_analyze/src/lib.rs` | `TurtleRuleAction` type alias, `analyze()` |
| `crates/biome_turtle_analyze/tests/spec_tests.rs` | Analyzer test runner |
| `crates/biome_turtle_analyze/tests/specs/nursery/*/` | Test fixtures |
| `crates/biome_configuration/src/turtle.rs` | Config structs |
| `crates/biome_service/src/file_handlers/turtle.rs` | Settings + ServiceLanguage impl |
| `crates/biome_turtle_formatter/src/context.rs` | TurtleFormatOptions |
| `crates/biome_turtle_syntax/src/generated/kind.rs` | Token kinds (A_KW, STRING_LITERAL_QUOTE, etc.) |
| `crates/biome_turtle_syntax/src/generated/nodes.rs` | AST node types and methods |

---

## Verification

1. `cargo build -p biome_turtle_analyze -p biome_turtle_formatter` — no compile errors
2. `cargo test -p biome_turtle_analyze` — all tests pass (with `cargo insta accept` for new/updated snapshots)
3. `cargo test -p biome_turtle_formatter` — all tests pass
4. Verify auto-fix snapshots show correct code transformations in `.snap` files
5. Verify suppression test snapshots show diagnostics are suppressed
6. `cargo clippy -p biome_turtle_analyze -p biome_turtle_formatter` — no warnings
