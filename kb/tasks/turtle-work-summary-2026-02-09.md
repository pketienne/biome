# Turtle Language Support — Work Summary

## Date: 2026-02-09

## Overview

Full W3C Turtle 1.1 language support has been implemented in a Biome fork on the `turtle` branch. The implementation spans parser, formatter, linter (analyzer), assist actions, configuration, and service integration across 10+ crates.

---

## Inventory

### Parser (`biome_turtle_parser`)

- Full W3C Turtle 1.1 grammar with error recovery
- Directives: `@prefix`, `@base`, `PREFIX`, `BASE`
- Triples: subject-predicate-object, predicate-object lists (`;`), object lists (`,`)
- Blank node property lists `[ ... ]`, collections `( ... )`
- All term types: IRIs, prefixed names, blank nodes, RDF/numeric/boolean literals, language tags, datatype annotations
- 4 string literal variants: single, double, long single, long double
- Special `a` keyword for `rdf:type`
- File extensions: `.ttl`, `.turtle`, `.nt`
- Language IDs: `turtle`, `ntriples`

### Formatter (`biome_turtle_formatter`)

**37 node formatters** organized in 6 groups:

| Group | Count | Formatters |
|-------|-------|------------|
| Auxiliary | 14 | root, triples, subject, verb, object, predicate_object_list, predicate_object_pair, iri, prefixed_name, string, blank_node, blank_node_property_list, collection, datatype_annotation |
| Declarations | 4 | prefix_declaration, base_declaration, sparql_prefix_declaration, sparql_base_declaration |
| Lists | 4 | statement_list, predicate_object_pair_list, object_list, collection_object_list |
| Values | 3 | rdf_literal, numeric_literal, boolean_literal |
| Any (dispatch) | 6 | statement, directive, subject, verb, object, iri_value |
| Bogus | 2 | bogus, bogus_statement |

**7 formatter options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `indent_style` | `IndentStyle` | (global) | Tab or space indentation |
| `indent_width` | `IndentWidth` | (global) | Number of spaces per indent |
| `line_ending` | `LineEnding` | (global) | LF, CRLF, or CR |
| `line_width` | `LineWidth` | (global) | Maximum line width |
| `quoteStyle` | `"double"` / `"single"` | `"double"` | Preferred string quote style |
| `firstPredicateInNewLine` | `bool` | `true` | Whether first predicate goes on a new line |
| `directiveStyle` | `"turtle"` / `"sparql"` | `"turtle"` | Directive keyword style |

**String normalization features:**
- Unicode escape normalization (`\u0041` → `A`, `\U00000041` → `A`)
- Triple-quote demotion (triple-quoted strings without newlines → single-quoted)
- Quote style conversion with safety checks (no swap when it would require escaping)

### Linter / Analyzer (`biome_turtle_analyze`)

**14 nursery lint rules:**

| # | Rule | Severity | Recommended | Auto-fix | Description |
|---|------|----------|-------------|----------|-------------|
| 1 | `noUndefinedPrefix` | Error | Yes | — | Disallows undeclared prefix usage |
| 2 | `noInvalidIri` | Error | Yes | — | Disallows invalid IRI characters |
| 3 | `noInvalidLanguageTag` | Warning | Yes | — | Validates BCP47 language tags |
| 4 | `noDuplicateTriple` | Warning | Yes | — | Detects duplicate triples |
| 5 | `noDuplicatePrefixDeclaration` | Error | Yes | Safe | Removes duplicate prefix declarations |
| 6 | `noUnusedPrefix` | Warning | Yes | Safe | Removes unused prefix declarations |
| 7 | `noLiteralTrimIssues` | Warning | No | Unsafe | Trims whitespace from literals |
| 8 | `noMalformedDatatype` | Error | Yes | — | Validates XSD datatype formats |
| 9 | `useShorthandRdfType` | Info | Yes | Safe | Replaces `rdf:type` with `a` |
| 10 | `useConsistentQuotes` | Info | No | Safe | Enforces double/single quote consistency |
| 11 | `useConsistentDirectiveStyle` | Warning | Yes | Safe | Enforces `@prefix`/`PREFIX` consistency |
| 12 | `useSortedPrefixes` | Info | No | — | Enforces alphabetical prefix order |
| 13 | `useGroupedSubjects` | Info | No | — | Suggests grouping same-subject triples |
| 14 | `usePrefixedNames` | Info | No | — | Suggests prefixed names over full IRIs |

- 9 recommended, 6 with auto-fix (5 Safe, 1 Unsafe)
- 4 Error, 4 Warning, 6 Information severity

**4 assist actions:**

| Assist | Category | Fix | Description |
|--------|----------|-----|-------------|
| `sortPrefixDeclarations` | source | Safe | Sorts `@prefix` declarations alphabetically |
| `removeUnusedPrefixes` | source | Safe | Bulk-removes all unused prefix declarations |
| `convertIriToPrefixedName` | source | Safe | Converts full IRIs to prefixed names when matching prefix is declared |
| `convertRdfTypeToShorthand` | source | Safe | Replaces all `rdf:type` / full IRI form with `a` keyword |

### Configuration (`biome_configuration`)

```jsonc
{
  "turtle": {
    "formatter": {
      "enabled": true,
      "indentStyle": "tab",
      "indentWidth": 2,
      "lineEnding": "lf",
      "lineWidth": 80,
      "quoteStyle": "double",
      "firstPredicateInNewLine": true,
      "directiveStyle": "turtle"
    },
    "linter": {
      "enabled": true
    },
    "assist": {
      "enabled": false
    }
  }
}
```

### Service Integration (`biome_service`)

**Implemented capabilities:**
- `parse()` — Parse Turtle documents
- `format()` / `format_range()` / `format_on_type()` — Format documents
- `lint()` — Run lint rules
- `code_actions()` — Generate code actions (lint fixes + assists)
- `fix_all()` — Apply all safe fixes
- `debug_syntax_tree()` / `debug_formatter_ir()` — Debug output

**Not yet implemented:**
- `rename()`, `pull_diagnostics_and_actions()`, `format_embedded()`, `debug_control_flow()`, `debug_semantic_model()`

### Tests

| Category | Count | Location |
|----------|-------|----------|
| Lint rule specs (valid + invalid) | 14 rules × 2 files | `biome_turtle_analyze/tests/specs/nursery/` |
| Assist specs | 4 assists × 2 files | `biome_turtle_analyze/tests/specs/source/` |
| Suppression tests | 8 rules | `biome_turtle_analyze/tests/suppression/nursery/` |
| Formatter specs | 16 test files | `biome_turtle_formatter/tests/specs/turtle/` |
| Parser unit tests | 2 | `biome_turtle_parser/src/` |
| **Total passing tests** | **62+** | All crates |

---

## Completed Work (Chronological)

### Session 1: Foundation
- Complete Turtle parser with W3C grammar support
- 23 formatter node implementations with real formatting logic
- 10 initial lint rules (P0-P2 priority)
- Service integration (file handler, settings, LSP capabilities)
- Snapshot tests for formatter and analyzer

### Session 2: Auto-fixes, Config, P3 Rules
- Auto-fixes for 5 lint rules (useShorthandRdfType, useConsistentQuotes, noLiteralTrimIssues, noDuplicatePrefixDeclaration, noUnusedPrefix)
- `quoteStyle` and `firstPredicateInNewLine` formatter config options (config → settings → options pipeline)
- 3 initial suppression comment tests
- 4 new P3 lint rules (useSortedPrefixes, useGroupedSubjects, usePrefixedNames, noMalformedDatatype)

### Session 3: Formatter Options, Normalization, Assists
- `useConsistentDirectiveStyle` auto-fix using `SyntaxNode::new_detached`
- `directiveStyle` formatter option with bidirectional `@prefix`↔`PREFIX` conversion
- `quoteStyle` formatter logic (actual string literal rewriting in FormatTurtleString)
- Unicode escape normalization and triple-quote demotion
- Assist infrastructure + `sortPrefixDeclarations` assist
- 5 additional suppression tests (8 total)

### Session 4: Literal Short Notation, Additional Assists, Documentation Polish
- Literal short notation in formatter: `"true"^^xsd:boolean` → `true`, `"42"^^xsd:integer` → `42`, `"3.14"^^xsd:decimal` → `3.14` (with validation)
- 3 additional assist actions: `removeUnusedPrefixes`, `convertIriToPrefixedName`, `convertRdfTypeToShorthand`
- Documentation polish: added second examples to `useShorthandRdfType`, `noMalformedDatatype`, `useConsistentDirectiveStyle`
- All 62+ tests passing across analyzer (44 tests) and formatter (15 tests) crates

---

## Remaining Work (Deferred — Not Feasible with Current Architecture)

### 1. `alignPredicates` Formatter Option
**Status: Deferred** — Vertically align predicates within a subject block. Not feasible with Biome's single-pass IR-based formatter architecture (no sibling width measurement). Biome's `align()` only supports fixed-width alignment.

### 2. `prefixOrder` / `predicateOrder` Formatter Options
**Status: Deferred** — Custom ordering arrays. Requires string array serialization in configuration, adding significant complexity for a niche feature.
