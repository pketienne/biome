# Biome Layer-by-Layer Integration Checklist

For each layer, what must exist, which traits to implement, and where to find the reference implementation.

## Layer 1: Grammar

**Purpose:** Define the language's concrete syntax tree structure.

**Files to create:**
- `xtask/codegen/src/{lang}.ungram` — Grammar definition
- `crates/biome_{lang}_syntax/src/generated/` — Generated syntax types (via codegen)

**Key trait:** `Language` (in `biome_rowan`)
- `type Kind: SyntaxKind`
- `type Root: AstNode`

**Reference:** `xtask/codegen/src/json.ungram`, `crates/biome_json_syntax/`

**Verification:** `cargo run -p xtask_codegen -- {lang}` succeeds and generates syntax types.

---

## Layer 2: Syntax Factory

**Purpose:** Provide a factory for constructing syntax nodes during parsing.

**Files to create:**
- `crates/biome_{lang}_factory/src/generated/` — Generated factory (via codegen)
- `crates/biome_{lang}_factory/src/lib.rs` — Re-exports

**Key trait:** `SyntaxFactory`
- `fn make_syntax(kind, children) -> RawSyntaxNode`
- `fn make_token(kind, text) -> RawSyntaxToken`

**Reference:** `crates/biome_json_factory/`

**Verification:** Factory compiles and is importable from the parser crate.

---

## Layer 3: Parser

**Purpose:** Parse source text into a CST using the grammar and factory.

**Files to create:**
- `crates/biome_{lang}_parser/src/` — Parser implementation
- `crates/biome_{lang}_parser/src/syntax/` — Per-node parsing functions
- `crates/biome_{lang}_parser/src/lexer/` — Tokenizer

**Key trait:** `Parser` / entry point function
- `pub fn parse_{lang}(source: &str) -> Parse<{Lang}Root>`

**Reference:** `crates/biome_json_parser/`

**Verification:** `cargo test -p biome_{lang}_parser` passes. Parse a sample file and verify the CST structure.

---

## Layer 4: Formatter

**Purpose:** Format CST nodes into Biome's IR for pretty-printing.

**Files to create:**
- `crates/biome_{lang}_formatter/src/lib.rs` — Crate root with `format_node`
- `crates/biome_{lang}_formatter/src/context.rs` — `{Lang}FormatContext`
- `crates/biome_{lang}_formatter/src/{lang}_module.rs` — Format options module
- `crates/biome_{lang}_formatter/src/generated.rs` — Generated `FormatNode` impls (via codegen)
- Per-node format files under `src/` matching syntax node types

**Key traits:**
- `FormatLanguage` — `type Context`, `type FormatRule`, options accessors
- `FormatNodeRule<N>` — `fn fmt(node, f)` per syntax node type

**Codegen:** `cargo run -p xtask_codegen -- formatter`

**IR primitives used (common):**
- `block_indent()` — indented block with hard line breaks
- `soft_block_indent()` — indented block that can flatten
- `hard_line_break()` — mandatory newline
- `soft_line_break()` — newline that can become space in flat mode
- `space()`, `text()`, `format_verbatim_node()` (for unhandled nodes)

**Reference:** `crates/biome_json_formatter/`

**Verification:**
1. `cargo test -p biome_{lang}_formatter` passes
2. `cargo run -p xtask_codegen -- formatter` succeeds
3. Format a sample file and verify output structure

---

## Layer 5: Analyzer

**Purpose:** Implement lint rules that analyze the CST.

**Files to create:**
- `crates/biome_{lang}_analyze/src/lib.rs` — Crate root with `visit_registry`
- `crates/biome_{lang}_analyze/src/lint/` — Rule implementations by group
- `crates/biome_{lang}_analyze/src/lint/suspicious/` — (example group directory)

**Key macros and traits:**
- `declare_lint_rule!` — Declares the rule with metadata
- `Rule` trait — `type Query`, `type State`, `type Signals`, `fn run(ctx)`
- `declare_lint_group!` — Groups rules together

**Rule metadata fields:**
- `version: "next"` (updated to actual version at release)
- `name: "ruleName"` (camelCase)
- `language: "{lang}"`
- `recommended: true/false`
- `source: RuleSource::*` (if adapting from another tool)

**Codegen:** `cargo run -p xtask_codegen -- analyzer`

**Reference:** `crates/biome_json_analyze/`

**Verification:**
1. `cargo test -p biome_{lang}_analyze` passes
2. Run `biome lint sample.{ext}` and verify rule fires (catches registration system bugs)

---

## Layer 6: Configuration

**Purpose:** Wire formatter options and analyzer rules into Biome's unified configuration.

**Files to modify:**
- `crates/biome_configuration/src/` — Add `{lang}.rs` module
- `crates/biome_configuration/src/generated/` — Updated by codegen

**Key types:**
- `{Lang}Configuration` — Top-level config struct
- `{Lang}Formatter` — Formatter options (mirrors `FormatOptions`)
- `{Lang}Linter` — Linter rules (generated from analyzer groups)

**Codegen:** `cargo run -p xtask_codegen -- configuration`

**Snapshot updates:** Adding a new language config key causes existing configuration snapshot tests to fail. Run `cargo insta accept` to update them. This is expected.

**Reference:** `crates/biome_configuration/src/json.rs` (if it exists) or the configuration codegen output.

**Verification:** `cargo test -p biome_configuration` passes after snapshot updates.

---

## Layer 7: Service Integration

**Purpose:** Register the language with Biome's service layer so CLI commands work.

**Files to modify:**
- `crates/biome_service/src/file_handlers/{lang}.rs` — Handler implementation
- `crates/biome_service/src/file_handlers/mod.rs` — Register handler
- `crates/biome_service/src/workspace/server.rs` — Wire capabilities

**Key trait:** `ExtensionHandler`
- `fn capabilities() -> Capabilities` — declares format/lint/assist support
- `fn parse(...)`, `fn format(...)`, `fn lint(...)` — delegates to language crates

**Key type:** `DocumentFileSource` — maps file extensions to the language

**Capabilities to wire:**
- `Formatter` — if Layer 4 exists
- `Analyzer` — if Layer 5 exists
- `Search` (optional) — pattern matching support

**Reference:** `crates/biome_service/src/file_handlers/json.rs`

**Verification:**
1. `biome format sample.{ext}` works
2. `biome lint sample.{ext}` works
3. `biome check sample.{ext}` works
4. File detection: Biome recognizes `.{ext}` files without `--files-include`
