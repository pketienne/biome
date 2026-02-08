# Turtle Language Support — Implementation Plan

## Status: Draft

## Summary

Add full Turtle (.ttl) language support to Biome: syntax types, parser, formatter, analyzer (linter), and service integration. Follows the established per-language crate pattern used by GraphQL and Markdown.

---

## Phase 0: Codegen Foundation

**Goal:** Define the grammar and syntax kinds so the code generator can produce typed AST nodes, syntax kinds, and factory methods.

### 0.1 Create `xtask/codegen/src/turtle_kinds_src.rs`

Define `TURTLE_KINDS_SRC: KindsSrc` with:

```
Punctuation (8):  .  ;  ,  [  ]  (  )  ^^
Keywords (7):     @prefix  @base  a  true  false  BASE  PREFIX
Literals (12):    IRIREF  PNAME_NS  PNAME_LN  BLANK_NODE_LABEL  LANGTAG
                  INTEGER  DECIMAL  DOUBLE
                  STRING_LITERAL_QUOTE  STRING_LITERAL_SINGLE_QUOTE
                  STRING_LITERAL_LONG_QUOTE  STRING_LITERAL_LONG_SINGLE_QUOTE
Tokens (4):       ERROR_TOKEN  NEWLINE  WHITESPACE  COMMENT
Nodes (~26):      TURTLE_ROOT  TURTLE_STATEMENT_LIST
                  TURTLE_PREFIX_DECLARATION  TURTLE_BASE_DECLARATION
                  TURTLE_SPARQL_PREFIX  TURTLE_SPARQL_BASE
                  TURTLE_TRIPLES  TURTLE_PREDICATE_OBJECT_LIST
                  TURTLE_PREDICATE_OBJECT_PAIR  TURTLE_OBJECT_LIST
                  TURTLE_SUBJECT  TURTLE_PREDICATE  TURTLE_OBJECT
                  TURTLE_IRI  TURTLE_PREFIXED_NAME
                  TURTLE_BLANK_NODE  TURTLE_BLANK_NODE_PROPERTY_LIST
                  TURTLE_COLLECTION  TURTLE_LITERAL
                  TURTLE_RDF_LITERAL  TURTLE_NUMERIC_LITERAL
                  TURTLE_BOOLEAN_LITERAL  TURTLE_DATATYPE_ANNOTATION
                  TURTLE_LANG_TAG
                  TURTLE_BOGUS  TURTLE_BOGUS_STATEMENT
```

**Reference:** `xtask/codegen/src/graphql_kind_src.rs`, `xtask/codegen/src/markdown_kinds_src.rs`

### 0.2 Create `xtask/codegen/turtle.ungram`

Write the ungrammar specification (~200-350 lines) mapping the W3C Turtle grammar to Biome's CST model. Key nodes:

```ungram
// Error recovery
TurtleBogus = SyntaxElement*
TurtleBogusStatement = SyntaxElement*

// Root
TurtleRoot = bom: 'UNICODE_BOM'? statements: TurtleStatementList

// Lists
TurtleStatementList = (TurtleDirective | TurtleTriples)*

// Directives
TurtleDirective = TurtlePrefixDeclaration | TurtleBaseDeclaration
                | TurtleSparqlPrefix | TurtleSparqlBase

TurtlePrefixDeclaration = '@prefix' namespace: PNAME_NS iri: IRIREF '.'
TurtleBaseDeclaration = '@base' iri: IRIREF '.'
TurtleSparqlPrefix = 'PREFIX' namespace: PNAME_NS iri: IRIREF
TurtleSparqlBase = 'BASE' iri: IRIREF

// Triples
TurtleTriples = subject: TurtleSubject predicates: TurtlePredicateObjectList '.'

TurtleSubject = TurtleIri | TurtleBlankNode | TurtleCollection
              | TurtleBlankNodePropertyList

TurtlePredicateObjectList = pairs: TurtlePredicateObjectPair
                            (';' TurtlePredicateObjectPair?)*

TurtlePredicateObjectPair = verb: TurtleVerb objects: TurtleObjectList

TurtleVerb = TurtlePredicate | 'a'
TurtlePredicate = TurtleIri

TurtleObjectList = TurtleObject (',' TurtleObject)*

TurtleObject = TurtleIri | TurtleBlankNode | TurtleCollection
             | TurtleBlankNodePropertyList | TurtleLiteral

// IRIs
TurtleIri = IRIREF | TurtlePrefixedName
TurtlePrefixedName = PNAME_LN | PNAME_NS

// Blank nodes
TurtleBlankNode = BLANK_NODE_LABEL | ANON
TurtleBlankNodePropertyList = '[' predicates: TurtlePredicateObjectList ']'

// Collections
TurtleCollection = '(' TurtleObject* ')'

// Literals
TurtleLiteral = TurtleRdfLiteral | TurtleNumericLiteral | TurtleBooleanLiteral

TurtleRdfLiteral = value: TurtleString
                   (TurtleLangTag | TurtleDatatypeAnnotation)?

TurtleDatatypeAnnotation = '^^' datatype: TurtleIri
TurtleLangTag = LANGTAG

TurtleString = STRING_LITERAL_QUOTE | STRING_LITERAL_SINGLE_QUOTE
             | STRING_LITERAL_LONG_QUOTE | STRING_LITERAL_LONG_SINGLE_QUOTE

TurtleNumericLiteral = INTEGER | DECIMAL | DOUBLE
TurtleBooleanLiteral = 'true' | 'false'
```

**Reference:** `xtask/codegen/graphql.ungram`, `xtask/codegen/markdown.ungram`

### 0.3 Register in `xtask/codegen/src/language_kind.rs`

Add `Turtle` variant to:
- `LanguageKind` enum
- `LANGUAGE_PREFIXES` array (prefix: `"Turtle"`)
- `ALL_LANGUAGE_KIND` array
- `Display` impl
- `FromStr` impl
- `kinds()` method → return `&TURTLE_KINDS_SRC`
- `load_grammar()` method → load `turtle.ungram`
- `syntax_crate_name()`, `factory_crate_name()`, `formatter_crate_name()` (via existing macro)

### 0.4 Register in `xtask/codegen/src/lib.rs`

Add `pub mod turtle_kinds_src;`

### 0.5 Run codegen

```shell
cargo run -p xtask_codegen -- grammar turtle
```

This generates:
- `crates/biome_turtle_syntax/src/generated/kind.rs`
- `crates/biome_turtle_syntax/src/generated/nodes.rs`
- `crates/biome_turtle_syntax/src/generated/nodes_mut.rs`
- `crates/biome_turtle_syntax/src/generated/macros.rs`
- `crates/biome_turtle_factory/src/generated/node_factory.rs`
- `crates/biome_turtle_factory/src/generated/syntax_factory.rs`

**Depends on:** Crate skeletons from Phase 1 must exist first (codegen writes into them). In practice, Phase 0 and Phase 1 are done together — create skeleton crates, then run codegen.

---

## Phase 1: Syntax Crate (`biome_turtle_syntax`)

**Goal:** Typed AST nodes and syntax infrastructure that all other crates depend on.

### 1.1 Create crate skeleton

```
crates/biome_turtle_syntax/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── syntax_node.rs
    ├── file_source.rs
    ├── generated.rs
    └── generated/        # populated by codegen
```

**Cargo.toml dependencies:**
- `biome_rowan` (workspace)

### 1.2 Implement `syntax_node.rs`

Define `TurtleLanguage` type implementing `biome_rowan::Language`:
- `type Kind = TurtleSyntaxKind`
- Type aliases: `TurtleSyntaxNode`, `TurtleSyntaxToken`, `TurtleSyntaxElement`, `TurtleSyntaxNodeChildren`, `TurtleSyntaxElementChildren`, `TurtleSyntaxList`

**Reference:** `crates/biome_graphql_syntax/src/syntax_node.rs`

### 1.3 Implement `lib.rs`

- Re-export generated types and syntax node types
- Implement `SyntaxKind` trait for `TurtleSyntaxKind`:
  - `TOMBSTONE`, `EOF` constants
  - `is_bogus()` → matches `TURTLE_BOGUS | TURTLE_BOGUS_STATEMENT`
  - `to_bogus()` → convert to nearest bogus node
  - `is_root()` → `TurtleRoot::can_cast(*self)`
  - `is_trivia()` → matches `NEWLINE | WHITESPACE | COMMENT`
- Implement `From<u16>` for `TurtleSyntaxKind`
- Implement `TryFrom<TurtleSyntaxKind>` for `TriviaPieceKind`

**Reference:** `crates/biome_graphql_syntax/src/lib.rs`

### 1.4 Implement `file_source.rs`

```rust
pub struct TurtleFileSource {
    variant: TurtleVariant,
}

enum TurtleVariant {
    #[default]
    Turtle,    // .ttl, .turtle
    NTriples,  // .nt
    // Future: NQuads (.nq), TriG (.trig)
}
```

Methods:
- `turtle()`, `ntriples()` constructors
- `try_from_extension()`: match `"ttl" | "turtle"` → Turtle, `"nt"` → NTriples
- `try_from_language_id()`: match `"turtle"` → Turtle, `"ntriples"` → NTriples
- `try_from_well_known()` → `Err(UnknownFileName)`

**Note:** Start with `.ttl` and `.turtle` only. Add `.nt`, `.nq`, `.trig` as future extensions once the parser handles their differences.

### 1.5 Register in workspace `Cargo.toml`

Add to `[workspace.dependencies]`:
```toml
biome_turtle_syntax = { path = "./crates/biome_turtle_syntax", version = "0.0.1" }
```

### Acceptance Criteria
- [ ] `cargo check -p biome_turtle_syntax` passes
- [ ] Codegen produces all generated files without errors
- [ ] `TurtleSyntaxKind` enum has all expected variants

---

## Phase 2: Factory Crate (`biome_turtle_factory`)

**Goal:** Provide AST node construction utilities for the parser and tests.

### 2.1 Create crate skeleton

```
crates/biome_turtle_factory/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── make.rs
    ├── generated.rs
    └── generated/        # populated by codegen
```

**Cargo.toml dependencies:**
- `biome_rowan` (workspace)
- `biome_turtle_syntax` (workspace)

### 2.2 Implement `lib.rs` and `make.rs`

- `lib.rs`: re-export generated factory + `make` module
- `make.rs`: manual convenience constructors (initially empty, add as needed)

### 2.3 Register in workspace `Cargo.toml`

```toml
biome_turtle_factory = { path = "./crates/biome_turtle_factory", version = "0.0.1" }
```

### Acceptance Criteria
- [ ] `cargo check -p biome_turtle_factory` passes
- [ ] Generated `node_factory.rs` and `syntax_factory.rs` exist

---

## Phase 3: Parser Crate (`biome_turtle_parser`)

**Goal:** Lex and parse Turtle source into a lossless CST. This is the core deliverable.

### 3.1 Create crate skeleton

```
crates/biome_turtle_parser/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── token_source.rs
    ├── lexer/
    │   ├── mod.rs
    │   └── tests.rs
    └── parser/
        ├── mod.rs
        ├── parse_error.rs
        ├── directive.rs
        ├── triples.rs
        ├── literal.rs
        ├── iri.rs
        ├── blank_node.rs
        └── collection.rs
```

**Cargo.toml dependencies:**
- `biome_parser` (workspace)
- `biome_turtle_syntax` (workspace)
- `biome_turtle_factory` (workspace)
- `biome_rowan` (workspace)
- `biome_unicode_table` (workspace)

### 3.2 Implement the Lexer (`lexer/mod.rs`)

The lexer is the most complex part due to Turtle-specific token types. Implement `Lexer` trait for `TurtleLexer`:

**Token categories:**

| Token | Complexity | Notes |
|---|---|---|
| `IRIREF` | Medium | `<` ... `>` with UCHAR escapes, no whitespace/`<`/`>` inside |
| `PNAME_NS` | Medium | Optional prefix + `:` (e.g., `foaf:`, `:`) |
| `PNAME_LN` | High | Prefix + `:` + local name with complex char rules and escapes |
| `BLANK_NODE_LABEL` | Medium | `_:` + name chars |
| `LANGTAG` | Medium | `@` + `[a-zA-Z]+` + (`-` `[a-zA-Z0-9]+`)* — must disambiguate from `@prefix`/`@base` |
| `STRING_LITERAL_*` (4 types) | Medium | Single/double, short/long; handle escapes |
| `INTEGER`, `DECIMAL`, `DOUBLE` | Low-Medium | Optional sign, decimal point, exponent |
| `COMMENT` | Low | `#` to end of line |
| Keywords | Low | `@prefix`, `@base`, `a`, `true`, `false`, `BASE`, `PREFIX` |
| Punctuation | Low | `.`, `;`, `,`, `[`, `]`, `(`, `)`, `^^` |

**Key lexer challenges:**
1. **`@` disambiguation:** `@prefix` and `@base` are keywords; `@en`, `@en-US` are language tags. Strategy: after lexing `@`, peek ahead — if followed by `prefix` or `base` + boundary, emit keyword; otherwise emit `LANGTAG`.
2. **`.` disambiguation:** `.` terminates statements, but also appears in `DECIMAL` and `DOUBLE` (e.g., `3.14`). After a digit, `.` followed by digit is part of the number.
3. **Prefixed names vs keywords:** `a` is a keyword (shorthand for `rdf:type`) but only in verb position. Lexer emits `A_KW`; parser handles context.
4. **`^^` vs `^`:** `^^` is datatype separator. Single `^` is not valid in Turtle; lex `^^` as one token.

### 3.3 Implement the Parser

**Entry point (`lib.rs`):**
```rust
pub fn parse_turtle(source: &str, options: TurtleParseOptions) -> TurtleParse {
    // create lexer, token source, parser; call parse_root()
}
```

**Grammar mapping to parser functions:**

| Parser module | W3C rules covered |
|---|---|
| `parser/mod.rs` | `turtleDoc` → `parse_root()` |
| `parser/directive.rs` | `prefixID`, `base`, `sparqlPrefix`, `sparqlBase` |
| `parser/triples.rs` | `triples`, `predicateObjectList`, `objectList`, `verb`, `subject`, `predicate`, `object` |
| `parser/literal.rs` | `literal`, `RDFLiteral`, `NumericLiteral`, `BooleanLiteral`, `String` |
| `parser/iri.rs` | `iri`, `PrefixedName` |
| `parser/blank_node.rs` | `BlankNode`, `blankNodePropertyList`, `ANON` |
| `parser/collection.rs` | `collection` |

**Error recovery strategy:**
- On unexpected token in statement position: skip to next `.` or `@` or `PREFIX`/`BASE` and wrap skipped tokens in `TurtleBogusStatement`
- On unexpected token inside predicate-object list: skip to next `;` or `.`
- On unexpected token inside object list: skip to next `,` or `;` or `.`
- Missing `.` at end of triples: emit diagnostic, continue

### 3.4 Implement `token_source.rs`

Standard `TokenSource` implementation wrapping the lexer, with trivia handling (whitespace, newlines, comments attached as leading/trailing trivia).

**Reference:** `crates/biome_graphql_parser/src/parser/token_source.rs`

### 3.5 Implement `parse_error.rs`

Error message functions:
- `expected_directive()`, `expected_iri()`, `expected_subject()`, `expected_predicate()`
- `expected_object()`, `expected_dot()`, `expected_semicolon()`

### 3.6 Add tests

```
crates/biome_turtle_parser/tests/
├── spec_tests.rs
├── spec_test.rs
└── turtle_test_suite/
    ├── ok/
    │   ├── basic_triples.ttl
    │   ├── prefix_declarations.ttl
    │   ├── blank_nodes.ttl
    │   ├── collections.ttl
    │   ├── literals.ttl
    │   ├── string_types.ttl
    │   ├── numeric_literals.ttl
    │   ├── multiline_triples.ttl
    │   ├── comments.ttl
    │   ├── sparql_keywords.ttl
    │   └── w3c_example.ttl        # full W3C spec example
    └── err/
        ├── missing_dot.ttl
        ├── undefined_escape.ttl
        ├── unterminated_iri.ttl
        ├── unterminated_string.ttl
        └── missing_object.ttl
```

Test runner pattern (spec_tests.rs):
```rust
mod spec_test;
mod ok {
    tests_macros::gen_tests! {
        "tests/turtle_test_suite/ok/**/*.ttl",
        crate::spec_test::run, "ok"
    }
}
mod err {
    tests_macros::gen_tests! {
        "tests/turtle_test_suite/err/**/*.ttl",
        crate::spec_test::run, "error"
    }
}
```

### 3.7 Register in workspace `Cargo.toml`

```toml
biome_turtle_parser = { path = "./crates/biome_turtle_parser", version = "0.0.1" }
```

### Acceptance Criteria
- [ ] Lexer correctly tokenizes all Turtle token types including edge cases
- [ ] Parser produces valid CST for the W3C Turtle spec examples
- [ ] Error recovery works: malformed input produces diagnostics without panicking
- [ ] All snapshot tests pass with `cargo insta accept`
- [ ] `cargo test -p biome_turtle_parser` passes

---

## Phase 4: Formatter Crate (`biome_turtle_formatter`)

**Goal:** Pretty-print Turtle CSTs with configurable style.

### 4.1 Create crate skeleton

```
crates/biome_turtle_formatter/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── context.rs
    ├── cst.rs
    ├── comments.rs
    ├── generated.rs
    ├── turtle/
    │   ├── mod.rs
    │   ├── auxiliary/     # smaller nodes
    │   ├── bogus/         # bogus node formatting
    │   └── any/           # enum/union formatting
    └── tests/
```

**Cargo.toml dependencies:**
- `biome_formatter` (workspace)
- `biome_turtle_syntax` (workspace)
- `biome_rowan` (workspace)

### 4.2 Define format options (`context.rs`)

```rust
pub struct TurtleFormatOptions {
    indent_style: IndentStyle,
    indent_width: IndentWidth,
    line_width: LineWidth,
    line_ending: LineEnding,
    quote_style: QuoteStyle,       // " vs '
}
```

### 4.3 Implement per-node formatting

**Key formatting rules:**

| Node | Formatting behavior |
|---|---|
| `TurtleRoot` | Statements separated by blank lines between subject blocks |
| `TurtlePrefixDeclaration` | `@prefix ns: <iri> .` — single space separators |
| `TurtleBaseDeclaration` | `@base <iri> .` |
| `TurtleTriples` | Subject on first line; predicate-object list indented if multi-line |
| `TurtlePredicateObjectList` | Each pair on its own line, separated by `;`, indented one level |
| `TurtleObjectList` | Single line if short; wrapped with `,` if long |
| `TurtleBlankNodePropertyList` | `[ ... ]` inline if single pair; expanded if multiple |
| `TurtleCollection` | `( ... )` inline if short; wrapped if long |
| `TurtleRdfLiteral` | Normalize quote style; preserve long strings |
| Comments | Preserve position; attach to nearest node |

**Example formatted output:**
```turtle
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

<http://example.org/alice>
    a foaf:Person ;
    foaf:name "Alice" ;
    foaf:knows <http://example.org/bob>,
        <http://example.org/carol> .
```

### 4.4 Implement comment handling (`comments.rs`)

Turtle has line comments (`# ...`). Attach as:
- **Leading trivia** when comment is on its own line before a node
- **Trailing trivia** when comment follows a node on the same line

### 4.5 Add tests

- Snapshot tests for each node type
- Round-trip tests (format → re-parse → compare CST)
- Edge case tests (empty files, comment-only files, very long IRIs)

### 4.6 Register in workspace `Cargo.toml`

```toml
biome_turtle_formatter = { path = "./crates/biome_turtle_formatter", version = "0.0.1" }
```

### Acceptance Criteria
- [ ] Formatting is idempotent (format twice = same result)
- [ ] All trivia (whitespace, comments) is preserved
- [ ] `cargo test -p biome_turtle_formatter` passes

---

## Phase 5: Analyzer Crate (`biome_turtle_analyze`)

**Goal:** Lint rules and validation for Turtle files.

### 5.1 Create crate skeleton

```
crates/biome_turtle_analyze/
├── build.rs
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── registry.rs           # generated
    ├── suppression_action.rs
    └── lint/
        ├── mod.rs
        ├── nursery.rs        # generated group module
        └── nursery/          # initial rules go in nursery
```

**Cargo.toml dependencies:**
- `biome_analyze` (workspace)
- `biome_analyze_macros` (workspace)
- `biome_turtle_syntax` (workspace)
- `biome_rowan` (workspace)
- `biome_console` (workspace)
- `biome_diagnostics` (workspace)
- `biome_suppression` (workspace)

### 5.2 Implement framework (`lib.rs`)

- `METADATA: LazyLock<MetadataRegistry>` static
- `analyze()` function wiring parser output to rule visitor
- `visit_registry()` for rule registration

### 5.3 Implement `build.rs`

Watch lint rule directories for changes and trigger recompilation.

### 5.4 Implement `suppression_action.rs`

Support `// biome-ignore` comment suppression (Turtle uses `#` comments, so suppression format: `# biome-ignore ruleName: reason`).

### 5.5 Initial lint rules (nursery)

Start with rules that need only syntax-level analysis (no semantic model):

| Priority | Rule | Category | Complexity |
|---|---|---|---|
| 1 | `noUndefinedPrefix` | correctness | Medium — track `@prefix` declarations, check all `PNAME_LN`/`PNAME_NS` uses |
| 2 | `noUnusedPrefix` | suspicious | Medium — track `@prefix` declarations, verify each is used |
| 3 | `noDuplicatePrefixDeclaration` | correctness | Low — check for duplicate `@prefix` namespace declarations |
| 4 | `useConsistentPrefixStyle` | style | Low — flag mixed `@prefix` and `PREFIX` usage |
| 5 | `useConsistentBaseStyle` | style | Low — flag mixed `@base` and `BASE` usage |
| 6 | `noRdfTypeIri` | suspicious | Low — flag `rdf:type` when `a` shorthand exists |

Rules 1-2 require walking the entire file to build a prefix map. Implement as file-level visitors.

### 5.6 Run codegen

```shell
just gen-analyzer
```

### 5.7 Register in workspace `Cargo.toml`

```toml
biome_turtle_analyze = { path = "./crates/biome_turtle_analyze", version = "0.0.1" }
```

### Acceptance Criteria
- [ ] `cargo test -p biome_turtle_analyze` passes
- [ ] At least `noUndefinedPrefix` and `noUnusedPrefix` produce correct diagnostics
- [ ] Suppression comments work

---

## Phase 6: Service Integration

**Goal:** Wire Turtle support into Biome's CLI, LSP, and configuration system.

### 6.1 Configuration (`biome_configuration/src/turtle.rs`)

Create:
```rust
pub struct TurtleConfiguration {
    pub formatter: Option<TurtleFormatterConfiguration>,
    pub linter: Option<TurtleLinterConfiguration>,
    pub assist: Option<TurtleAssistConfiguration>,
}

pub struct TurtleFormatterConfiguration {
    pub enabled: Option<TurtleFormatterEnabled>,
    pub indent_style: Option<IndentStyle>,
    pub indent_width: Option<IndentWidth>,
    pub line_ending: Option<LineEnding>,
    pub line_width: Option<LineWidth>,
    pub quote_style: Option<QuoteStyle>,
}

pub struct TurtleLinterConfiguration {
    pub enabled: Option<TurtleLinterEnabled>,
}

pub struct TurtleAssistConfiguration {
    pub enabled: Option<TurtleAssistEnabled>,
}
```

### 6.2 Register configuration

**Modify `biome_configuration/src/lib.rs`:**
- Add `pub mod turtle;`
- Add `pub use turtle::{TurtleConfiguration, turtle_configuration};`
- Add `pub turtle: Option<TurtleConfiguration>` field to `Configuration` struct
- Add getter methods: `get_turtle_formatter_configuration()`, `get_turtle_linter_configuration()`

**Modify `biome_configuration/src/overrides.rs`:**
- Add `pub turtle: Option<TurtleConfiguration>` to `OverridePattern`

### 6.3 File handler (`biome_service/src/file_handlers/turtle.rs`)

Create handler implementing `ServiceLanguage` for `TurtleLanguage`:

```rust
pub struct TurtleFormatterSettings { /* indent_style, indent_width, line_width, ... */ }
pub struct TurtleLinterSettings { /* enabled */ }
pub struct TurtleAssistSettings { /* enabled */ }

impl ServiceLanguage for TurtleLanguage {
    type FormatterSettings = TurtleFormatterSettings;
    type LinterSettings = TurtleLinterSettings;
    type AssistSettings = TurtleAssistSettings;
    type FormatOptions = TurtleFormatOptions;
    type ParserSettings = ();
    type ParserOptions = ();
    // ...
}
```

Implement `ExtensionHandler` trait with capabilities:
- Parse, format, lint, code actions (suppression)
- No semantic model initially (add later)

### 6.4 Register in `file_handlers/mod.rs`

- Add `pub(crate) mod turtle;`
- Import `biome_turtle_analyze::METADATA as turtle_metadata`
- Import `biome_turtle_syntax::{TurtleFileSource, TurtleLanguage}`
- Add `Turtle(TurtleFileSource)` to `DocumentFileSource` enum
- Add `TurtleFileHandler` to `ExtensionHandlers` struct
- Wire `try_from_extension`, `try_from_language_id`, `get_capabilities`
- Register `visit_registry` calls for syntax, lint, assist

### 6.5 Settings integration (`biome_service/src/settings.rs`)

- Add `turtle: LanguageSettings<TurtleLanguage>` to `LanguageListSettings`
- Add configuration merge in `merge_with_configuration()`
- Add override handling in `to_override_settings()`
- Add rule push in `override_analyzer_rules()`

### Acceptance Criteria
- [ ] `biome check example.ttl` works from CLI
- [ ] `biome format example.ttl` produces formatted output
- [ ] `biome lint example.ttl` runs analyzer rules
- [ ] Configuration in `biome.json` is respected
- [ ] LSP `textDocument/formatting` works for `.ttl` files

---

## Phase 7: Semantic Model (Optional, Future)

**Goal:** Enable cross-statement analysis for advanced lint rules.

### 7.1 Create `biome_turtle_semantic`

```
crates/biome_turtle_semantic/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── events.rs
    └── semantic_model/
        ├── mod.rs
        ├── prefix_map.rs       # @prefix ns → IRI resolution
        └── blank_node_scope.rs  # blank node tracking
```

### 7.2 Semantic model data

- **Prefix map:** Maps declared prefixes to their IRI namespaces
- **Blank node scope:** Tracks blank node labels and their scope
- **Subject index:** Maps subjects to their predicate-object lists (enables `noDuplicateSubjectBlock`)

### 7.3 Advanced rules enabled by semantic model

- `noDuplicateTriple` — exact triple deduplication
- `noDuplicateSubjectBlock` — subject appears in multiple blocks
- `noMalformedDatatype` — validate literal values against XSD datatypes
- `usePrefixedNames` — suggest prefixed names when a prefix covers the IRI
- `useGroupedSubjects` — detect ungrouped triples with same subject

---

## Execution Order and Dependencies

```
Phase 0 (Codegen) ──┐
                     ├──> Phase 1 (Syntax) ──> Phase 2 (Factory)
                     │                                  │
                     │                                  v
                     │                          Phase 3 (Parser)
                     │                            │         │
                     │                            v         v
                     │                    Phase 4 (Fmt)  Phase 5 (Analyze)
                     │                            │         │
                     │                            v         v
                     └──────────────────> Phase 6 (Service Integration)
                                                  │
                                                  v
                                          Phase 7 (Semantic — future)
```

**Phases 0+1+2** must be done first (codegen + syntax + factory).
**Phase 3** (parser) depends on 0+1+2.
**Phases 4 and 5** can be done in parallel after Phase 3.
**Phase 6** requires all prior phases.
**Phase 7** is independent and can be deferred.

---

## Files Created (New)

| File | Phase |
|---|---|
| `xtask/codegen/turtle.ungram` | 0 |
| `xtask/codegen/src/turtle_kinds_src.rs` | 0 |
| `crates/biome_turtle_syntax/` (entire crate) | 1 |
| `crates/biome_turtle_factory/` (entire crate) | 2 |
| `crates/biome_turtle_parser/` (entire crate) | 3 |
| `crates/biome_turtle_formatter/` (entire crate) | 4 |
| `crates/biome_turtle_analyze/` (entire crate) | 5 |
| `crates/biome_configuration/src/turtle.rs` | 6 |
| `crates/biome_service/src/file_handlers/turtle.rs` | 6 |
| `crates/biome_turtle_semantic/` (entire crate) | 7 |

## Files Modified

| File | Phase | Change |
|---|---|---|
| `xtask/codegen/src/language_kind.rs` | 0 | Add `Turtle` variant |
| `xtask/codegen/src/lib.rs` | 0 | Add `mod turtle_kinds_src` |
| `Cargo.toml` (workspace root) | 1-5 | Add crate dependencies |
| `crates/biome_configuration/src/lib.rs` | 6 | Add turtle module + config fields |
| `crates/biome_configuration/src/overrides.rs` | 6 | Add turtle override field |
| `crates/biome_service/src/file_handlers/mod.rs` | 6 | Add DocumentFileSource::Turtle + handler |
| `crates/biome_service/src/settings.rs` | 6 | Add turtle to LanguageListSettings |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Lexer complexity (IRI/prefix parsing) | High | Medium | Start with strict W3C compliance; add error tolerance later |
| `.` disambiguation (statement terminator vs decimal) | Medium | Medium | Use parser-assisted lexing or lookahead in lexer |
| `@` disambiguation (keyword vs lang tag) | Medium | Low | Peek-ahead in lexer; well-defined fallback |
| Codegen compatibility issues | Low | High | Run codegen early; iterate on `.ungram` |
| Formatter edge cases | Medium | Low | Start minimal; add formatting rules incrementally |

---

## Estimated Scope

| Phase | New files | Modified files | Estimated lines |
|---|---|---|---|
| 0: Codegen | 2 | 2 | ~400 |
| 1: Syntax | ~6 | 1 | ~200 + generated |
| 2: Factory | ~4 | 1 | ~50 + generated |
| 3: Parser | ~12 + tests | 1 | ~2000-3000 |
| 4: Formatter | ~15 + tests | 1 | ~1500-2000 |
| 5: Analyzer | ~10 + tests | 1 | ~800-1200 |
| 6: Integration | 2 | 4 | ~600-800 |
| **Total** | **~50+** | **~11** | **~5500-8000** |
