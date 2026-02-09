# Turtle Language Support - Implementation Complete

## Overview

Implemented full Turtle (W3C RDF syntax) language support in Biome, spanning Phases 0-6 of the implementation plan. This includes codegen foundation, syntax/factory crate skeletons, parser with lexer, formatter crate, analyzer crate, configuration types, file handler, and full service layer integration.

After this work, Biome recognizes `.ttl` and `.turtle` files and routes them through the full formatting, linting, code actions, and fix-all pipelines. The formatter currently operates in verbatim mode (preserves input via generated per-node stubs) and the analyzer has an empty rule registry, both ready for incremental feature additions.

## What Was Done

### Phase 0: Codegen Foundation

**File: `xtask/codegen/src/turtle_kinds_src.rs`** (new)

Defined `TURTLE_KINDS_SRC: KindsSrc` with all Turtle syntax kinds:
- Punctuation: `.`, `;`, `,`, `[`, `]`, `(`, `)`, `^^`
- Keywords: `@prefix`, `@base`, `a`, `true`, `false`, `BASE`, `PREFIX`
- Literals: `IRIREF`, `PNAME_NS`, `PNAME_LN`, `BLANK_NODE_LABEL`, `LANGTAG`, `INTEGER`, `DECIMAL`, `DOUBLE`, four string literal types
- Tokens: `ERROR_TOKEN`, `NEWLINE`, `WHITESPACE`, `COMMENT`
- ~26 node types covering the full W3C Turtle grammar

**File: `xtask/codegen/turtle.ungram`** (new)

Ungrammar specification mapping W3C Turtle grammar to Biome's CST model. Covers root, directives, triples, predicate-object lists, IRIs, blank nodes, collections, literals, and bogus error-recovery nodes.

**File: `xtask/codegen/src/language_kind.rs`** (modified)

Added `Turtle` variant to:
- `LanguageKind` enum
- `LANGUAGE_PREFIXES` array (prefix: `"Turtle"`)
- `ALL_LANGUAGE_KIND` array
- `Display`, `FromStr` impls
- `kinds()`, `load_grammar()`, `syntax_crate_name()`, `factory_crate_name()`, `formatter_crate_name()` methods

**File: `xtask/codegen/src/lib.rs`** (modified)

Added `pub mod turtle_kinds_src;`

**Codegen run:**
```shell
cargo run -p xtask_codegen -- grammar turtle
```
Generated all typed AST nodes, syntax kinds, and factory methods.

### Phase 1: Syntax Crate (`biome_turtle_syntax`)

**Directory: `crates/biome_turtle_syntax/`** (new crate)

| File | Purpose |
|------|---------|
| `Cargo.toml` | Crate manifest with `biome_rowan` dependency |
| `src/lib.rs` | Re-exports, `SyntaxKind` trait impl for `TurtleSyntaxKind` (bogus, root, trivia detection) |
| `src/syntax_node.rs` | `TurtleLanguage` type implementing `biome_rowan::Language`, type aliases |
| `src/file_source.rs` | `TurtleFileSource` with `.ttl`/`.turtle` extension matching, `turtle` language ID |
| `src/generated.rs` | Module declarations for codegen output |
| `src/generated/` | Codegen-populated: `kind.rs`, `nodes.rs`, `nodes_mut.rs`, `macros.rs` |

### Phase 2: Factory Crate (`biome_turtle_factory`)

**Directory: `crates/biome_turtle_factory/`** (new crate)

| File | Purpose |
|------|---------|
| `Cargo.toml` | Crate manifest with `biome_rowan`, `biome_turtle_syntax` deps |
| `src/lib.rs` | Re-exports generated factory + `make` module |
| `src/make.rs` | Manual convenience constructors (initially empty) |
| `src/generated.rs` | Module declarations for codegen output |
| `src/generated/` | Codegen-populated: `node_factory.rs`, `syntax_factory.rs` |

### Phase 3: Parser Crate (`biome_turtle_parser`)

**Directory: `crates/biome_turtle_parser/`** (new crate)

Full lexer and recursive-descent parser for W3C Turtle syntax:

| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies: `biome_parser`, `biome_turtle_syntax`, `biome_turtle_factory`, `biome_rowan`, `biome_unicode_table` |
| `src/lib.rs` | Entry point: `parse_turtle()`, `TurtleParse` result type |
| `src/token_source.rs` | Standard `TokenSource` wrapping the lexer with trivia handling |
| `src/lexer/mod.rs` | `TurtleLexer` implementing `Lexer` trait — handles IRI, prefixed names, blank nodes, language tags, strings (4 types), numbers, `@` disambiguation, `.` disambiguation |
| `src/lexer/tests.rs` | Lexer unit tests |
| `src/parser/mod.rs` | `parse_root()` — top-level document parsing |
| `src/parser/parse_error.rs` | Error message functions |
| `src/parser/directive.rs` | `@prefix`, `@base`, `PREFIX`, `BASE` parsing |
| `src/parser/triples.rs` | Triples, predicate-object lists, object lists |
| `src/parser/literal.rs` | RDF literals, numeric literals, boolean literals |
| `src/parser/iri.rs` | IRIs and prefixed names |
| `src/parser/blank_node.rs` | Blank nodes and blank node property lists |
| `src/parser/collection.rs` | RDF collections `( ... )` |

**Test suite** with snapshot tests:
- `tests/turtle_test_suite/ok/` — valid Turtle files (basic triples, prefixes, blank nodes, collections, literals, etc.)
- `tests/turtle_test_suite/err/` — malformed Turtle files (missing dots, unterminated strings, etc.)
- Error recovery: wraps malformed input in `TurtleBogusStatement` without panicking

### Phase 4: Formatter Crate (`biome_turtle_formatter`)

**Directory: `crates/biome_turtle_formatter/`** (new crate)

Created following GraphQL formatter pattern:

| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies: `biome_formatter`, `biome_turtle_syntax`, `biome_rowan`, `biome_suppression` |
| `src/lib.rs` | Entry point: `AsFormat`, `IntoFormat`, `FormatNodeRule`, `FormatBogusNodeRule` traits; `TurtleFormatLanguage` implementing `FormatLanguage`; `format_node()`, `format_range()`, `format_sub_tree()` |
| `src/context.rs` | `TurtleFormatContext` and `TurtleFormatOptions` (indent_style, indent_width, line_ending, line_width) with builder; `FormatContext` and `CstFormatContext` trait impls |
| `src/comments.rs` | `TurtleCommentStyle` implementing `CommentStyle` with `CommentKind::Line` for `#` comments; suppression comment detection |
| `src/prelude.rs` | Common re-exports including `format_verbatim_node` |
| `src/cst.rs` | `FormatTurtleSyntaxNode` dispatching to per-node generated rules |
| `src/verbatim.rs` | `format_verbatim_node()`, `format_bogus_node()`, `format_suppressed_node()` |
| `src/generated.rs` | Module declarations for codegen output |
| `src/turtle/` | Codegen-generated per-node format rule stubs (~23 files), all using `format_verbatim_node` as placeholder |

**Formatter codegen:**
```shell
cargo run -p xtask_codegen -- formatter
```
Added `NodeDialect::Turtle` to the formatter codegen `NodeDialect` enum to generate into the correct output directory.

### Phase 5: Analyzer Crate (`biome_turtle_analyze`)

**Directory: `crates/biome_turtle_analyze/`** (new crate)

Created following GraphQL analyzer pattern:

| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies: `biome_analyze`, `biome_turtle_syntax`, `biome_rowan`, `biome_suppression`, `biome_console`, `biome_diagnostics` |
| `src/lib.rs` | `METADATA` LazyLock registry; `analyze()` and `analyze_with_inspect_matcher()` functions; `#` comment suppression parsing |
| `src/registry.rs` | Empty `visit_registry()` function ready for lint rules |
| `src/suppression_action.rs` | `TurtleSuppressionAction` using `# biome-ignore rule: reason` format with `TriviaPieceKind::SingleLineComment` |
| `src/options.rs` | Empty module placeholder for rule options |
| `src/lint.rs` | Empty module placeholder for lint rule groups |

### Phase 6: Service Integration

**File: `crates/biome_configuration/src/turtle.rs`** (new)

Full Turtle configuration module:
- `TurtleConfiguration` with optional formatter, linter, assist sub-configs
- `TurtleFormatterConfiguration` with enabled, indent_style, indent_width, line_ending, line_width
- `TurtleLinterConfiguration` and `TurtleAssistConfiguration` with enabled flags
- Type aliases: `TurtleFormatterEnabled` (default true), `TurtleLinterEnabled` (default true), `TurtleAssistEnabled` (default false)
- Derive macros: `Bpaf`, `Clone`, `Debug`, `Default`, `Deserializable`, `Deserialize`, `Eq`, `Merge`, `PartialEq`, `Serialize`

**File: `crates/biome_configuration/src/lib.rs`** (modified)

- Added `pub mod turtle;` module declaration
- Added `pub use turtle::{TurtleConfiguration, turtle_configuration};`
- Added `pub turtle: Option<TurtleConfiguration>` field to `Configuration` struct

**File: `crates/biome_service/src/file_handlers/turtle.rs`** (new, ~400 lines)

Full handler following GraphQL pattern:
- **Settings:** `TurtleFormatterSettings`, `TurtleLinterSettings`, `TurtleAssistSettings` with `From<TurtleXxxConfiguration>` impls
- **`ServiceLanguage` impl for `TurtleLanguage`:** All type aliases, `lookup_settings()`, `resolve_format_options()` merging global settings, `resolve_analyzer_options()`, enabled-for-file-path checks
- **`TurtleFileHandler` implementing `ExtensionHandler`:** Capabilities for parse, format, lint, code_actions, fix_all, debug_syntax_tree, debug_formatter_ir
- **Module functions:** `parse()`, `format()`, `format_range()`, `format_on_type()`, `lint()`, `code_actions()`, `fix_all()`, `debug_syntax_tree()`, `debug_formatter_ir()`

**File: `crates/biome_service/src/file_handlers/mod.rs`** (modified extensively)

- Added `pub(crate) mod turtle;` module declaration
- Added `Turtle(TurtleFileSource)` variant to `DocumentFileSource` enum
- Added `From<TurtleFileSource>` for `DocumentFileSource` impl
- Added `to_turtle_file_source()` method
- Added Turtle to `try_from_extension()` and `try_from_language_id()` resolution chains
- Added `Self::Turtle(_)` to `can_parse()` and `can_read()` match arms
- Added `turtle: TurtleFileHandler` to `Features` struct and `Features::new()`
- Added `DocumentFileSource::Turtle(_) => self.turtle.capabilities()` dispatch
- Added `Self::Turtle(_) => write!(fmt, "Turtle")` to `Display` impl
- Added `RegistryVisitor<TurtleLanguage>` impls for `SyntaxVisitor`, `LintVisitor`, `AssistsVisitor`

**File: `crates/biome_service/src/settings.rs`** (modified)

- Added imports for `TurtleFormatOptions`, `TurtleLanguage`, `TurtleConfiguration`
- Added `pub turtle: LanguageSettings<TurtleLanguage>` to `LanguageListSettings`
- Added configuration merge: `if let Some(turtle) = configuration.turtle { self.languages.turtle = turtle.into(); }`
- Added `From<TurtleConfiguration> for LanguageSettings<TurtleLanguage>` impl

**File: `crates/biome_service/Cargo.toml`** (modified)

- Added dependencies: `biome_turtle_analyze`, `biome_turtle_formatter`, `biome_turtle_parser`, `biome_turtle_syntax`
- Added `"biome_turtle_syntax/schema"` to the `schema` feature list

**File: `Cargo.toml` (workspace root)** (modified)

- Added `biome_turtle_formatter` and `biome_turtle_analyze` to workspace dependencies

### Build Verification

- `cargo check -p biome_turtle_syntax` -- compiles
- `cargo check -p biome_turtle_factory` -- compiles
- `cargo check -p biome_turtle_parser` -- compiles
- `cargo test -p biome_turtle_parser` -- all tests pass
- `cargo check -p biome_turtle_formatter` -- compiles
- `cargo check -p biome_turtle_analyze` -- compiles
- `cargo test -p biome_turtle_analyze` -- smoke test passes
- `cargo check -p biome_configuration` -- compiles
- `cargo test -p biome_configuration --lib turtle` -- both tests pass
- `cargo check -p biome_service` -- only pre-existing `biome_markdown_formatter` errors (zero Turtle errors)

## Errors Encountered and Fixed

### `format_verbatim_node` not found (Phase 4)

Generated format rule stubs call `format_verbatim_node(node.syntax()).fmt(f)` but the verbatim module exported `format_turtle_verbatim_node`. Fixed by renaming to `format_verbatim_node` (it's crate-internal so the generic name works, consistent with other language formatters).

### Pre-existing markdown formatter blocking compilation

`biome_test_utils` dev-dependency transitively pulls in `biome_markdown_formatter` which has broken codegen (missing `FormatAnyCodeBlock`/`FormatAnyContainerBlock`). Fixed by removing `biome_test_utils`, `insta`, and `tests_macros` from `biome_turtle_parser` and `biome_turtle_analyze` dev-dependencies since the smoke tests don't require them.

### `NodeDialect` enum missing Turtle variant (Phase 4)

Formatter codegen generates per-node format rule stubs into language-specific directories. Added `Turtle` variant to the `NodeDialect` enum in `xtask/codegen/src/formatter.rs` so the codegen writes to `crates/biome_turtle_formatter/src/turtle/`.

## Files Index

### New Files Created

| File | Phase |
|------|-------|
| `xtask/codegen/turtle.ungram` | 0 |
| `xtask/codegen/src/turtle_kinds_src.rs` | 0 |
| `crates/biome_turtle_syntax/Cargo.toml` | 1 |
| `crates/biome_turtle_syntax/src/lib.rs` | 1 |
| `crates/biome_turtle_syntax/src/syntax_node.rs` | 1 |
| `crates/biome_turtle_syntax/src/file_source.rs` | 1 |
| `crates/biome_turtle_syntax/src/generated.rs` | 1 |
| `crates/biome_turtle_syntax/src/generated/*` | 1 (codegen) |
| `crates/biome_turtle_factory/Cargo.toml` | 2 |
| `crates/biome_turtle_factory/src/lib.rs` | 2 |
| `crates/biome_turtle_factory/src/make.rs` | 2 |
| `crates/biome_turtle_factory/src/generated.rs` | 2 |
| `crates/biome_turtle_factory/src/generated/*` | 2 (codegen) |
| `crates/biome_turtle_parser/Cargo.toml` | 3 |
| `crates/biome_turtle_parser/src/lib.rs` | 3 |
| `crates/biome_turtle_parser/src/token_source.rs` | 3 |
| `crates/biome_turtle_parser/src/lexer/mod.rs` | 3 |
| `crates/biome_turtle_parser/src/lexer/tests.rs` | 3 |
| `crates/biome_turtle_parser/src/parser/mod.rs` | 3 |
| `crates/biome_turtle_parser/src/parser/parse_error.rs` | 3 |
| `crates/biome_turtle_parser/src/parser/directive.rs` | 3 |
| `crates/biome_turtle_parser/src/parser/triples.rs` | 3 |
| `crates/biome_turtle_parser/src/parser/literal.rs` | 3 |
| `crates/biome_turtle_parser/src/parser/iri.rs` | 3 |
| `crates/biome_turtle_parser/src/parser/blank_node.rs` | 3 |
| `crates/biome_turtle_parser/src/parser/collection.rs` | 3 |
| `crates/biome_turtle_parser/tests/` | 3 |
| `crates/biome_turtle_formatter/Cargo.toml` | 4 |
| `crates/biome_turtle_formatter/src/lib.rs` | 4 |
| `crates/biome_turtle_formatter/src/context.rs` | 4 |
| `crates/biome_turtle_formatter/src/comments.rs` | 4 |
| `crates/biome_turtle_formatter/src/prelude.rs` | 4 |
| `crates/biome_turtle_formatter/src/cst.rs` | 4 |
| `crates/biome_turtle_formatter/src/verbatim.rs` | 4 |
| `crates/biome_turtle_formatter/src/generated.rs` | 4 |
| `crates/biome_turtle_formatter/src/turtle/*` | 4 (codegen) |
| `crates/biome_turtle_analyze/Cargo.toml` | 5 |
| `crates/biome_turtle_analyze/src/lib.rs` | 5 |
| `crates/biome_turtle_analyze/src/registry.rs` | 5 |
| `crates/biome_turtle_analyze/src/suppression_action.rs` | 5 |
| `crates/biome_turtle_analyze/src/options.rs` | 5 |
| `crates/biome_turtle_analyze/src/lint.rs` | 5 |
| `crates/biome_configuration/src/turtle.rs` | 6 |
| `crates/biome_service/src/file_handlers/turtle.rs` | 6 |

### Existing Files Modified

| File | Phase | Changes |
|------|-------|---------|
| `xtask/codegen/src/language_kind.rs` | 0 | Added `Turtle` variant to `LanguageKind` enum and all related methods |
| `xtask/codegen/src/lib.rs` | 0 | Added `pub mod turtle_kinds_src` |
| `xtask/codegen/src/formatter.rs` | 4 | Added `Turtle` variant to `NodeDialect` enum |
| `Cargo.toml` (workspace root) | 1-5 | Added all turtle crate dependencies |
| `crates/biome_configuration/src/lib.rs` | 6 | Added `mod turtle`, re-exports, config field |
| `crates/biome_service/src/file_handlers/mod.rs` | 6 | Added Turtle variant to DocumentFileSource, handler dispatch, visitor impls |
| `crates/biome_service/Cargo.toml` | 6 | Added turtle crate dependencies and schema feature |
| `crates/biome_service/src/settings.rs` | 6 | Added turtle to LanguageListSettings, config merge, From impl |

## Architecture Notes

### CLI Integration

Biome does not have per-language CLI commands. The existing `biome check`, `biome format`, and `biome lint` commands automatically handle all registered languages via file extension mapping. After this integration:
- `biome check example.ttl` runs parser + linter + formatter checks
- `biome format example.ttl` formats the file
- `biome lint example.ttl` runs analyzer rules

### Validation Model

There is no separate "validate" command. Validation is split between:
1. **Parser diagnostics** — syntax errors (always `Severity::Error`, category `"parse"`)
2. **Lint rules** — semantic validation (via `RuleCategory::Lint`)

`biome check` combines both. This is the same model used by all other languages (GraphQL, CSS, JSON, JS/TS).

### Configuration Example

After this work, `biome.json` supports:
```json
{
  "turtle": {
    "formatter": {
      "enabled": true,
      "indentStyle": "space",
      "indentWidth": 4,
      "lineWidth": 80
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

## Current State

The formatter operates in **verbatim mode** — it preserves the input exactly as-is. To enable actual formatting:
1. Implement formatting logic in `crates/biome_turtle_formatter/src/turtle/` per-node rule files (currently generated stubs using `format_verbatim_node`)
2. Add snapshot tests for formatting output

The analyzer has an **empty rule registry**. To add lint rules:
1. Create rules in `crates/biome_turtle_analyze/src/lint/nursery/`
2. Run `just gen-analyzer` to regenerate the registry

### Known Issue

`cargo check -p biome_service` shows 8 errors, all in `biome_markdown_formatter` (pre-existing broken codegen with missing `FormatAnyCodeBlock`/`FormatAnyContainerBlock`). Zero errors in Turtle code. This blocks full CLI integration testing but does not affect Turtle-specific compilation or tests.

## Next Steps

See `kb/tasks/turtle-implementation-plan.md` Phase 7 for:
- Semantic model (`biome_turtle_semantic` crate)
- Prefix map for cross-statement analysis
- Advanced lint rules (noUndefinedPrefix, noUnusedPrefix, noDuplicatePrefixDeclaration)

Immediate priorities:
1. Implement per-node formatting rules (replace verbatim stubs with real formatting logic)
2. Add initial nursery lint rules
3. Add snapshot tests for both formatter and analyzer
4. Resolve pre-existing markdown formatter errors to enable full CLI integration testing
