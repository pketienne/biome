# Implementing Turtle (.ttl) Linting, Formatting, and Validation in Biome

## Overview

Turtle (Terse RDF Triple Language) is a W3C standard syntax for RDF graphs. Unlike YAML, biome has **no existing Turtle support** — parser, syntax, and all downstream crates must be built from scratch.

The Turtle grammar is small (19 productions + 20 terminals in the W3C spec), making it one of the simpler languages to add. For comparison, GraphQL has a 578-line `.ungram` grammar; Turtle would need roughly 200-350 lines.

## W3C Turtle Grammar Summary

The full grammar from [W3C TR/turtle](https://www.w3.org/TR/turtle/#sec-grammar):

**Productions (19):**

| # | Rule | Definition |
|---|---|---|
| 1 | `turtleDoc` | `statement*` |
| 2 | `statement` | `directive \| triples '.'` |
| 3 | `directive` | `prefixID \| base \| sparqlPrefix \| sparqlBase` |
| 4 | `prefixID` | `'@prefix' PNAME_NS IRIREF '.'` |
| 5 | `base` | `'@base' IRIREF '.'` |
| 5s | `sparqlBase` | `"BASE" IRIREF` |
| 6s | `sparqlPrefix` | `"PREFIX" PNAME_NS IRIREF` |
| 6 | `triples` | `subject predicateObjectList \| blankNodePropertyList predicateObjectList?` |
| 7 | `predicateObjectList` | `verb objectList (';' (verb objectList)?)*` |
| 8 | `objectList` | `object (',' object)*` |
| 9 | `verb` | `predicate \| 'a'` |
| 10 | `subject` | `iri \| BlankNode \| collection` |
| 11 | `predicate` | `iri` |
| 12 | `object` | `iri \| BlankNode \| collection \| blankNodePropertyList \| literal` |
| 13 | `literal` | `RDFLiteral \| NumericLiteral \| BooleanLiteral` |
| 14 | `blankNodePropertyList` | `'[' predicateObjectList ']'` |
| 15 | `collection` | `'(' object* ')'` |
| 16 | `NumericLiteral` | `INTEGER \| DECIMAL \| DOUBLE` |
| 128s | `RDFLiteral` | `String (LANGTAG \| '^^' iri)?` |
| 133s | `BooleanLiteral` | `'true' \| 'false'` |
| 17 | `String` | 4 string literal variants (single/double, short/long) |
| 135s | `iri` | `IRIREF \| PrefixedName` |
| 136s | `PrefixedName` | `PNAME_LN \| PNAME_NS` |
| 137s | `BlankNode` | `BLANK_NODE_LABEL \| ANON` |

**Terminals (20):** `IRIREF`, `PNAME_NS`, `PNAME_LN`, `BLANK_NODE_LABEL`, `LANGTAG`, `INTEGER`, `DECIMAL`, `DOUBLE`, `EXPONENT`, `STRING_LITERAL_QUOTE`, `STRING_LITERAL_SINGLE_QUOTE`, `STRING_LITERAL_LONG_SINGLE_QUOTE`, `STRING_LITERAL_LONG_QUOTE`, `UCHAR`, `ECHAR`, `WS`, `ANON`, `PN_CHARS_BASE`, `PN_CHARS_U`, `PN_CHARS`, `PN_PREFIX`, `PN_LOCAL`, `PLX`, `PERCENT`, `HEX`, `PN_LOCAL_ESC`

**Keywords:** `@prefix`, `@base`, `a`, `true`, `false`, `BASE` (case-insensitive), `PREFIX` (case-insensitive)

**Punctuation:** `.`, `;`, `,`, `[`, `]`, `(`, `)`, `^^`, `<`, `>`, `:`, `_:`

## Crates Required

Following biome's per-language architecture (modeled after `biome_graphql_*`):

### 1. `biome_turtle_syntax` — syntax node types

```
crates/biome_turtle_syntax/
├── Cargo.toml          # deps: biome_rowan
└── src/
    ├── lib.rs
    ├── syntax_node.rs  # TurtleLanguage, TurtleSyntaxNode, etc.
    ├── file_source.rs  # TurtleFileSource (.ttl, .turtle, .nt, .nq, .trig)
    ├── generated.rs
    └── generated/      # auto-generated from codegen
        ├── kind.rs     # TurtleSyntaxKind enum
        ├── nodes.rs    # typed AST node wrappers
        ├── nodes_mut.rs
        └── macros.rs
```

### 2. `biome_turtle_factory` — AST node construction

```
crates/biome_turtle_factory/
├── Cargo.toml          # deps: biome_rowan, biome_turtle_syntax
└── src/
    ├── lib.rs
    ├── make.rs
    ├── generated.rs
    └── generated/      # auto-generated
```

### 3. `biome_turtle_parser` — lexer and parser

```
crates/biome_turtle_parser/
├── Cargo.toml          # deps: biome_parser, biome_turtle_syntax, biome_turtle_factory
└── src/
    ├── lib.rs          # parse() entry point
    ├── token_source.rs
    ├── lexer/
    │   ├── mod.rs      # Turtle lexer (IRI, prefixed names, strings, numbers)
    │   └── tests.rs
    └── parser/
        ├── mod.rs
        ├── directive.rs    # @prefix, @base, PREFIX, BASE
        ├── triples.rs      # subject-predicate-object patterns
        ├── literal.rs      # RDF literals, numeric, boolean
        ├── iri.rs          # IRIREF, prefixed names
        ├── blank_node.rs   # blank nodes, anonymous nodes, property lists
        ├── collection.rs   # RDF collections ( ... )
        └── tests.rs
```

Lexer considerations specific to Turtle:
- IRI parsing (`<...>`) with unicode escapes
- Prefixed name parsing (`prefix:localName`) — context-sensitive, colon is both separator and part of token
- 4 string literal types (single/double quote, short/long)
- Language tags (`@en`, `@en-US`) vs `@prefix`/`@base` keywords — both start with `@`
- Numeric literals with optional sign
- Comment handling (`#` to end of line)

### 4. `biome_turtle_formatter` — pretty-printer

```
crates/biome_turtle_formatter/
├── Cargo.toml          # deps: biome_formatter, biome_turtle_syntax, biome_rowan
└── src/
    ├── lib.rs          # format_node entry point
    ├── context.rs      # TurtleFormatOptions
    ├── comments.rs     # comment attachment
    ├── generated.rs    # auto-generated per-node formatting
    ├── turtle/         # per-node formatting rules
    │   ├── directive.rs
    │   ├── triples.rs
    │   ├── predicate_object_list.rs
    │   ├── object_list.rs
    │   ├── literal.rs
    │   └── blank_node_property_list.rs
    └── utils/
```

Formatting decisions specific to Turtle:
- Prefix declarations: aligned or not, sorted or not
- Subject-predicate-object layout: same line vs multi-line
- Predicate-object list indentation after `;`
- Object list wrapping after `,`
- Blank node property list `[ ... ]` inline vs expanded
- Collection `( ... )` wrapping
- IRI style: full IRI vs prefixed name (formatter should preserve, not convert)
- String quote normalization (e.g. always use `"` over `'`)

### 5. `biome_turtle_analyze` — linter and validator

```
crates/biome_turtle_analyze/
├── Cargo.toml          # deps: biome_analyze, biome_analyze_macros, biome_turtle_syntax, biome_suppression
└── src/
    ├── lib.rs          # analyze entry point + METADATA
    ├── registry.rs     # auto-generated rule registry
    ├── suppression_action.rs
    └── lint/
        ├── mod.rs
        ├── correctness.rs
        ├── correctness/    # validation rules
        ├── style.rs
        ├── style/          # formatting/convention rules
        ├── suspicious.rs
        ├── suspicious/     # likely-mistake rules
        └── nursery.rs
```

### 6. `biome_turtle_semantic` (optional) — semantic model

```
crates/biome_turtle_semantic/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── events.rs
    └── semantic_model/
        ├── mod.rs
        ├── prefix_map.rs   # tracks @prefix declarations → IRI mappings
        └── blank_node_scope.rs
```

A semantic model enables cross-statement analysis (undefined prefixes, unused prefixes, duplicate subjects). Without it, rules can only inspect individual syntax nodes.

## Codegen Files

### New files

| File | Purpose |
|---|---|
| `xtask/codegen/turtle.ungram` | Ungrammar definition (~200-350 lines) |
| `xtask/codegen/src/turtle_kinds_src.rs` | Syntax kind definitions |

### Modified files

| File | Change |
|---|---|
| `xtask/codegen/src/language_kind.rs` | Add `Turtle` variant to `LanguageKind` enum, `LANGUAGE_PREFIXES`, `ALL_LANGUAGE_KIND`, `FromStr`, `.kinds()`, `.load_grammar()` |
| `xtask/codegen/src/lib.rs` | Add `mod turtle_kinds_src;` |
| `Cargo.toml` (workspace root) | Add 5-6 crate entries to `[workspace.dependencies]` |

### Estimated syntax kinds

```rust
pub const TURTLE_KINDS_SRC: KindsSrc = KindsSrc {
    punct: &[
        (".", "DOT"), (";", "SEMICOLON"), (",", "COMMA"),
        ("[", "L_BRACK"), ("]", "R_BRACK"),
        ("(", "L_PAREN"), (")", "R_PAREN"),
        ("^^", "CARET_CARET"),
    ],                              // 8 punctuation tokens
    keywords: &[
        ("@prefix", "PREFIX_KW"), ("@base", "BASE_KW"),
        ("a", "A_KW"), ("true", "TRUE_KW"), ("false", "FALSE_KW"),
        ("BASE", "SPARQL_BASE_KW"), ("PREFIX", "SPARQL_PREFIX_KW"),
    ],                              // 7 keywords
    literals: &[
        "IRIREF", "PNAME_NS", "PNAME_LN", "BLANK_NODE_LABEL",
        "LANGTAG", "INTEGER", "DECIMAL", "DOUBLE",
        "STRING_LITERAL_QUOTE", "STRING_LITERAL_SINGLE_QUOTE",
        "STRING_LITERAL_LONG_QUOTE", "STRING_LITERAL_LONG_SINGLE_QUOTE",
    ],                              // 12 literal types
    tokens: &[
        "NEWLINE", "WHITESPACE", "COMMENT", "ANON",
    ],                              // 4 token types
    nodes: &[
        "TURTLE_ROOT", "TURTLE_STATEMENT_LIST",
        "TURTLE_PREFIX_DECLARATION", "TURTLE_BASE_DECLARATION",
        "TURTLE_SPARQL_PREFIX", "TURTLE_SPARQL_BASE",
        "TURTLE_TRIPLES", "TURTLE_PREDICATE_OBJECT_LIST",
        "TURTLE_PREDICATE_OBJECT_PAIR", "TURTLE_OBJECT_LIST",
        "TURTLE_SUBJECT", "TURTLE_PREDICATE", "TURTLE_OBJECT",
        "TURTLE_IRI", "TURTLE_PREFIXED_NAME",
        "TURTLE_BLANK_NODE", "TURTLE_BLANK_NODE_PROPERTY_LIST",
        "TURTLE_COLLECTION", "TURTLE_LITERAL",
        "TURTLE_RDF_LITERAL", "TURTLE_NUMERIC_LITERAL",
        "TURTLE_BOOLEAN_LITERAL", "TURTLE_DATATYPE_ANNOTATION",
        "TURTLE_LANG_TAG",
    ],                              // ~24 node types
};
```

## Service Integration

### File handler (`biome_service/src/file_handlers/turtle.rs`)

Implement `ServiceLanguage` for `TurtleLanguage`:
- `TurtleFormatterSettings`
- `TurtleLinterSettings`
- `TurtleAssistSettings`
- `TurtleParserSettings`

### File source registration (`file_handlers/mod.rs`)

Add `Turtle(TurtleFileSource)` variant to `DocumentFileSource` enum and wire up:
- `try_from_extension`: `.ttl`, `.turtle`, `.nt`, `.nq`, `.trig`
- `try_from_language_id`: `"turtle"`, `"ntriples"`, `"trig"`

### Configuration (`biome_configuration/src/turtle.rs`)

```rust
pub struct TurtleConfiguration {
    pub formatter: Option<TurtleFormatterConfiguration>,
    pub linter: Option<TurtleLinterConfiguration>,
    pub assist: Option<TurtleAssistConfiguration>,
}
```

Wire into `lib.rs` and `settings.rs`.

## Candidate Lint Rules

### Correctness (validation — catches real errors)

| Rule | Description |
|---|---|
| `noUndefinedPrefix` | Prefixed name uses a prefix not declared with `@prefix` |
| `noDuplicatePrefixDeclaration` | Same prefix declared twice with different IRIs |
| `noMalformedIri` | IRI contains disallowed characters or is syntactically invalid |
| `noMalformedLangTag` | Language tag doesn't match BCP 47 pattern |
| `noMalformedDatatype` | Literal value doesn't match its declared XSD datatype |
| `noUndefinedBlankNodeReference` | Blank node referenced but never defined (in TriG context) |
| `noMissingSemicolonBeforeDot` | Likely missing `;` separator in predicate-object list |

### Suspicious (likely mistakes)

| Rule | Description |
|---|---|
| `noDuplicateTriple` | Exact same subject-predicate-object stated twice |
| `noDuplicateSubjectBlock` | Same subject appears in multiple separate blocks (should merge) |
| `noUnusedPrefix` | Prefix declared but never used in the document |
| `noEmptyPrefixedName` | Using bare prefix (e.g. `:`) without local name where likely unintentional |
| `noRdfTypeIri` | Using `rdf:type` instead of the shorthand `a` |

### Style (conventions)

| Rule | Description |
|---|---|
| `useSortedPrefixes` | Prefix declarations should be alphabetically sorted |
| `useConsistentPrefixStyle` | Prefer `@prefix` over `PREFIX` (or vice versa) |
| `useConsistentBaseStyle` | Prefer `@base` over `BASE` (or vice versa) |
| `useConsistentStringQuotes` | Use `"` consistently over `'` for short strings |
| `useTypeShorthand` | Use `a` instead of `rdf:type` |
| `usePrefixedNames` | Prefer prefixed names over full IRIs when a prefix is available |
| `useGroupedSubjects` | Triples with the same subject should be grouped using `;` |
| `useBlanksAroundBlocks` | Blank lines between subject blocks |

### Accessibility / Ontology Quality (stretch)

| Rule | Description |
|---|---|
| `useRdfsLabel` | Resources should have an `rdfs:label` |
| `useRdfsComment` | Classes and properties should have an `rdfs:comment` |

## Comparison with Existing Turtle Tools

| Tool | Type | Notes |
|---|---|---|
| [serdi](https://github.com/drobilla/serd) | C parser/serializer | Fastest, streaming, validation via re-serialization |
| [rapper](https://librdf.org/raptor/) | C parser | Part of Raptor/Redland, multi-format |
| [riot](https://jena.apache.org/) | Java (Jena) | Full RDF toolkit, extensive validation |
| [turtle-validator](https://github.com/IDLabResearch/TurtleValidator) | Node.js | Syntax + XSD datatype validation |

None of these provide lint-style rules (style enforcement, suspicious pattern detection). A biome implementation would be the first tool to offer both validation and linting for Turtle in a single pass.

## Turtle vs GraphQL Complexity Comparison

| Aspect | GraphQL | Turtle |
|---|---|---|
| Grammar productions | ~90 | ~19 |
| `.ungram` lines | 578 | ~200-350 est. |
| Syntax node kinds | 80+ | ~24 |
| Keyword count | 38 | 7 |
| Lexer complexity | Medium | Medium (IRI/prefix parsing) |
| Formatter complexity | High (query formatting) | Medium (triple alignment) |
| Semantic model needed | Yes (operations, types) | Yes (prefix map, subject grouping) |

Turtle is structurally simpler than GraphQL but has lexer-level complexity in IRI and prefixed name parsing.
