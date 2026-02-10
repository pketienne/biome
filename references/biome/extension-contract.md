# Biome Language Integration Contract

## Purpose and Audience

This document describes the contract a new language must satisfy to be fully integrated into Biome. It is the primary reference for:

- **Architecture analysts** assessing how much of a language integration exists and what gaps remain.
- **Spec writers** translating feature requirements into concrete implementation plans.
- **Implementers** building parser, formatter, analyzer, or service integration crates.

The contract is organized as a 7-layer stack. Each layer has a defined crate boundary, key traits to implement, and explicit inputs/outputs. Layers build on each other bottom-up: grammar before syntax, syntax before parser, parser before formatter/analyzer, and all of these before service integration.

All file paths are relative to the Biome repository root.

## 7-Layer Overview

| Layer | Name | Crate Pattern | Key Trait / Type | Depends On |
|-------|------|---------------|-----------------|------------|
| 1 | Grammar & Code Generation | `xtask/codegen/` | `KindsSrc`, `LanguageKind` | None |
| 2 | Syntax Crate | `biome_{lang}_syntax` | `Language`, `SyntaxKind`, `FileSource` | Layer 1 (generated) |
| 3 | Factory Crate | `biome_{lang}_factory` | `SyntaxFactory` | Layer 2 |
| 4 | Parser Crate | `biome_{lang}_parser` | `Parser`, `AnyParse` | Layers 2, 3 |
| 5 | Formatter Crate | `biome_{lang}_formatter` | `FormatLanguage`, `FormatContext`, `FormatOptions` | Layers 2, 4 |
| 6 | Analyzer Crate | `biome_{lang}_analyze` | `Rule`, `RuleGroup`, `declare_lint_rule!` | Layers 2, 4 |
| 7 | Service Integration | `biome_service` | `ServiceLanguage`, `ExtensionHandler`, `DocumentFileSource` | All above |

## Layer 1: Grammar & Code Generation

Defines the concrete syntax tree (CST) structure and generates Rust types from it.

### Key files

- **Grammar definition**: `xtask/codegen/{lang}.ungram` — Ungrammar file defining all node types, their children, and token slots. Example: `xtask/codegen/yaml.ungram`.
- **Kinds source**: `xtask/codegen/src/{lang}_kinds_src.rs` — Defines a `KindsSrc` constant listing all punctuation, keywords, literals, tokens, and nodes. Example: `xtask/codegen/src/yaml_kinds_src.rs`.
- **KindsSrc struct**: `xtask/codegen/src/kind_src.rs:1-15` — The struct that organizes syntax kinds into categories:
  ```rust
  pub struct KindsSrc<'a> {
      pub punct: &'a [(&'a str, &'a str)],
      pub keywords: &'a [&'a str],
      pub literals: &'a [&'a str],
      pub tokens: &'a [&'a str],
      pub nodes: &'a [&'a str],
  }
  ```
- **LanguageKind enum**: `xtask/codegen/src/language_kind.rs:31-41` — Registers the language in codegen infrastructure. Must include the new language variant.

### What to implement

1. Write a `.ungram` file defining the CST grammar.
2. Write a `{lang}_kinds_src.rs` file populating `KindsSrc` with all token and node kinds.
3. Add the language to the `LanguageKind` enum.
4. Run codegen (`just gen-bindings`) to produce generated types in the syntax crate.

## Layer 2: Syntax Crate

Provides the typed syntax tree, language definition, file source detection, and trivia classification.

### Key files

- **Crate entry**: `crates/biome_{lang}_syntax/src/lib.rs` — Exports generated types, implements `SyntaxKind` trait for the language's kind enum, maps trivia tokens (whitespace, comments, newlines) to `TriviaPieceKind`.
- **Language type**: `crates/biome_{lang}_syntax/src/syntax_node.rs` — Defines the zero-sized `{Lang}Language` struct and implements the `Language` trait. Also defines type aliases: `{Lang}SyntaxNode`, `{Lang}SyntaxToken`, `{Lang}SyntaxElement`, etc.
- **File source**: `crates/biome_{lang}_syntax/src/file_source.rs` — `{Lang}FileSource` struct with `try_from_extension()`, `try_from_file_path()`, and well-known file detection.
- **Generated types**: `crates/biome_{lang}_syntax/src/generated/` — Auto-generated `kind.rs` (SyntaxKind enum), `nodes.rs` (typed AST node structs), and `macros.rs`.

### Trait contracts

**`Language`** (`crates/biome_rowan/src/syntax.rs:61-64`):
```rust
pub trait Language: Sized + Clone + Copy + fmt::Debug + Eq + Ord + std::hash::Hash {
    type Kind: SyntaxKind;
    type Root: AstNode<Language = Self> + Clone + Eq + fmt::Debug;
}
```

**`SyntaxKind`** (`crates/biome_rowan/src/syntax.rs`) — Must implement `to_string()`, `from_raw()`, `into_raw()`, `is_trivia()`, and trivia classification methods.

### What to implement

1. Create the crate with `Cargo.toml` depending on `biome_rowan`.
2. Implement `Language` for a zero-sized type (e.g., `YamlLanguage`).
3. Implement `SyntaxKind` with trivia mapping (whitespace, comments, newlines).
4. Create `FileSource` with extension matching and well-known file detection.
5. Export generated types from codegen.

## Layer 3: Factory Crate

Provides the syntax tree builder used by the parser to construct green nodes with slot validation.

### Key files

- **Crate entry**: `crates/biome_{lang}_factory/src/lib.rs` — Exports `{Lang}SyntaxFactory` and `{Lang}SyntaxTreeBuilder` type alias.
- **Generated factory**: `crates/biome_{lang}_factory/src/generated/syntax_factory.rs` — Implements `SyntaxFactory` with a large `match` over all node kinds, validating expected child slots.
- **Make utilities**: `crates/biome_{lang}_factory/src/make.rs` — Re-exports generated constructors from `generated/node_factory.rs`.

### Trait contracts

**`SyntaxFactory`** (`crates/biome_rowan/src/syntax_factory.rs:17-32`):
```rust
pub trait SyntaxFactory: fmt::Debug {
    type Kind: SyntaxKind;
    fn make_syntax(
        kind: Self::Kind,
        children: ParsedChildren<Self::Kind>,
    ) -> RawSyntaxNode<Self::Kind>;
}
```

### What to implement

This layer is almost entirely auto-generated from codegen. After Layer 1 and Layer 2 are in place, running `just gen-bindings` produces the factory. The `make.rs` file is typically a one-line re-export.

## Layer 4: Parser Crate

Implements lexing, parsing, and tree construction. Produces a typed parse result convertible to `AnyParse`.

### Key files

- **Crate entry**: `crates/biome_{lang}_parser/src/lib.rs` — Public `parse_{lang}()` function, `{Lang}Parse` struct with `syntax()`, `into_diagnostics()`, and `From<{Lang}Parse> for AnyParse`.
- **Parser struct**: `crates/biome_{lang}_parser/src/parser/mod.rs` — Implements the `Parser` trait from `biome_parser`.
- **Lexer**: `crates/biome_{lang}_parser/src/lexer/mod.rs` — Token source producing `SyntaxKind` tokens with trivia attached.
- **Tree sink**: `crates/biome_{lang}_parser/src/` — Lossless tree sink converting parser events into green tree via the factory.

### Key types

**`AnyParse`** (`crates/biome_parser/src/lib.rs:704-713`) — The language-erased parse result used by the service layer:
```rust
pub enum AnyParse {
    Node(NodeParse),
    EmbeddedNode(EmbeddedNodeParse),
}
```

**`Parse` function signature** (from service layer `mod.rs:499`):
```rust
type Parse = fn(&BiomePath, DocumentFileSource, &str, &Settings, &mut NodeCache) -> ParseResult;
```

### What to implement

1. Build a lexer that tokenizes the language and classifies trivia.
2. Build a recursive-descent parser that produces parser events.
3. Wire up a tree sink that converts events to a green tree via the factory.
4. Expose a public `parse_{lang}()` → `{Lang}Parse` function.
5. Implement `From<{Lang}Parse> for AnyParse`.

## Layer 5: Formatter Crate

Formats source code by walking the CST and emitting an intermediate representation (IR) that a printer renders to text.

### Key files

- **Crate entry**: `crates/biome_{lang}_formatter/src/lib.rs` — Implements `FormatLanguage` for a `{Lang}FormatLanguage` struct.
- **Context**: `crates/biome_{lang}_formatter/src/context.rs` — `{Lang}FormatContext` (implements `CstFormatContext`) and `{Lang}FormatOptions` (implements `FormatOptions`).
- **Root format rule**: `crates/biome_{lang}_formatter/src/cst.rs` — `Format{Lang}SyntaxNode` implementing `FormatRule<SyntaxNode<{Lang}Language>>`.
- **Comment handling**: `crates/biome_{lang}_formatter/src/comments.rs` — `{Lang}CommentStyle` implementing `CommentStyle`.
- **Generated rules**: `crates/biome_{lang}_formatter/src/generated/` — Per-node format rules.

### Trait contracts

**`FormatLanguage`** (`crates/biome_formatter/src/lib.rs:1496-1515`):
```rust
pub trait FormatLanguage {
    type SyntaxLanguage: Language;
    type Context: CstFormatContext<Language = Self::SyntaxLanguage>;
    type FormatRule: FormatRule<SyntaxNode<Self::SyntaxLanguage>, Context = Self::Context> + Default;

    fn transform(&self, root: &SyntaxNode<Self::SyntaxLanguage>)
        -> Option<(SyntaxNode<Self::SyntaxLanguage>, TransformSourceMap)>;
    fn is_range_formatting_node(&self, node: &SyntaxNode<Self::SyntaxLanguage>) -> bool;
    fn options(&self) -> &<Self::Context as FormatContext>::Options;
    fn create_context(self, root: &SyntaxNode<Self::SyntaxLanguage>,
        source_map: Option<TransformSourceMap>, delegate: bool) -> Self::Context;
}
```

**`FormatOptions`** (`crates/biome_formatter/src/lib.rs:853-868`):
```rust
pub trait FormatOptions {
    fn indent_style(&self) -> IndentStyle;
    fn indent_width(&self) -> IndentWidth;
    fn line_width(&self) -> LineWidth;
    fn line_ending(&self) -> LineEnding;
    fn as_print_options(&self) -> PrinterOptions;
}
```

**`CstFormatContext`** (`crates/biome_formatter/src/lib.rs:874-883`):
```rust
pub trait CstFormatContext: FormatContext {
    type Language: Language;
    type Style: CommentStyle<Language = Self::Language>;
    type CommentRule: FormatRule<SourceComment<Self::Language>, Context = Self> + Default;
    fn comments(&self) -> &Comments<Self::Language>;
}
```

### Reference implementation

`crates/biome_json_formatter/` — The simplest complete formatter. Key files:
- `src/lib.rs:246-279` — `JsonFormatLanguage` implementing `FormatLanguage`
- `src/context.rs:16-73` — `JsonFormatContext` and `JsonFormatOptions`

### What to implement

1. Create `{Lang}FormatOptions` implementing `FormatOptions` (indent style/width, line width, line ending, plus any language-specific options).
2. Create `{Lang}FormatContext` implementing `CstFormatContext` with comment handling.
3. Create `{Lang}FormatLanguage` implementing `FormatLanguage`.
4. Create `CommentStyle` implementation for the language's comment syntax.
5. Implement `FormatRule` for each node type (partially auto-generated).

## Layer 6: Analyzer Crate

Hosts lint rules, code actions, and the analysis infrastructure for a language.

### Key files

- **Crate entry**: `crates/biome_{lang}_analyze/src/lib.rs` — `analyze()` function, `{Lang}AnalyzeServices` struct, suppression comment parsing, rule registry setup.
- **Lint rules**: `crates/biome_{lang}_analyze/src/lint/` — Rules organized by category subdirectories (e.g., `suspicious/`, `correctness/`, `style/`).
- **Assist actions**: `crates/biome_{lang}_analyze/src/assist/` — Code action implementations.

### Trait contracts

**`Rule`** (`crates/biome_analyze/src/rule.rs:1116-1138`):
```rust
pub trait Rule: RuleMeta + Sized {
    type Query: Queryable;
    type State;
    type Signals: IntoIterator<Item = Self::State>;
    type Options: Default + Clone + Debug;

    fn phase() -> Phases;
    fn run(ctx: &RuleContext<Self>) -> Self::Signals;
    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic>;
    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<Self::Query>>;
}
```

**`declare_lint_rule!`** macro — Generates `RuleMeta` impl with metadata (version, name, language, recommended, severity) and registers the rule in a group.

### Reference implementation

`crates/biome_json_analyze/` — Contains a small set of JSON-specific rules. Key files:
- `src/lib.rs` — `analyze()` function pattern, `JsonAnalyzeServices`, suppression parsing
- `src/lint/suspicious/no_duplicate_object_keys.rs` — Example lint rule using `declare_lint_rule!`

### What to implement

1. Create the crate with `analyze()` entry point.
2. Implement suppression comment parsing (e.g., `# biome-ignore` for YAML).
3. Create a `SuppressionAction` implementation.
4. Register rule groups and categories.
5. Implement individual lint rules using `declare_lint_rule!` and the `Rule` trait.
6. Run `just gen-analyzer` to generate registry code.

## Layer 7: Service Integration

Wires the language into Biome's workspace service — file detection, configuration, and capability dispatch.

### Key files

- **File handlers module**: `crates/biome_service/src/file_handlers/mod.rs` — Contains `DocumentFileSource` (line 78), `Capabilities` (line 480), `ExtensionHandler` (line 1014), and `Features` (line 1022).
- **Language handler**: `crates/biome_service/src/file_handlers/{lang}.rs` — Implements `ExtensionHandler` for a `{Lang}FileHandler` struct, plus standalone functions for each capability (parse, format, lint, code_actions, etc.).
- **Settings**: `crates/biome_service/src/settings.rs` — `ServiceLanguage` trait (line 648).

### Trait contracts

**`ExtensionHandler`** (`file_handlers/mod.rs:1014-1019`):
```rust
pub(crate) trait ExtensionHandler {
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
}
```

**`Capabilities`** (`file_handlers/mod.rs:480-487`):
```rust
pub struct Capabilities {
    pub(crate) parser: ParserCapabilities,
    pub(crate) debug: DebugCapabilities,
    pub(crate) analyzer: AnalyzerCapabilities,
    pub(crate) formatter: FormatterCapabilities,
    pub(crate) search: SearchCapabilities,
    pub(crate) enabled_for_path: EnabledForPath,
}
```

**`ServiceLanguage`** (`settings.rs:648-679`):
```rust
pub trait ServiceLanguage: biome_rowan::Language {
    type FormatterSettings: Default;
    type LinterSettings: Default;
    type AssistSettings: Default;
    type FormatOptions: biome_formatter::FormatOptions + Clone + std::fmt::Display + Default;
    type ParserSettings: Default;
    type ParserOptions: Default;
    type EnvironmentSettings: Default;

    fn lookup_settings(languages: &LanguageListSettings) -> &LanguageSettings<Self>;
    fn resolve_environment(settings: &Settings) -> Option<&Self::EnvironmentSettings>;
    fn resolve_parse_options(...) -> Self::ParserOptions;
    fn resolve_format_options(...) -> Self::FormatOptions;
}
```

### Integration checklist

1. Add `{Lang}({Lang}FileSource)` variant to `DocumentFileSource` enum.
2. Implement `From<{Lang}FileSource> for DocumentFileSource`.
3. Wire up `DocumentFileSource::try_from_extension()` and related methods for the new variant.
4. Create `{Lang}FileHandler` struct implementing `ExtensionHandler`.
5. Implement `ServiceLanguage` for `{Lang}Language`.
6. Add handler field to `Features` struct and wire up `capabilities()` dispatch.
7. Add language settings to `LanguageListSettings`.

### Reference implementation

`crates/biome_service/src/file_handlers/json.rs` — Complete end-to-end handler. Implements:
- `ServiceLanguage for JsonLanguage` with all settings types
- `ExtensionHandler for JsonFileHandler` returning full `Capabilities`
- Standalone capability functions: `parse`, `format`, `format_range`, `format_on_type`, `lint`, `code_actions`, `fix_all`, `debug_syntax_tree`, `debug_formatter_ir`

## Integration Status: YAML

| Layer | Crate | Status | Notes |
|-------|-------|--------|-------|
| 1 | `xtask/codegen/` | **Complete** | `yaml.ungram` (315 lines), `yaml_kinds_src.rs` (96 lines), `Yaml` in `LanguageKind` |
| 2 | `biome_yaml_syntax` | **Complete** | `YamlLanguage`, `YamlSyntaxKind`, `YamlFileSource` with 14+ extensions |
| 3 | `biome_yaml_factory` | **Complete** | `YamlSyntaxFactory` with full slot validation, `make.rs` constructors |
| 4 | `biome_yaml_parser` | **Complete** | `parse_yaml()`, lexer with indentation-sensitive scoping, `AnyParse` conversion |
| 5 | `biome_yaml_formatter` | **Not started** | Crate does not exist |
| 6 | `biome_yaml_analyze` | **Not started** | Crate does not exist |
| 7 | `biome_service` (YAML) | **Not started** | No `Yaml` variant in `DocumentFileSource`, no `yaml.rs` handler, no `YamlFileHandler` in `Features` |

### Existing YAML crates

- `crates/biome_yaml_syntax/` — Full syntax types with trivia mapping
- `crates/biome_yaml_factory/` — Generated factory with slot validation
- `crates/biome_yaml_parser/` — Working parser with indentation-aware lexer
- `xtask/codegen/yaml.ungram` — Complete grammar definition
- `xtask/codegen/src/yaml_kinds_src.rs` — All syntax kinds defined

### Missing YAML crates

- `crates/biome_yaml_formatter/` — Needs `FormatLanguage`, `FormatContext`, `FormatOptions`, comment style, per-node rules
- `crates/biome_yaml_analyze/` — Needs `Rule` implementations, suppression parsing, registry
- `crates/biome_service/src/file_handlers/yaml.rs` — Needs `ExtensionHandler`, `ServiceLanguage`, capability functions

## Dependency Graph

```
Layer 1: Grammar & Codegen
    │
    ▼
Layer 2: Syntax Crate ◄──────────────────────────┐
    │                                              │
    ▼                                              │
Layer 3: Factory Crate                             │
    │                                              │
    ▼                                              │
Layer 4: Parser Crate ─────────────┐               │
    │                              │               │
    ├──────────────┐               │               │
    ▼              ▼               ▼               ▼
Layer 5:       Layer 6:        Layer 7: Service Integration
Formatter      Analyzer        (requires all of the above)
```

Layers 5 and 6 are independent of each other and can be built in parallel. Layer 7 requires all preceding layers.

**Recommended build order for YAML** (layers 1-4 already complete):
1. Layer 5 (Formatter) and Layer 6 (Analyzer) — in parallel
2. Layer 7 (Service Integration) — after both 5 and 6

## Reference Implementations

**JSON** is the recommended reference for new language integrations. It is the simplest end-to-end implementation: no embedded languages, no semantic model, minimal formatting options, straightforward lint rules.

### JSON reference file paths

| Layer | Key File |
|-------|----------|
| 1 | `xtask/codegen/json.ungram`, `xtask/codegen/src/json_kinds_src.rs` |
| 2 | `crates/biome_json_syntax/src/lib.rs`, `crates/biome_json_syntax/src/file_source.rs` |
| 3 | `crates/biome_json_factory/src/lib.rs` |
| 4 | `crates/biome_json_parser/src/lib.rs` |
| 5 | `crates/biome_json_formatter/src/lib.rs`, `crates/biome_json_formatter/src/context.rs` |
| 6 | `crates/biome_json_analyze/src/lib.rs` |
| 7 | `crates/biome_service/src/file_handlers/json.rs` |

**Why not CSS or JS?** CSS has embedded language handling and more complex selector formatting. JS has semantic analysis, module graphs, type inference, and embedded JSX — complexity YAML doesn't need. JSON's scope matches YAML's scope: data serialization format with comments and basic structure.
