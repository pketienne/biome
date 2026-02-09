# YAML Service Integration - Implementation Complete

## Overview

Implemented full YAML formatting and linting service integration into Biome. The existing YAML parser (`biome_yaml_parser`), syntax tree (`biome_yaml_syntax`), and factory (`biome_yaml_factory`) were already complete (YAML 1.2.2 spec). This work created the formatter crate, analyzer crate, configuration types, file handler, and wired everything into the service layer.

After this work, Biome recognizes `.yaml`/`.yml` files and routes them through the full formatting, linting, code actions, and fix-all pipelines. The formatter currently operates in verbatim mode (preserves input) and the analyzer has an empty rule registry, both ready for incremental feature additions.

## What Was Done

### Phase 1: File Detection Wiring

**File: `crates/biome_service/src/file_handlers/mod.rs`**

- Added `Yaml(YamlFileSource)` variant to the `DocumentFileSource` enum
- Added `From<YamlFileSource> for DocumentFileSource` impl
- Added YAML dispatch to `try_from_well_known()`, `try_from_extension()`, `try_from_language_id()`
- Added `to_yaml_file_source()` method
- Added YAML match arms to `can_parse()`, `can_read()`, `can_contain_embeds()`, `Display`
- Import: `use biome_yaml_syntax::file_source::YamlFileSource;`

**File: `crates/biome_yaml_syntax/src/lib.rs`**

- Changed `mod file_source;` to `pub mod file_source;` so `YamlFileSource` is accessible externally

### Phase 2: Configuration Types

**File: `crates/biome_configuration/src/yaml.rs`** (new)

Created the full YAML configuration module following the Markdown pattern:

- `YamlConfiguration` with optional formatter, linter, and assist sub-configs
- `YamlFormatterConfiguration` / `YamlLinterConfiguration` / `YamlAssistConfiguration` structs
- `YamlFormatterEnabled` / `YamlLinterEnabled` / `YamlAssistEnabled` type aliases (all default `true`)
- Derive macros: `Bpaf`, `Clone`, `Debug`, `Default`, `Deserializable`, `Deserialize`, `Eq`, `Merge`, `PartialEq`, `Serialize`
- Schema support via `schemars::JsonSchema`

**File: `crates/biome_configuration/src/lib.rs`**

- Added `pub mod yaml;` module declaration
- Added `pub use yaml::{YamlConfiguration, yaml_configuration};`
- Added `pub yaml: Option<YamlConfiguration>` field to the `Configuration` struct

### Phase 3: Formatter Crate

**Directory: `crates/biome_yaml_formatter/`** (new crate)

Created following the `biome_markdown_formatter` and `biome_graphql_formatter` patterns:

| File | Purpose |
|------|---------|
| `Cargo.toml` | Crate manifest with biome_yaml_syntax, biome_formatter, biome_rowan, biome_suppression deps |
| `src/lib.rs` | Entry point: `AsFormat`, `IntoFormat`, `FormatNodeRule`, `FormatBogusNodeRule` traits; `YamlFormatLanguage` implementing `FormatLanguage`; `FormatYamlSyntaxToken`; public `format_node()`, `format_range()`, `format_sub_tree()` |
| `src/context.rs` | `YamlFormatContext` and `YamlFormatOptions` (indent_style, indent_width, line_ending, line_width) with builder pattern; `FormatContext` and `CstFormatContext` trait impls |
| `src/comments.rs` | `YamlCommentStyle` implementing `CommentStyle` with `CommentKind::Line` for `#` comments; `FormatYamlLeadingComment`; suppression comment detection |
| `src/prelude.rs` | Common re-exports |
| `src/cst.rs` | Temporary `FormatYamlSyntaxNode` that formats all nodes as verbatim (until codegen generates per-node rules) |
| `src/verbatim.rs` | `format_verbatim_node()`, `format_bogus_node()`, `format_suppressed_node()` for nodes with no custom formatting |

### Phase 4: Analyzer Crate

**Directory: `crates/biome_yaml_analyze/`** (new crate)

Created following the `biome_markdown_analyze` pattern:

| File | Purpose |
|------|---------|
| `Cargo.toml` | Crate manifest with biome_analyze, biome_yaml_syntax, biome_rowan, biome_suppression deps |
| `src/lib.rs` | `METADATA` LazyLock registry; `analyze()` and `analyze_with_inspect_matcher()` functions; YAML `#` comment suppression parsing; `quick_test` |
| `src/registry.rs` | Empty `visit_registry()` function ready for rules |
| `src/suppression_action.rs` | `YamlSuppressionAction` using `# biome-ignore rule: reason` YAML comments with `TriviaPieceKind::SingleLineComment` |
| `src/options.rs` | Empty module placeholder for rule options |
| `src/lint.rs` | Empty module placeholder for lint rule groups |

### Phase 5: File Handler

**File: `crates/biome_service/src/file_handlers/yaml.rs`** (new)

Full handler following `markdown.rs`:

- **Settings:** `YamlFormatterSettings`, `YamlLinterSettings`, `YamlAssistSettings` with `From<YamlXxxConfiguration>` impls
- **`ServiceLanguage` impl for `YamlLanguage`:** All type aliases, `lookup_settings()` -> `&languages.yaml`, `resolve_format_options()` merging global settings, `resolve_analyzer_options()`, enabled-for-file-path checks using `check_feature_activity()`
- **`YamlFileHandler` implementing `ExtensionHandler`:** Capabilities for parse, format, lint, code_actions, fix_all, debug_syntax_tree, search_enabled
- **Module functions:** `parse()` (using `parse_yaml_with_cache`), `format()` (using `format_node`), `lint()`, `code_actions()`, `fix_all()` with full `AnalyzerVisitorBuilder` and `ProcessLint`/`ProcessFixAll` integration

### Phase 6: Service Integration

**File: `crates/biome_service/src/file_handlers/mod.rs`**

- Added `pub(crate) mod yaml;` module declaration
- Added `yaml: YamlFileHandler` field to `Features` struct
- Added `DocumentFileSource::Yaml(_) => self.yaml.capabilities()` dispatch
- Added `RegistryVisitor<YamlLanguage>` impls for `SyntaxVisitor`, `LintVisitor`, `AssistsVisitor`
- Added `biome_yaml_analyze::visit_registry` calls in `finish()` for all three visitor types
- Import: `use biome_yaml_analyze::METADATA as yaml_metadata;`

**File: `crates/biome_service/src/settings.rs`**

- Added `use biome_configuration::yaml::YamlConfiguration;` and `use biome_yaml_syntax::YamlLanguage;`
- Added `pub yaml: LanguageSettings<YamlLanguage>` to `LanguageListSettings`
- Added yaml config merge block: `if let Some(yaml) = configuration.yaml { self.languages.yaml = yaml.into(); }`
- Added `From<YamlConfiguration> for LanguageSettings<YamlLanguage>` impl

**File: `crates/biome_service/Cargo.toml`**

- Added dependencies: `biome_yaml_analyze`, `biome_yaml_formatter`, `biome_yaml_parser`, `biome_yaml_syntax`
- Added `biome_yaml_syntax/schema` to the `schema` feature list

**File: `Cargo.toml` (workspace root)**

- Added `biome_yaml_formatter` and `biome_yaml_analyze` to workspace dependencies

### Phase 7: Build Verification

- `cargo check -p biome_yaml_formatter` -- compiles (warnings only)
- `cargo check -p biome_yaml_analyze` -- compiles (warnings only)
- `cargo check -p biome_service` -- compiles (zero errors)
- `cargo check -p biome_cli` -- compiles (zero errors, full workspace)

## Files Index

### New Files Created

| File | Phase |
|------|-------|
| `crates/biome_configuration/src/yaml.rs` | 2 |
| `crates/biome_yaml_formatter/Cargo.toml` | 3 |
| `crates/biome_yaml_formatter/src/lib.rs` | 3 |
| `crates/biome_yaml_formatter/src/context.rs` | 3 |
| `crates/biome_yaml_formatter/src/comments.rs` | 3 |
| `crates/biome_yaml_formatter/src/prelude.rs` | 3 |
| `crates/biome_yaml_formatter/src/cst.rs` | 3 |
| `crates/biome_yaml_formatter/src/verbatim.rs` | 3 |
| `crates/biome_yaml_analyze/Cargo.toml` | 4 |
| `crates/biome_yaml_analyze/src/lib.rs` | 4 |
| `crates/biome_yaml_analyze/src/registry.rs` | 4 |
| `crates/biome_yaml_analyze/src/suppression_action.rs` | 4 |
| `crates/biome_yaml_analyze/src/options.rs` | 4 |
| `crates/biome_yaml_analyze/src/lint.rs` | 4 |
| `crates/biome_service/src/file_handlers/yaml.rs` | 5 |

### Existing Files Modified

| File | Phase | Changes |
|------|-------|---------|
| `crates/biome_yaml_syntax/src/lib.rs` | 1 | Made `file_source` module public |
| `crates/biome_service/src/file_handlers/mod.rs` | 1, 6 | Added Yaml variant to DocumentFileSource, handler dispatch, visitor impls |
| `crates/biome_configuration/src/lib.rs` | 2 | Added `mod yaml`, re-exports, config field |
| `Cargo.toml` (workspace root) | 3, 4 | Added biome_yaml_formatter and biome_yaml_analyze to workspace deps |
| `crates/biome_service/Cargo.toml` | 6 | Added yaml crate dependencies and schema feature |
| `crates/biome_service/src/settings.rs` | 6 | Added yaml to LanguageListSettings, config merge, From impl |

## Current State

The formatter operates in **verbatim mode** -- it preserves the input exactly as-is. To enable actual formatting:
1. Run `cargo codegen formatter yaml` (or equivalent xtask) to generate per-node format rules
2. Implement formatting logic in `crates/biome_yaml_formatter/src/yaml/` per-node rule files

The analyzer has an **empty rule registry**. To add lint rules:
1. Create rules in `crates/biome_yaml_analyze/src/lint/nursery/`
2. Run `just gen-analyzer` to regenerate the registry

## Next Steps

See `kb/tasks/yaml-implementation-plan.md` Phases 7-8 for:
- Formatter codegen and per-node formatting rules
- Initial lint rules (noDuplicateKeys, noTabIndentation, useConsistentBooleanStyle)
- Snapshot testing for formatter and analyzer
- CLI integration tests
- Override settings support
