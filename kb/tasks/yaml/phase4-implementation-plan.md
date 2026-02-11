# Phase 4: YAML Implementation Plan

## Context

Phases 1-3 produced the spec (`references/yaml/yaml-support-spec.md`). Phase 4 implements it. YAML has Layers 1-4 complete (grammar, syntax, factory, parser). We need to build Layer 5 (formatter), Layer 6 (analyzer), and Layer 7 (service integration) to make `biome check *.yaml` work.

The codegen infrastructure already knows about YAML (`LanguageKind::Yaml` in `xtask/codegen/src/formatter.rs`). The workspace has `biome_yaml_syntax`, `biome_yaml_factory`, `biome_yaml_parser` but no formatter, analyzer, or service wiring.

## Approach

5 stages, each independently verifiable. Stages 1-2 can partially overlap. Stage 3 depends on both. Stage 4 glues everything together.

---

## Stage 1: `biome_yaml_formatter` crate

**Goal:** Compilable formatter crate with basic node formatting.

### Files to create

| File | Purpose | Reference |
|------|---------|-----------|
| `crates/biome_yaml_formatter/Cargo.toml` | Manifest | `crates/biome_json_formatter/Cargo.toml` |
| `src/lib.rs` | `YamlFormatLanguage`, `FormatNodeRule`, `FormatBogusNodeRule`, public API (`format_node`, `format_range`, `format_sub_tree`) | `crates/biome_json_formatter/src/lib.rs` |
| `src/context.rs` | `YamlFormatContext` (impl `CstFormatContext`), `YamlFormatOptions` (impl `FormatOptions`) — 4 base options only | `crates/biome_json_formatter/src/context.rs` |
| `src/comments.rs` | `YamlCommentStyle` (impl `CommentStyle`) — `#` line comments only, no block comments | `crates/biome_json_formatter/src/comments.rs` |
| `src/cst.rs` | `FormatYamlSyntaxNode` root dispatch via `map_syntax_node!` | `crates/biome_json_formatter/src/cst.rs` |
| `src/prelude.rs` | Common re-exports | `crates/biome_json_formatter/src/prelude.rs` |
| `src/verbatim.rs` | Verbatim formatting for bogus/suppressed nodes | `crates/biome_json_formatter/src/verbatim.rs` |
| `src/separated.rs` | Separated list formatting (flow commas) | `crates/biome_json_formatter/src/separated.rs` |
| `src/generated.rs` | Placeholder — codegen fills this | — |

### Run codegen

```bash
just gen-formatter   # cargo run -p xtask_codegen -- formatter
```

Generates `src/generated.rs` (FormatRule/AsFormat/IntoFormat for all ~51 node types) and stub formatters in `src/yaml/` subdirectories.

### Implement core node formatters (replace stubs)

Priority order — enough for basic YAML formatting:

1. `YamlRoot` — BOM + document list + EOF + hard_line_break
2. `YamlDocument` — directives + `---` + body + `...`
3. `YamlBlockMapping` — block_map_entry_list
4. `YamlBlockMapImplicitEntry` — key + `:` + space + value
5. `YamlBlockSequence` — block_sequence_entry_list
6. `YamlBlockSequenceEntry` — `-` + space + content
7. `YamlPlainScalar` / `YamlSingleQuotedScalar` / `YamlDoubleQuotedScalar` — token verbatim
8. `YamlLiteralScalar` / `YamlFoldedScalar` — indicator line + verbatim content
9. `YamlFlowMapping` — `{` + group(soft_block_indent(entries)) + `}`
10. `YamlFlowSequence` — `[` + group(soft_block_indent(entries)) + `]`

**Initial formatter strategy:** Preserve indentation structure verbatim for block-style nodes (YAML indentation is semantic). Use IR-based formatting for flow-style nodes. Normalize trailing whitespace and line endings. Ensure final newline.

### Also modify

- `Cargo.toml` (root): add `biome_yaml_formatter = { path = "./crates/biome_yaml_formatter", version = "0.0.1" }` to workspace dependencies

### Verify

```bash
cargo build -p biome_yaml_formatter
cargo test -p biome_yaml_formatter
```

---

## Stage 2: `biome_yaml_analyze` crate

**Goal:** Compilable analyzer crate with one lint rule (`noDuplicateKeys`) and suppression support.

### Files to create

| File | Purpose | Reference |
|------|---------|-----------|
| `crates/biome_yaml_analyze/Cargo.toml` | Manifest | `crates/biome_json_analyze/Cargo.toml` |
| `src/lib.rs` | `analyze()`, `METADATA`, `YamlAnalyzeServices`, suppression parsing | `crates/biome_json_analyze/src/lib.rs` |
| `src/suppression_action.rs` | `YamlSuppressionAction` — `# biome-ignore` format | `crates/biome_json_analyze/src/suppression_action.rs` |
| `src/lint.rs` | `declare_category! { Lint { suspicious } }` | `crates/biome_json_analyze/src/lint.rs` |
| `src/lint/suspicious.rs` | `declare_group_from_fs!` | `crates/biome_json_analyze/src/lint/suspicious.rs` |
| `src/lint/suspicious/no_duplicate_keys.rs` | First rule — detect duplicate mapping keys | `crates/biome_json_analyze/src/lint/suspicious/no_duplicate_object_keys.rs` |
| `src/registry.rs` | `visit_registry()` | `crates/biome_json_analyze/src/registry.rs` |
| `build.rs` | Watch rule directories | `crates/biome_json_analyze/build.rs` |

### `noDuplicateKeys` rule structure

```rust
declare_lint_rule! {
    pub NoDuplicateKeys {
        version: "next",
        name: "noDuplicateKeys",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
    }
}
impl Rule for NoDuplicateKeys {
    type Query = Ast<YamlBlockMapping>;
    type State = (/* first key node */, Vec<TextRange> /* duplicate ranges */);
    type Signals = Box<[Self::State]>;
    type Options = ();
    // Walk block_map_entry_list, collect key text via plain_scalar, report duplicates
}
```

### Also modify

- `Cargo.toml` (root): add `biome_yaml_analyze = { path = "./crates/biome_yaml_analyze", version = "0.0.1" }`
- `crates/biome_diagnostics_categories/src/categories.rs`: add `"lint/suspicious/noDuplicateKeys"` entry

### Run codegen

```bash
just gen-rules       # cargo run -p xtask_codegen -- analyzer
```

Regenerates `registry.rs`, `lint.rs`, `build.rs`.

### Verify

```bash
cargo build -p biome_yaml_analyze
cargo test -p biome_yaml_analyze
```

---

## Stage 3: Configuration module

**Goal:** YAML configuration types for `biome.json`.

### Files to create

| File | Purpose | Reference |
|------|---------|-----------|
| `crates/biome_configuration/src/yaml.rs` | `YamlConfiguration`, `YamlFormatterConfiguration`, `YamlLinterConfiguration`, `YamlParserConfiguration` | `crates/biome_configuration/src/json.rs` |

### Files to modify

| File | Change |
|------|--------|
| `crates/biome_configuration/src/lib.rs` | Add `pub mod yaml;`, add `yaml: Option<YamlConfiguration>` to `Configuration` struct |
| `crates/biome_configuration/src/overrides.rs` | Add YAML override fields following JSON/CSS pattern |
| `crates/biome_configuration/Cargo.toml` | Add `biome_yaml_formatter` dependency (if YAML-specific option types live there) |

### Verify

```bash
cargo build -p biome_configuration
cargo test -p biome_configuration
```

---

## Stage 4: Service integration

**Goal:** Wire YAML into `biome_service` so `biome format/lint/check *.yaml` works.

### Create new file

**`crates/biome_service/src/file_handlers/yaml.rs`** (~400 lines, modeled on `json.rs`):

- `YamlFormatterSettings`, `YamlLinterSettings`, `YamlParserSettings` structs
- `impl ServiceLanguage for YamlLanguage` (7 type aliases + 9 methods)
- `YamlFileHandler` struct implementing `ExtensionHandler`
- Capability functions: `parse`, `format`, `format_range`, `lint`, `code_actions`, `fix_all`, `debug_syntax_tree`, `debug_formatter_ir`

### Modify `crates/biome_service/src/file_handlers/mod.rs` (~15 edit locations)

1. Add `pub(crate) mod yaml;` to module declarations
2. Add `Yaml(YamlFileSource)` variant to `DocumentFileSource` enum (~line 78)
3. Add `From<YamlFileSource> for DocumentFileSource` impl
4. Wire `try_from_well_known()`, `try_from_extension()`, `try_from_language_id()`, `try_from_path()`
5. Add `is_yaml_like()`, `to_yaml_file_source()` methods
6. Add `Yaml(_)` to `can_parse()`, `can_read()`, `can_contain_embeds()` matches
7. Add `Yaml(_)` to `Display` and `biome_console::fmt::Display` impls
8. Add `yaml: YamlFileHandler` field to `Features` struct
9. Add `Yaml(_)` to `get_capabilities()` match
10. Add `RegistryVisitor<YamlLanguage>` impls for `SyntaxVisitor`, `LintVisitor`, `AssistsVisitor`
11. Add `biome_yaml_analyze::visit_registry()` calls in `AnalyzerVisitorBuilder::build()` (3 locations)

### Modify `crates/biome_service/src/settings.rs`

1. Add `pub yaml: LanguageSettings<YamlLanguage>` to `LanguageListSettings`
2. Add YAML config merging in `Settings::apply_configuration()`
3. Add `From<YamlConfiguration> for LanguageSettings<YamlLanguage>`

### Modify `crates/biome_service/Cargo.toml`

Add dependencies: `biome_yaml_analyze`, `biome_yaml_formatter`, `biome_yaml_parser`, `biome_yaml_syntax`

### Verify

```bash
cargo build -p biome_service
cargo test -p biome_service
# End-to-end:
cargo biome-cli-dev format test.yaml
cargo biome-cli-dev lint test.yaml
cargo biome-cli-dev check test.yaml
```

---

## Stage 5: Tests and polish

### Formatter tests

- `crates/biome_yaml_formatter/tests/spec_tests.rs` — test harness with `gen_tests!`
- `crates/biome_yaml_formatter/tests/specs/yaml/` — fixtures: simple_mapping, nested, sequences, flow_style, scalars, multi_document, comments

### Analyzer tests

- `crates/biome_yaml_analyze/tests/spec_tests.rs` — test harness
- `crates/biome_yaml_analyze/tests/specs/suspicious/noDuplicateKeys/` — valid.yaml, invalid.yaml + snapshots

### Verify

```bash
cargo test -p biome_yaml_formatter
cargo test -p biome_yaml_analyze
cargo test -p biome_service
```

---

## Critical files (modification targets)

| File | Type | Stage |
|------|------|-------|
| `crates/biome_yaml_formatter/src/lib.rs` | Create | 1 |
| `crates/biome_yaml_formatter/src/context.rs` | Create | 1 |
| `crates/biome_yaml_formatter/src/comments.rs` | Create | 1 |
| `crates/biome_yaml_analyze/src/lib.rs` | Create | 2 |
| `crates/biome_yaml_analyze/src/lint/suspicious/no_duplicate_keys.rs` | Create | 2 |
| `crates/biome_configuration/src/yaml.rs` | Create | 3 |
| `crates/biome_configuration/src/lib.rs` | Modify | 3 |
| `crates/biome_service/src/file_handlers/yaml.rs` | Create | 4 |
| `crates/biome_service/src/file_handlers/mod.rs` | Modify (15 locations) | 4 |
| `crates/biome_service/src/settings.rs` | Modify | 4 |
| `crates/biome_diagnostics_categories/src/categories.rs` | Modify | 2 |
| `Cargo.toml` (root) | Modify | 1-2 |
