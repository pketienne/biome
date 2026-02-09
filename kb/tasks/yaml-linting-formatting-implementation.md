# Implementing YAML Linting and Formatting in Biome

## Current State

Biome already has a **complete YAML parser** and syntax layer:

| Component | Crate | Status |
|---|---|---|
| Parser | `biome_yaml_parser` | Complete (YAML 1.2.2) |
| Syntax | `biome_yaml_syntax` | Generated, complete |
| Factory | `biome_yaml_factory` | Generated, complete |
| Grammar | `xtask/codegen/yaml.ungram` | 315 lines, complete |
| File source | `biome_yaml_syntax/src/file_source.rs` | Detects `.yaml`, `.yml`, `.eyaml`, `.cff`, etc. |

Two minor parser TODOs remain:
- Multiline plain scalar parsing at current indentation level (`lexer/mod.rs`)
- Error handling message improvement (`parser/block.rs`)

## What's Missing

### 1. Formatter crate (`biome_yaml_formatter`)

New crate following the pattern of `biome_css_formatter` or `biome_graphql_formatter`:

```
crates/biome_yaml_formatter/
├── Cargo.toml          # deps: biome_formatter, biome_yaml_syntax, biome_rowan
└── src/
    ├── lib.rs          # format_node entry point
    ├── context.rs      # YamlFormatOptions (indent style, indent width, line width)
    ├── generated.rs    # auto-generated from xtask codegen (~100K lines)
    ├── comments.rs     # comment attachment/formatting
    ├── yaml/           # per-node formatting rules
    └── utils/
```

Run `xtask codegen formatter yaml` to generate the boilerplate formatting implementations.

### 2. Linter crate (`biome_yaml_analyze`)

New crate following the pattern of `biome_css_analyze` or `biome_markdown_analyze`:

```
crates/biome_yaml_analyze/
├── Cargo.toml          # deps: biome_analyze, biome_analyze_macros, biome_yaml_syntax, biome_suppression
└── src/
    ├── lib.rs          # analyze entry point + METADATA
    ├── registry.rs     # rule registry
    ├── lint/
    │   ├── mod.rs
    │   ├── correctness.rs
    │   ├── style.rs
    │   ├── suspicious.rs
    │   ├── nursery.rs
    │   └── nursery/    # individual rule files
    └── assist/
```

### 3. Configuration (`biome_configuration/src/yaml.rs`)

Add a YAML configuration module mirroring the CSS/GraphQL pattern:

- `YamlConfiguration` — root config struct
- `YamlParserConfiguration`
- `YamlFormatterConfiguration` / `YamlFormatterEnabled`
- `YamlLinterConfiguration` / `YamlLinterEnabled`
- `YamlAssistConfiguration` / `YamlAssistEnabled`

Wire into `biome_configuration/src/lib.rs`:
- Declare `mod yaml;` and re-export types
- Add `yaml: Option<YamlConfiguration>` to the `Configuration` struct

### 4. File handler (`biome_service/src/file_handlers/yaml.rs`)

Create a handler implementing the `ServiceLanguage` trait for `YamlLanguage`:

- `YamlFormatterSettings`
- `YamlLinterSettings`
- `YamlAssistSettings`
- `YamlParserSettings`
- Methods: `resolve_parse_options`, `resolve_format_options`, `resolve_analyzer_options`
- Methods: `linter_enabled_for_file_path`, `formatter_enabled_for_file_path`

### 5. Service integration (multiple files)

| File | Change |
|---|---|
| `biome_service/src/file_handlers/mod.rs` | Import yaml handler, add `YamlLanguage` to dispatch logic |
| `biome_service/src/settings.rs` | Add `pub yaml: LanguageSettings<YamlLanguage>` to `LanguageListSettings` |
| `biome_service/src/settings.rs` | Add yaml config merge logic |
| `biome_yaml_syntax/src/file_source.rs` | Already exists but `YamlFileSource` needs adding to `DocumentFileSource` enum |
| `Cargo.toml` (workspace) | Add new crate members |

## Implementation Order

1. **Wire file detection** — add `YamlFileSource` to the `DocumentFileSource` enum so biome recognizes `.yaml`/`.yml` files
2. **Create formatter crate** — scaffold `biome_yaml_formatter` with codegen, implement `context.rs` and per-node formatting
3. **Create linter crate** — scaffold `biome_yaml_analyze` with empty registry
4. **Add configuration** — create `yaml.rs` config types, wire into `Configuration` struct
5. **Create file handler** — implement `yaml.rs` handler with formatter + linter + assist settings
6. **Register in service** — add YAML to `mod.rs` dispatch, `settings.rs` language list
7. **Add lint rules** — implement rules incrementally in nursery, promote as they stabilize

## Candidate YAML Lint Rules

Based on common YAML linting needs (yamllint, etc.):

| Rule | Group | Description |
|---|---|---|
| `noTabIndentation` | Correctness | YAML only allows spaces for indentation |
| `useConsistentIndentation` | Style | Enforce consistent indent width (2 or 4) |
| `noTrailingSpaces` | Style | No trailing whitespace on lines |
| `useFinalNewline` | Style | File should end with a newline |
| `noConsecutiveBlankLines` | Style | Limit consecutive blank lines |
| `noLongLines` | Style | Line length limit |
| `useConsistentQuoteStyle` | Style | Consistent string quoting (single, double, or unquoted) |
| `noDuplicateKeys` | Correctness | Duplicate mapping keys in the same level |
| `noEmptyValues` | Suspicious | Mapping value is empty (likely unfinished) |
| `useConsistentBooleanStyle` | Style | Prefer `true`/`false` over `yes`/`no`/`on`/`off` |
| `noTruthyStrings` | Suspicious | Unquoted strings that YAML 1.1 treated as booleans |
| `useDocumentStartMarker` | Style | Require `---` document start |
| `noDocumentEndMarker` | Style | Forbid `...` document end |
| `useSortedKeys` | Style | Mapping keys in alphabetical order |
| `noAnchorDuplicates` | Correctness | Duplicate anchor names |
| `noUndefinedAliases` | Correctness | Alias references undefined anchor |
| `noFloatKeys` | Suspicious | Mapping keys that resolve to floats |
| `noOctalValues` | Suspicious | Unquoted octal-like values (e.g. `0777`) |

## Reference Crates to Study

- `biome_graphql_formatter` — cleanest recent formatter implementation
- `biome_css_analyze` — good linter crate pattern with multiple rule categories
- `biome_markdown_analyze` — simplest linter (fewest rules, nursery only)
