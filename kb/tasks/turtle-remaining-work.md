# Turtle Remaining Work

## Date: 2026-02-09 (updated)

## Current State

- **Parser**: Complete W3C Turtle syntax parsing with error recovery
- **Formatter**: All 23 node formatters implemented with real formatting logic
- **Linter**: 14 nursery rules implemented (10 original P0-P2 + 4 new P3 rules)
- **Auto-Fixes**: 6 rules have working auto-fixes (including useConsistentDirectiveStyle)
- **Suppression Tests**: 8 suppression comment tests covering all key rules
- **Snapshot Tests**: 14 formatter + 38 analyzer snapshot tests (specs + suppression + assist)
- **Configuration**: `quoteStyle`, `firstPredicateInNewLine`, and `directiveStyle` formatter options wired through config/settings/options
- **String Normalization**: Unicode escape normalization, triple-quote demotion
- **Assists**: Infrastructure + `sortPrefixDeclarations` assist with auto-fix
- **Service Integration**: File handler, settings resolution, LSP capabilities wired up

---

## Completed Work

### 1. Auto-Fixes for Existing Lint Rules -- DONE (6 of 6)

| Rule | Fix | Status |
|------|-----|--------|
| `useShorthandRdfType` | Replace `rdf:type` with `a` | Done (Safe) |
| `useConsistentQuotes` | Replace `'string'` with `"string"` | Done (Safe) |
| `noLiteralTrimIssues` | Trim whitespace from literal value | Done (Unsafe) |
| `noDuplicatePrefixDeclaration` | Remove duplicate declaration | Done (Safe) |
| `noUnusedPrefix` | Remove unused `@prefix` declaration | Done (Safe) |
| `useConsistentDirectiveStyle` | Convert `PREFIX`/`BASE` to `@prefix`/`@base` | Done (Safe) |

### 2. Turtle-Specific Formatter Configuration Options -- DONE

| Option | Type | Default | Status |
|--------|------|---------|--------|
| `quoteStyle` | `"double"` \| `"single"` | `"double"` | Done (config + settings + options + formatting logic) |
| `firstPredicateInNewLine` | `bool` | `true` | Done (config + settings + options + formatting logic) |
| `directiveStyle` | `"turtle"` \| `"sparql"` | `"turtle"` | Done (config + settings + options + formatting logic) |

### 3. Suppression Comment Tests -- DONE (8 tests)

- `run_suppression_test()` wired up in `spec_tests.rs`
- 8 test fixtures: `noUnusedPrefix`, `useShorthandRdfType`, `noDuplicatePrefixDeclaration`, `noUndefinedPrefix`, `useConsistentQuotes`, `useConsistentDirectiveStyle`, `noLiteralTrimIssues`, `noDuplicateTriple`
- All snapshots accepted and passing

### 4. Additional P3 Lint Rules -- DONE (4 of 5)

| Rule | Category | Status |
|------|----------|--------|
| `useSortedPrefixes` | style | Done |
| `useGroupedSubjects` | style | Done |
| `usePrefixedNames` | style | Done |
| `noMalformedDatatype` | correctness | Done |
| `noUndefinedSubjectReference` | correctness | **Skipped** (high false-positive risk) |

### 5. `quoteStyle` Formatter Logic -- DONE

- String literal rewriting in `FormatTurtleString` node formatter
- Checks `f.options().quote_style()` and swaps quotes when possible
- Handles single/double and long (triple) quote variants
- Preserves original quotes when swapping would require escaping

### 6. `directiveStyle` Formatter Option -- DONE

- `TurtleDirectiveStyle` enum (`Turtle` / `Sparql`) in configuration
- All 4 directive formatter nodes handle bidirectional conversion
- Converts `@prefix`/`@base` ↔ `PREFIX`/`BASE` during formatting

### 7. `useConsistentDirectiveStyle` Auto-Fix -- DONE

- Converts SPARQL-style to Turtle-style using `SyntaxNode::new_detached`
- Handles proper trivia (whitespace) in constructed tokens

### 8. Escape and Literal Normalization -- DONE (2 of 3)

| Feature | Status |
|---------|--------|
| Unicode escape normalization | Done (`\u0041` → `A`, `\U00000041` → `A`) |
| Quote demotion | Done (triple-quoted strings without newlines → single-quoted) |
| Literal short notation | **Deferred** (complex, requires separate `TurtleRdfLiteral` formatting) |

### 9. Assist Actions -- DONE (1 assist)

- Assist infrastructure: `assist.rs`, `assist/source.rs`, registry integration
- `sortPrefixDeclarations`: Sorts `@prefix` declarations alphabetically with auto-fix
- Test fixtures (valid + invalid) with snapshot tests

---

## Remaining Work

### 1. `alignPredicates` Formatter Option

**Priority: Low** -- Vertically align predicates within a subject block. Complex in Biome's IR-based formatter (requires computing max predicate width across siblings).

---

### 2. `prefixOrder` / `predicateOrder` Formatter Options

**Priority: Low** -- Custom ordering arrays for prefix declarations and predicates. Requires string array serialization in configuration.

---

### 3. Literal Short Notation

**Priority: Low** -- Convert `"true"^^xsd:boolean` → `true`, `"42"^^xsd:integer` → `42`. Requires a formatter node for `TurtleRdfLiteral` that understands XSD datatypes.

---

### 4. Additional Assists

**Priority: Low** -- More code actions beyond `sortPrefixDeclarations`.

| Assist | Description |
|--------|-------------|
| Remove unused prefixes | Bulk remove all unused prefix declarations |
| Convert IRI to prefixed name | Replace `<http://xmlns.com/foaf/0.1/name>` with `foaf:name` |
| Convert `rdf:type` to `a` | Replace all `rdf:type` usages with shorthand |

---

### 5. Documentation

**Priority: Medium** -- Required for website generation.

- Ensure each lint rule has complete rustdoc with examples (used by `gen-analyzer` codegen)
- Verify rule metadata (`version`, `language`, `recommended`) is correct
- Add Turtle formatter section to website docs (PR to biomejs/website)

---

## Suggested Order of Implementation

1. Documentation polish for all 14 rules
2. `alignPredicates` formatter option
3. Literal short notation
4. Additional assists
5. `prefixOrder` / `predicateOrder`
