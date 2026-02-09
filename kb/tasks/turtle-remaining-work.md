# Turtle Remaining Work

## Date: 2026-02-09 (updated)

## Current State

- **Parser**: Complete W3C Turtle syntax parsing with error recovery
- **Formatter**: All 23 node formatters implemented with real formatting logic
- **Linter**: 14 nursery rules implemented (10 original P0-P2 + 4 new P3 rules)
- **Auto-Fixes**: 5 rules have working auto-fixes
- **Suppression Tests**: 3 suppression comment tests
- **Snapshot Tests**: 13 formatter + 31 analyzer snapshot tests (specs + suppression)
- **Configuration**: `quoteStyle` and `firstPredicateInNewLine` formatter options wired through config/settings/options
- **Service Integration**: File handler, settings resolution, LSP capabilities wired up

---

## Completed Work

### 1. Auto-Fixes for Existing Lint Rules -- DONE (5 of 6)

| Rule | Fix | Status |
|------|-----|--------|
| `useShorthandRdfType` | Replace `rdf:type` with `a` | Done (Safe) |
| `useConsistentQuotes` | Replace `'string'` with `"string"` | Done (Safe) |
| `noLiteralTrimIssues` | Trim whitespace from literal value | Done (Unsafe) |
| `noDuplicatePrefixDeclaration` | Remove duplicate declaration | Done (Safe) |
| `noUnusedPrefix` | Remove unused `@prefix` declaration | Done (Safe) |
| `useConsistentDirectiveStyle` | Convert `PREFIX`/`BASE` to `@prefix`/`@base` | **Deferred** (complex node reconstruction) |

### 2. Turtle-Specific Formatter Configuration Options -- DONE

| Option | Type | Default | Status |
|--------|------|---------|--------|
| `quoteStyle` | `"double"` \| `"single"` | `"double"` | Done (config + settings + options) |
| `firstPredicateInNewLine` | `bool` | `true` | Done (config + settings + options + formatting logic) |

### 3. Suppression Comment Tests -- DONE

- `run_suppression_test()` wired up in `spec_tests.rs`
- 3 test fixtures: `noUnusedPrefix`, `useShorthandRdfType`, `noDuplicatePrefixDeclaration`
- All snapshots accepted and passing

### 4. Additional P3 Lint Rules -- DONE (4 of 5)

| Rule | Category | Status |
|------|----------|--------|
| `useSortedPrefixes` | style | Done |
| `useGroupedSubjects` | style | Done |
| `usePrefixedNames` | style | Done |
| `noMalformedDatatype` | correctness | Done |
| `noUndefinedSubjectReference` | correctness | **Skipped** (high false-positive risk) |

### 5. Advanced Formatter Options -- PARTIAL

| Option | Status |
|--------|--------|
| `firstPredicateInNewLine` | Done (implemented in triples.rs) |
| `directiveStyle` | **Deferred** |
| `alignPredicates` | **Deferred** |
| `prefixOrder` / `predicateOrder` | **Deferred** |

---

## Remaining Work

### 1. `useConsistentDirectiveStyle` Auto-Fix

**Priority: Medium** -- Requires complex node reconstruction (replacing SPARQL-style `PREFIX`/`BASE` with Turtle-style `@prefix`/`@base` involves multi-token replacement and adding trailing `.`).

---

### 2. `directiveStyle` Formatter Option

**Priority: Medium** -- Enum `DirectiveStyle { Turtle, Sparql }`, default `Turtle`. Would auto-convert directive syntax during formatting. Blocked on same complexity as the lint rule auto-fix.

**Files to modify:**
- `crates/biome_configuration/src/turtle.rs` -- add field
- `crates/biome_turtle_formatter/src/context.rs` -- add to `TurtleFormatOptions`
- `crates/biome_service/src/file_handlers/turtle.rs` -- wire through
- Formatter node for directives -- apply style conversion

---

### 3. `alignPredicates` Formatter Option

**Priority: Low** -- Vertically align predicates within a subject block. Complex in Biome's IR-based formatter (requires computing max predicate width across siblings).

---

### 4. `prefixOrder` / `predicateOrder` Formatter Options

**Priority: Low** -- Custom ordering arrays for prefix declarations and predicates. Requires string array serialization in configuration.

---

### 5. Escape and Literal Normalization

**Priority: Low** -- Polish features found in turtlefmt and prttl.

| Feature | Description |
|---------|-------------|
| Escape normalization | Minimize string/IRI escape sequences (e.g. `\u0041` -> `A`) |
| Literal short notation | Convert `"true"^^xsd:boolean` -> `true`, `"42"^^xsd:integer` -> `42` |
| Quote promotion/demotion | Use triple quotes only for multiline strings |

---

### 6. Assists

**Priority: Low** -- Code actions that aren't tied to diagnostics.

| Assist | Description |
|--------|-------------|
| Sort prefix declarations | Reorder `@prefix` lines alphabetically |
| Remove unused prefixes | Bulk remove all unused prefix declarations |
| Convert IRI to prefixed name | Replace `<http://xmlns.com/foaf/0.1/name>` with `foaf:name` |
| Convert `rdf:type` to `a` | Replace all `rdf:type` usages with shorthand |

---

### 7. Documentation

**Priority: Medium** -- Required for website generation.

- Ensure each lint rule has complete rustdoc with examples (used by `gen-analyzer` codegen)
- Verify rule metadata (`version`, `language`, `recommended`, `sources`) is correct
- Add Turtle formatter section to website docs (PR to biomejs/website)

---

### 8. `quoteStyle` Formatter Logic

**Priority: Medium** -- The `quoteStyle` option is wired through config/settings/options but the actual string literal rewriting logic in the formatter node is not yet implemented. The formatter needs to check `f.options().quote_style()` and rewrite quotes on `TurtleString` nodes accordingly.

---

### 9. Additional Suppression Tests

**Priority: Low** -- Currently 3 suppression tests cover representative rules. Could add coverage for remaining rules.

---

## Suggested Order of Implementation

1. `quoteStyle` formatter logic (option is wired, just needs the node-level implementation)
2. `useConsistentDirectiveStyle` auto-fix + `directiveStyle` formatter option (related complexity)
3. Documentation polish for all 14 rules
4. `alignPredicates` formatter option
5. Escape/literal normalization
6. Assists
7. `prefixOrder` / `predicateOrder`
8. Additional suppression tests
