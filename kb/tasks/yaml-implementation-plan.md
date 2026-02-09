# YAML Formatting and Linting Implementation Plan

## Overview

This plan covers the full implementation of YAML formatting and linting support in Biome. The YAML parser (`biome_yaml_parser`), syntax tree (`biome_yaml_syntax`), and factory (`biome_yaml_factory`) already exist and are complete (YAML 1.2.2). What remains is creating the formatter crate, analyzer crate, configuration types, file handler, and service integration.

The implementation follows established patterns from the Markdown language support (simplest reference) and GraphQL (cleanest formatter reference).

---

## Phase 1: File Detection Wiring

Wire `YamlFileSource` into the `DocumentFileSource` enum so Biome recognizes `.yaml`/`.yml` files.

### 1.1 Add `Yaml` variant to `DocumentFileSource`

**File:** `crates/biome_service/src/file_handlers/mod.rs`

- Add `Yaml(YamlFileSource)` variant to the `DocumentFileSource` enum (line ~90, before `Ignore`)
- Add `From<YamlFileSource> for DocumentFileSource` impl
- Add `YamlFileSource` to `try_from_well_known()` (line ~175, after GraphQL)
- Add `YamlFileSource` to `try_from_extension()` (after Markdown, line ~228)
- Add `YamlFileSource` to `try_from_language_id()` (after Markdown, line ~254)
- Add match arm in `can_parse()` (line ~400): `DocumentFileSource::Yaml(_) => true`
- Add match arm in `can_read()` (line ~415): `DocumentFileSource::Yaml(_) => true`
- Add match arm in `Display` impl (line ~447)
- Add `use biome_yaml_syntax::YamlFileSource;` import

### 1.2 Remove `#[expect(dead_code)]` from `try_from_language_id`

**File:** `crates/biome_yaml_syntax/src/file_source.rs` (line 63)

- Remove the `#[expect(dead_code)]` attribute from `try_from_language_id()` since it will now be called

### Verification

- `cargo check -p biome_service` compiles
- YAML files are recognized by DocumentFileSource

---

## Phase 2: Configuration Types

### 2.1 Create YAML configuration module

**File:** `crates/biome_configuration/src/yaml.rs` (new file)

Follow the Markdown pattern (`crates/biome_configuration/src/markdown.rs`):

```rust
pub struct YamlConfiguration {
    pub formatter: Option<YamlFormatterConfiguration>,
    pub linter: Option<YamlLinterConfiguration>,
    pub assist: Option<YamlAssistConfiguration>,
}

pub type YamlFormatterEnabled = Bool<true>;
pub type YamlLinterEnabled = Bool<true>;
pub type YamlAssistEnabled = Bool<true>;

pub struct YamlFormatterConfiguration {
    pub enabled: Option<YamlFormatterEnabled>,
}

pub struct YamlLinterConfiguration {
    pub enabled: Option<YamlLinterEnabled>,
}

pub struct YamlAssistConfiguration {
    pub enabled: Option<YamlAssistEnabled>,
}
```

Derive macros: `Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Bpaf, Deserializable, Merge`

### 2.2 Wire into configuration crate

**File:** `crates/biome_configuration/src/lib.rs`

- Add `pub mod yaml;`
- Add `yaml: Option<YamlConfiguration>` to the `Configuration` struct (or `PartialConfiguration` -- check exact struct name)

### Verification

- `cargo check -p biome_configuration` compiles

---

## Phase 3: Formatter Crate

### 3.1 Scaffold `biome_yaml_formatter`

**Directory:** `crates/biome_yaml_formatter/` (new crate)

Create with `cargo new crates/biome_yaml_formatter --lib`

**`Cargo.toml`** (following Markdown formatter pattern):
```toml
[package]
name = "biome_yaml_formatter"
version = "0.0.1"
publish = false

[dependencies]
biome_deserialize = { workspace = true }
biome_deserialize_macros = { workspace = true }
biome_diagnostics = { workspace = true }
biome_formatter = { workspace = true }
biome_yaml_syntax = { workspace = true }
biome_rowan = { workspace = true }
biome_suppression = { workspace = true }
camino = { workspace = true }

[dev-dependencies]
biome_formatter = { workspace = true, features = ["countme"] }
biome_yaml_parser = { path = "../biome_yaml_parser" }
biome_parser = { path = "../biome_parser" }
```

### 3.2 Run codegen

```shell
cargo codegen formatter yaml
# or: cargo xtask codegen formatter yaml
```

This generates:
- `src/generated.rs` - Format trait implementations for all YAML syntax nodes
- `src/cst.rs` - CST formatting dispatch

### 3.3 Implement core files

**`src/context.rs`** - Format context (follow `biome_markdown_formatter/src/context.rs`):
- `YamlFormatContext` struct
- `YamlFormatOptions` with: indent_style, indent_width, line_ending, line_width
- Builder pattern with `with_*` methods
- Implement `FormatContext` and `CstFormatContext` traits

**`src/lib.rs`** - Entry point (follow `biome_markdown_formatter/src/lib.rs`):
- `AsFormat<Context>` and `IntoFormat<Context>` trait impls
- `FormatNodeRule<N>` and `FormatBogusNodeRule<N>` trait impls
- `FormatYamlSyntaxToken` token formatting rule
- `FormatLanguage` impl for `YamlFormatLanguage`
- Public API: `format_node()`, `format_sub_tree()`

**`src/comments.rs`** - Comment handling:
- `YamlCommentStyle` implementing `CommentStyle`
- YAML uses `#` for comments (always line comments)
- Suppression comment parsing integration

**`src/prelude.rs`** - Common imports

**`src/trivia.rs`** - Whitespace/trivia handling

**`src/verbatim.rs`** - Verbatim node formatting (for nodes with no custom formatting)

**`src/yaml/`** - Per-node formatting rules organized by category:
- `auxiliary/` - Mapping pairs, scalar values, tags, anchors
- `any/` - Enum variant dispatchers (AnyYamlValue, etc.)
- `bogus/` - Error recovery nodes
- `lists/` - List container formatting

### 3.4 YAML-specific formatting decisions

YAML formatting has unique considerations:
- **Indentation is semantic** - unlike most languages, YAML indentation determines structure
- **Block vs flow style** - mappings/sequences can be block (`key: value`) or flow (`{key: value}`)
- **Scalar styles** - plain, single-quoted, double-quoted, literal block (`|`), folded block (`>`)
- **Comment placement** - trailing comments, standalone comments between nodes
- Start conservatively: preserve existing style, normalize whitespace only

### 3.5 Register in workspace

**File:** `Cargo.toml` (root workspace)
- Add `biome_yaml_formatter` to workspace members
- Add `biome_yaml_formatter = { path = "./crates/biome_yaml_formatter", version = "0.0.1" }` to `[workspace.dependencies]`

### Verification

- `cargo check -p biome_yaml_formatter` compiles
- Basic smoke test: parse YAML, format, verify output

---

## Phase 4: Analyzer Crate

### 4.1 Scaffold `biome_yaml_analyze`

**Directory:** `crates/biome_yaml_analyze/` (new crate)

**`Cargo.toml`** (following Markdown analyzer pattern):
```toml
[package]
name = "biome_yaml_analyze"
version = "0.0.1"
publish = false

[dependencies]
biome_analyze = { workspace = true }
biome_analyze_macros = { workspace = true }
biome_yaml_syntax = { workspace = true }
biome_rowan = { workspace = true }
biome_suppression = { workspace = true }

[dev-dependencies]
biome_markdown_parser = { path = "../biome_yaml_parser" }
biome_test_utils = { path = "../biome_test_utils" }
insta = { workspace = true, features = ["glob"] }
tests_macros = { path = "../tests_macros" }
```

### 4.2 Implement core files

**`src/lib.rs`** (follow `biome_markdown_analyze/src/lib.rs`):
- `METADATA` LazyLock for rule registry
- `analyze()` and `analyze_with_inspect_matcher()` functions
- Suppression comment parsing (`biome-ignore` in YAML `#` comments)
- No services parameter needed (no semantic model)

**`src/registry.rs`** - Generated registry (run codegen after adding rules)

**`src/suppression_action.rs`** - `YamlSuppressionAction`

**`src/options.rs`** - Rule options re-exports

**`src/lint.rs`** / **`src/lint/mod.rs`** - Lint rules module

### 4.3 Initial lint rules (nursery)

Start with 3-4 high-value rules in `src/lint/nursery/`:

| Rule | File | Description |
|---|---|---|
| `noDuplicateKeys` | `no_duplicate_keys.rs` | Duplicate mapping keys at the same level |
| `noTabIndentation` | `no_tab_indentation.rs` | YAML only allows spaces for indentation |
| `useConsistentBooleanStyle` | `use_consistent_boolean_style.rs` | Prefer `true`/`false` over `yes`/`no`/`on`/`off` |

Each rule follows the `declare_lint_rule!` macro pattern.

### 4.4 Run codegen

```shell
just gen-analyzer
```

This generates `src/registry.rs` with the rule registrations.

### 4.5 Register in workspace

**File:** `Cargo.toml` (root workspace)
- Add `biome_yaml_analyze` to workspace members and dependencies

### Verification

- `cargo check -p biome_yaml_analyze` compiles
- `cargo t -p biome_yaml_analyze` passes
- Snapshot tests for each rule

---

## Phase 5: File Handler

### 5.1 Create YAML file handler

**File:** `crates/biome_service/src/file_handlers/yaml.rs` (new file)

Follow `crates/biome_service/src/file_handlers/markdown.rs` exactly:

**Settings structs:**
```rust
pub struct YamlFormatterSettings {
    pub enabled: Option<YamlFormatterEnabled>,
}
pub struct YamlLinterSettings {
    pub enabled: Option<YamlLinterEnabled>,
}
pub struct YamlAssistSettings {
    pub enabled: Option<YamlAssistEnabled>,
}
```

Each with `From<YamlXxxConfiguration>` impl.

**`ServiceLanguage` impl for `YamlLanguage`:**
- `type FormatterSettings = YamlFormatterSettings`
- `type LinterSettings = YamlLinterSettings`
- `type AssistSettings = YamlAssistSettings`
- `type FormatOptions = YamlFormatOptions`
- `type ParserSettings = ()`
- `type ParserOptions = ()`
- `type EnvironmentSettings = ()`
- `lookup_settings()` -> `&languages.yaml`
- `resolve_format_options()` - merge global settings into `YamlFormatOptions`
- `resolve_analyzer_options()` - standard analyzer config
- `formatter_enabled_for_file_path()` / `linter_enabled_for_file_path()` / `assist_enabled_for_file_path()` - using `check_feature_activity()`
- `resolve_environment()` -> `None`
- `resolve_parse_options()` -> `()`

**`YamlFileHandler` struct implementing `ExtensionHandler`:**
- `capabilities()` returns `Capabilities` with:
  - `parser: parse` (uses `biome_yaml_parser::parse_yaml_with_cache`)
  - `formatter: format` (uses `biome_yaml_formatter::format_node`)
  - `analyzer: lint, code_actions, fix_all`
  - `debug: debug_syntax_tree`
  - `search: None` (initially)

**Module-level functions:** `parse()`, `debug_syntax_tree()`, `format()`, `lint()`, `code_actions()`, `fix_all()`, `formatter_enabled()`, `linter_enabled()`, `assist_enabled()`, `search_enabled()`

---

## Phase 6: Service Integration

### 6.1 Register handler in mod.rs

**File:** `crates/biome_service/src/file_handlers/mod.rs`

- Add `pub(crate) mod yaml;` module declaration
- Add `yaml: YamlFileHandler` field to `Features` struct (line ~1060)
- Add match arm `DocumentFileSource::Yaml(_) => self.yaml.capabilities()` in `get_capabilities()` (line ~1090)

### 6.2 Register in settings.rs

**File:** `crates/biome_service/src/settings.rs`

- Add `use biome_configuration::yaml::YamlConfiguration;`
- Add `use biome_yaml_syntax::YamlLanguage;`
- Add `pub yaml: LanguageSettings<YamlLanguage>` to `LanguageListSettings` (line ~511)
- Add yaml config merge block (after markdown, line ~182):
  ```rust
  if let Some(yaml) = configuration.yaml {
      self.languages.yaml = yaml.into();
  }
  ```
- Add `From<YamlConfiguration> for LanguageSettings<YamlLanguage>` impl (after Markdown impl, line ~668)

### 6.3 Register analyzer visitors

**File:** `crates/biome_service/src/file_handlers/mod.rs`

- Add `impl RegistryVisitor<YamlLanguage> for SyntaxVisitor` (empty, returns immediately)
- Add `impl RegistryVisitor<YamlLanguage> for LintVisitor` (calls `biome_yaml_analyze::visit_registry`)
- Add `impl RegistryVisitor<YamlLanguage> for AssistsVisitor` (empty initially)
- Add `biome_yaml_analyze::visit_registry(&mut syntax_visitor)` call in `AnalyzerVisitorBuilder.finish()`

### 6.4 Update Cargo.toml dependencies

**File:** `crates/biome_service/Cargo.toml`

- Add `biome_yaml_analyze`, `biome_yaml_formatter`, `biome_yaml_parser`, `biome_yaml_syntax` as dependencies

### Verification

- `cargo check -p biome_service` compiles
- `cargo biome-cli-dev format test.yaml` works
- `cargo biome-cli-dev lint test.yaml` works
- `cargo biome-cli-dev check test.yaml` works

---

## Phase 7: Testing

### 7.1 Formatter tests

**Directory:** `crates/biome_yaml_formatter/tests/`

- Snapshot tests using `insta` for common YAML patterns:
  - Simple key-value mappings
  - Nested mappings
  - Sequences (block and flow)
  - Multi-line scalars (literal `|`, folded `>`)
  - Comments (inline, standalone)
  - Mixed block/flow styles
  - Anchors and aliases
  - Tags
  - Multi-document files (`---`/`...`)

### 7.2 Analyzer tests

**Directory:** `crates/biome_yaml_analyze/tests/`

- Per-rule snapshot tests under `specs/nursery/`
- Valid and invalid cases for each rule
- Suppression comment tests

### 7.3 CLI integration tests

**Directory:** `crates/biome_cli/tests/`

- Add YAML-specific test cases to existing CLI test infrastructure
- Test `biome format`, `biome lint`, `biome check` with YAML files
- Test `biome.json` configuration with `yaml` section

---

## Phase 8: Future Enhancements

These are not part of the initial implementation but should be tracked:

### 8.1 Additional lint rules

| Rule | Group | Priority |
|---|---|---|
| `noTrailingSpaces` | Style | High |
| `useFinalNewline` | Style | High |
| `noConsecutiveBlankLines` | Style | Medium |
| `useConsistentQuoteStyle` | Style | Medium |
| `noEmptyValues` | Suspicious | Medium |
| `noTruthyStrings` | Suspicious | Medium |
| `useConsistentIndentation` | Style | Medium |
| `useDocumentStartMarker` | Style | Low |
| `noDocumentEndMarker` | Style | Low |
| `useSortedKeys` | Style | Low |
| `noAnchorDuplicates` | Correctness | Low |
| `noUndefinedAliases` | Correctness | Low |
| `noFloatKeys` | Suspicious | Low |
| `noOctalValues` | Suspicious | Low |

### 8.2 Parser improvements

- Multiline plain scalar parsing at current indentation level
- Error handling message improvements

### 8.3 Advanced formatter features

- Format range support
- Format on type support
- Style option: block vs flow for short collections
- Configurable quote style preference

### 8.4 Override settings

- Add `to_yaml_language_settings()` function in override handling
- Support per-path YAML configuration overrides

---

## File Index

Summary of all files created or modified:

### New files
| File | Phase |
|---|---|
| `crates/biome_configuration/src/yaml.rs` | 2 |
| `crates/biome_yaml_formatter/Cargo.toml` | 3 |
| `crates/biome_yaml_formatter/src/lib.rs` | 3 |
| `crates/biome_yaml_formatter/src/context.rs` | 3 |
| `crates/biome_yaml_formatter/src/comments.rs` | 3 |
| `crates/biome_yaml_formatter/src/prelude.rs` | 3 |
| `crates/biome_yaml_formatter/src/trivia.rs` | 3 |
| `crates/biome_yaml_formatter/src/verbatim.rs` | 3 |
| `crates/biome_yaml_formatter/src/cst.rs` | 3 (generated) |
| `crates/biome_yaml_formatter/src/generated.rs` | 3 (generated) |
| `crates/biome_yaml_formatter/src/yaml/**` | 3 (per-node rules) |
| `crates/biome_yaml_analyze/Cargo.toml` | 4 |
| `crates/biome_yaml_analyze/src/lib.rs` | 4 |
| `crates/biome_yaml_analyze/src/registry.rs` | 4 (generated) |
| `crates/biome_yaml_analyze/src/suppression_action.rs` | 4 |
| `crates/biome_yaml_analyze/src/options.rs` | 4 |
| `crates/biome_yaml_analyze/src/lint/**` | 4 (rules) |
| `crates/biome_service/src/file_handlers/yaml.rs` | 5 |

### Modified files
| File | Phase | Changes |
|---|---|---|
| `crates/biome_yaml_syntax/src/file_source.rs` | 1 | Remove `#[expect(dead_code)]` |
| `crates/biome_service/src/file_handlers/mod.rs` | 1, 6 | Add Yaml variant, handler, visitors |
| `crates/biome_configuration/src/lib.rs` | 2 | Add `mod yaml`, config field |
| `Cargo.toml` (workspace root) | 3, 4 | Add new crate members/deps |
| `crates/biome_service/Cargo.toml` | 6 | Add yaml crate deps |
| `crates/biome_service/src/settings.rs` | 6 | Add yaml to LanguageListSettings |

---

## Reference Crates

| Purpose | Crate | Why |
|---|---|---|
| Simplest formatter | `biome_markdown_formatter` | Closest match for initial YAML impl |
| Cleanest formatter | `biome_graphql_formatter` | Most polished formatter pattern |
| Simplest analyzer | `biome_markdown_analyze` | Minimal rule set, no semantic model |
| Good analyzer pattern | `biome_css_analyze` | Multiple rule categories |
| File handler template | `biome_service/src/file_handlers/markdown.rs` | Direct template to copy |
| Config template | `biome_configuration/src/markdown.rs` | Direct template to copy |
