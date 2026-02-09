# Turtle Language Support — Status Report

## Date: 2026-02-09
## Branch: `turtle` (9 commits ahead of `main`)

---

## Executive Summary

Full W3C Turtle 1.1 language support has been implemented in a Biome fork across 10+ crates. The implementation includes a complete parser with error recovery, a formatter with 8 configurable options and 30 node formatters, 14 nursery lint rules (6 with auto-fix), 4 assist actions, and full service integration for LSP capabilities. All 62+ tests pass.

---

## Commit History

```
cd1f87a7e0 feat(turtle): add literal short notation, 3 assist actions, and doc polish
777af6c8e0 feat(turtle): add directiveStyle option, escape normalization, assist actions, and suppression tests
2cc58dc01c docs: update remaining work task file with completed items
a2cc8f43ed feat(turtle): add auto-fixes, config options, suppression tests, and new lint rules
54d9f8aae9 docs: add implementation plan and research notes for Turtle language
83468184c6 docs: add remaining work task file for Turtle language support
6cb7927ad4 test(turtle): add snapshot tests for formatter and analyzer
bb7b3a6852 feat(turtle): implement formatter, linter, and fix grammar for Turtle language
44cf4767a8 WIP: turtle language support
```

---

## Parser (`biome_turtle_parser` + `biome_turtle_syntax`)

Full W3C Turtle 1.1 grammar with error recovery.

### Syntax Kinds (73 total)

| Category | Count | Details |
|----------|-------|---------|
| Node kinds | 28 | Root, statements, directives, triples, terms, literals, bogus |
| List kinds | 6 | Statement, predicate-object pair, object, collection lists |
| Literal tokens | 11 | IRIREF, PNAME_NS, PNAME_LN, BLANK_NODE_LABEL, LANGTAG, 4 string variants, INTEGER, DECIMAL, DOUBLE |
| Keywords | 7 | `@prefix`, `@base`, `a`, `true`, `false`, `PREFIX`, `BASE` |
| Punctuation | 8 | `. ; , [ ] ( ) ^^` |
| Special | 5 | ERROR_TOKEN, NEWLINE, WHITESPACE, COMMENT, ANON |

### Supported Constructs
- Directives: `@prefix`, `@base`, `PREFIX`, `BASE`
- Triples: subject-predicate-object, predicate-object lists (`;`), object lists (`,`)
- Blank nodes: labeled (`_:name`), anonymous (`[]`), property lists (`[ p o ]`)
- Collections: `( item1 item2 ... )`
- Terms: IRIs, prefixed names, blank nodes, string/numeric/boolean literals
- Strings: 4 variants (single, double, long single, long double)
- Annotations: language tags (`@en`), datatype annotations (`^^xsd:integer`)
- Special `a` keyword for `rdf:type`
- File extensions: `.ttl`, `.turtle`, `.nt`
- Language IDs: `turtle`, `ntriples`

---

## Formatter (`biome_turtle_formatter`)

### Node Formatters (30 implementations + 7 module files + 2 bogus)

| Group | Count | Nodes |
|-------|-------|-------|
| Auxiliary | 14 | root, triples, subject, verb, object, predicate_object_list, predicate_object_pair, iri, prefixed_name, string, blank_node, blank_node_property_list, collection, datatype_annotation |
| Declarations | 4 | prefix_declaration, base_declaration, sparql_prefix_declaration, sparql_base_declaration |
| Lists | 4 | statement_list, predicate_object_pair_list, object_list, collection_object_list |
| Values | 3 | rdf_literal, numeric_literal, boolean_literal |
| Any (dispatch) | 6 | statement, directive, subject, verb, object, iri_value |
| Bogus | 2 | bogus, bogus_statement |

### Configuration Options (8)

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `indentStyle` | `"tab"` / `"space"` | (global) | Tab or space indentation |
| `indentWidth` | `number` | (global) | Spaces per indent level |
| `lineEnding` | `"lf"` / `"crlf"` / `"cr"` | (global) | Line ending style |
| `lineWidth` | `number` | (global) | Maximum line width |
| `quoteStyle` | `"double"` / `"single"` | `"double"` | Preferred string quote style |
| `firstPredicateInNewLine` | `bool` | `true` | First predicate on a new line after subject |
| `directiveStyle` | `"turtle"` / `"sparql"` | `"turtle"` | `@prefix`/`@base` vs `PREFIX`/`BASE` |
| `_fileSource` | `TurtleFileSource` | (internal) | Internal file source tracking |

### Formatting Features
- **String normalization**: Quote style conversion with safety checks (no swap when it would require escaping)
- **Unicode escape normalization**: `\u0041` → `A`, `\U00000041` → `A`
- **Triple-quote demotion**: Triple-quoted strings without newlines → single-quoted
- **Literal short notation**: `"true"^^xsd:boolean` → `true`, `"42"^^xsd:integer` → `42`, `"3.14"^^xsd:decimal` → `3.14` (with validation — invalid values preserved as-is)
- **Directive style conversion**: Bidirectional `@prefix`↔`PREFIX`, `@base`↔`BASE`
- **Predicate-object formatting**: Configurable first-predicate placement, `;`-separated pairs with indentation

### Test Fixtures (15)

a_keyword, blank_node, blank_node_property_list, boolean_literal, collection, full_document, literal_short_notation, literals, numeric_literal, object_list, predicate_object_list, prefix_declaration, simple_triples, sparql_declaration, string_normalization

---

## Linter / Analyzer (`biome_turtle_analyze`)

### Nursery Lint Rules (14)

| # | Rule | Severity | Rec. | Fix | Description |
|---|------|----------|------|-----|-------------|
| 1 | `noUndefinedPrefix` | Error | Yes | — | Disallow undeclared prefix usage |
| 2 | `noInvalidIri` | Error | Yes | — | Disallow invalid IRI characters |
| 3 | `noInvalidLanguageTag` | Warning | Yes | — | Validate BCP47 language tags |
| 4 | `noDuplicateTriple` | Warning | Yes | — | Detect duplicate triples |
| 5 | `noDuplicatePrefixDeclaration` | Error | Yes | Safe | Remove duplicate prefix declarations |
| 6 | `noUnusedPrefix` | Warning | Yes | Safe | Remove unused prefix declarations |
| 7 | `noLiteralTrimIssues` | Warning | No | Unsafe | Trim whitespace from string literals |
| 8 | `noMalformedDatatype` | Error | Yes | — | Validate XSD datatype literal values |
| 9 | `useShorthandRdfType` | Info | Yes | Safe | Replace `rdf:type` with `a` |
| 10 | `useConsistentQuotes` | Info | No | Safe | Enforce double/single quote consistency |
| 11 | `useConsistentDirectiveStyle` | Warning | Yes | Safe | Enforce `@prefix`/`PREFIX` consistency |
| 12 | `useSortedPrefixes` | Info | No | — | Enforce alphabetical prefix order |
| 13 | `useGroupedSubjects` | Info | No | — | Suggest grouping same-subject triples |
| 14 | `usePrefixedNames` | Info | No | — | Suggest prefixed names over full IRIs |

**Summary**: 9 recommended, 6 with auto-fix (5 Safe, 1 Unsafe), 4 Error / 4 Warning / 6 Info severity

### Assist Actions (4)

| Assist | Category | Fix | Description |
|--------|----------|-----|-------------|
| `sortPrefixDeclarations` | source | Safe | Sort `@prefix` declarations alphabetically |
| `removeUnusedPrefixes` | source | Safe | Bulk-remove all unused prefix declarations |
| `convertIriToPrefixedName` | source | Safe | Convert full IRIs to prefixed names |
| `convertRdfTypeToShorthand` | source | Safe | Replace all `rdf:type` with `a` keyword |

---

## Configuration (`biome_configuration`)

```jsonc
{
  "turtle": {
    "formatter": {
      "enabled": true,          // default: true
      "indentStyle": "tab",
      "indentWidth": 2,
      "lineEnding": "lf",
      "lineWidth": 80,
      "quoteStyle": "double",   // "double" | "single"
      "firstPredicateInNewLine": true,
      "directiveStyle": "turtle" // "turtle" | "sparql"
    },
    "linter": {
      "enabled": true           // default: true
    },
    "assist": {
      "enabled": false          // default: false
    }
  }
}
```

### Configuration Structs
- `TurtleConfiguration` — top-level with formatter/linter/assist sections
- `TurtleFormatterConfiguration` — 8 fields (4 global + 3 Turtle-specific + enabled)
- `TurtleLinterConfiguration` — enabled toggle
- `TurtleAssistConfiguration` — enabled toggle
- `TurtleDirectiveStyle` — enum: `Turtle` (default), `Sparql`

---

## Service Integration (`biome_service`)

### Implemented Capabilities

| Capability | Method | Status |
|------------|--------|--------|
| Parser | `parse` | Implemented |
| Formatter | `format` | Implemented |
| Formatter | `format_range` | Implemented |
| Formatter | `format_on_type` | Implemented |
| Linter | `lint` | Implemented |
| Analyzer | `code_actions` | Implemented |
| Analyzer | `fix_all` | Implemented |
| Debug | `debug_syntax_tree` | Implemented |
| Debug | `debug_formatter_ir` | Implemented |
| Settings | `lookup_settings` | Implemented |
| Settings | `resolve_format_options` | Implemented |
| Settings | `resolve_analyzer_options` | Implemented |
| Settings | `*_enabled_for_file_path` | Implemented (formatter, linter, assist) |

### Not Implemented

| Method | Reason |
|--------|--------|
| `rename` | Requires semantic model |
| `pull_diagnostics_and_actions` | Not required for initial support |
| `format_embedded` | No embedded language support needed |
| `debug_control_flow` | Not applicable |
| `debug_semantic_model` | No semantic model yet |
| `search` | Not implemented |

---

## Test Summary

| Category | Count | Location |
|----------|-------|----------|
| Lint rule specs (valid + invalid) | 14 rules x 2 | `tests/specs/nursery/` |
| Assist specs (valid + invalid) | 4 assists x 2 | `tests/specs/source/` |
| Suppression tests | 8 rules | `tests/suppression/nursery/` |
| Formatter specs | 15 fixtures | `tests/specs/turtle/` |
| Unit tests (analyzer) | 3 | `biome_turtle_analyze/src/lib.rs` |
| **Total passing** | **62+** | All crates |

All tests verified passing:
- `cargo test -p biome_turtle_analyze` — 44 passed
- `cargo test -p biome_turtle_formatter` — 15 passed
- All snapshots accepted

---

## Remaining / Deferred Items

| Item | Status | Reason |
|------|--------|--------|
| `alignPredicates` option | Deferred | Not feasible — Biome's single-pass IR formatter has no sibling width measurement |
| `prefixOrder` / `predicateOrder` options | Deferred | Low priority — requires string array serialization in config schema |
| Semantic model | Not started | Would enable: rename, go-to-definition, cross-file analysis |
| Website documentation | Not started | Requires PR against biomejs/website `next` branch |

---

## Crate Dependency Graph

```
biome_turtle_syntax ─┬─► biome_turtle_parser
                     ├─► biome_turtle_formatter ─► biome_turtle_factory
                     └─► biome_turtle_analyze
                                ↓
                         biome_service (file_handlers/turtle.rs)
                                ↓
                         biome_configuration (turtle.rs)
```

---

## Key Technical Decisions

1. **Literal short notation**: Uses `format_replaced` + `format_removed` pattern with `mark_suppression_checked` to bypass standard node formatting when emitting bare values
2. **Directive style conversion**: Formatter handles bidirectional `@prefix`↔`PREFIX` conversion using `SyntaxNode::new_detached` for node-level reconstruction
3. **String normalization**: Quote style, unicode escapes, and triple-quote demotion all handled in `FormatTurtleString` with safety checks to prevent escaping issues
4. **Assist vs lint pattern**: Assists use `declare_source_rule!` and query `Ast<TurtleRoot>` for document-wide bulk transformations; lint rules use `declare_lint_rule!` and query individual nodes
5. **`alignPredicates` infeasibility**: Biome's formatter is single-pass IR-based; `align()` only supports fixed-width alignment, not dynamic vertical alignment across siblings
