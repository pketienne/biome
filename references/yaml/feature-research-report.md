# YAML Tooling Feature Research Report

> **Date:** 2026-02-10
> **Scope:** All 13 repos — 3 linters, 3 formatters, 3 parsers, 3 validators, 1 language server
> **Extraction depth:** Deep (config options, defaults, error messages, implementation patterns)
> **Priority areas:** Indentation & formatting rules, key ordering & duplicates

---

## Executive Summary

- **Total features found:** 90+ distinct features/rules across all tools
- **Consensus features (in 2+ tools):** 28 linting rules, 15 formatting features, 5 parser capabilities
- **Unique features (in 1 tool only):** 30+ tool-specific features
- **Key patterns:** Token-based linting (yamllint family), IR-based formatting (prettier), event-based parsing (all parsers), JSON Schema validation (kubeconform, action-validator, vscode-yaml)
- **Critical gap:** No existing parser preserves comments or whitespace — Biome must build a CST from scratch

---

## 1. Feature Matrix: Linters

### 1.1 Rule Coverage Across Tools

| Rule | yamllint (Py) | yaml-lint-rs (Rust) | yamllint-rs (Rust) | Category |
|------|:---:|:---:|:---:|----------|
| `indentation` | Y | Y | Y | **Indentation** |
| `key-duplicates` | Y | Y | Y | **Keys** |
| `key-ordering` | Y | - | Y | **Keys** |
| `braces` | Y | - | Y | Spacing |
| `brackets` | Y | - | Y | Spacing |
| `colons` | Y | Y | Y | Spacing |
| `commas` | Y | - | Y | Spacing |
| `comments` | Y | Y | Y | Comments |
| `comments-indentation` | Y | - | Y | Comments |
| `document-start` | Y | Y | Y | Structure |
| `document-end` | Y | - | Y | Structure |
| `empty-lines` | Y | Y | Y | Formatting |
| `empty-values` | Y | - | Y | Values |
| `float-values` | Y | - | Y | Values |
| `hyphens` | Y | Y | Y | Spacing |
| `line-length` | Y | Y | Y | Formatting |
| `new-line-at-end-of-file` | Y | Y | Y | Formatting |
| `new-lines` | Y | - | Y | Formatting |
| `octal-values` | Y | - | Y | Values |
| `quoted-strings` | Y | - | Y | Style |
| `trailing-spaces` | Y | Y | Y | Formatting |
| `truthy` | Y | Y | Y | Values |
| `anchors` | Y | - | Y | Structure |
| **Total** | **22** | **11** | **22** | |

### 1.2 Priority Area: `indentation` Rule (Deep)

| Option | Type | yamllint Default | yaml-lint-rs | yamllint-rs |
|--------|------|:---:|:---:|:---:|
| `spaces` | int / `"consistent"` | `consistent` | `Consistent` | `2` |
| `indent-sequences` | bool / `"whatever"` / `"consistent"` | `true` | N/A | `true` (bool only) |
| `check-multi-line-strings` | bool | `false` | N/A | `false` |

**Implementation:** yamllint (Py) and yamllint-rs use a token-level parent stack (Root, BlockMap, BlockSeq, BlockEnt, Key, Val) to compute expected indentation. yaml-lint-rs uses simpler line-level checking.

**Gap:** Python yamllint supports `indent-sequences: whatever|consistent` (more flexible); Rust ports only support bool.

### 1.3 Priority Area: `key-duplicates` Rule (Deep)

| Option | Type | yamllint Default | yaml-lint-rs | yamllint-rs |
|--------|------|:---:|:---:|:---:|
| `forbid-duplicated-merge-keys` | bool | `false` | N/A | `false` |

**Implementation:** All three use key-tracking within mapping contexts. yamllint and yamllint-rs special-case `<<` merge keys.

### 1.4 Priority Area: `key-ordering` Rule (Deep)

| Option | Type | yamllint Default | yamllint-rs |
|--------|------|:---:|:---:|
| `ignored-keys` | list of regex | `[]` | N/A |

**yaml-lint-rs does NOT implement this rule.** yamllint uses locale-aware comparison (`locale.strcoll`); yamllint-rs uses simple string comparison.

### 1.5 Infrastructure Comparison

| Feature | yamllint (Py) | yaml-lint-rs | yamllint-rs |
|---------|:---:|:---:|:---:|
| Rule analysis level | Token (pyyaml) | Line (text) | Token (yaml-rust) + line |
| Severity levels | error, warning | Error, Warning, Disable | Error, Warning, Info |
| Config format | YAML `.yamllint` | YAML `.yamllint` | YAML `.yamllint` |
| Built-in presets | `default`, `relaxed` | `default`, `relaxed` | Default config only |
| Inline disable directives | `# yamllint disable` | None | Yes |
| Auto-fix support | None | 2 rules | 7 rules |
| Parallel processing | No | No | Yes (rayon) |

---

## 2. Feature Matrix: Formatters

### 2.1 Indentation & Formatting (Priority Area)

| Feature | yamlfmt (Go) | prettier (JS) | yamlfix (Py) |
|---------|:---:|:---:|:---:|
| Indent size | `indent: 2` | `tabWidth: 2` | `indent_mapping: 2` |
| Separate sequence indent | `array_indent` | N/A | `indent_sequence: 4` |
| Indentless arrays | `indentless_arrays: false` | N/A | N/A |
| Indent root array | `indent_root_array: false` | N/A | N/A |
| Max line length | `max_line_length: 0` (off) | `printWidth: 80` | `line_length: 80` |
| Use tabs | N/A | `useTabs: false` | N/A |
| Line ending style | `line_ending: OS` | `endOfLine: lf` | N/A (always `\n`) |
| Retain blank lines | `retain_line_breaks: false` | Preserves 1 | N/A |
| Trim trailing whitespace | `trim_trailing_whitespace: false` | Always | N/A |
| EOF newline | `eof_newline: false` | Always | Always |

### 2.2 Key Ordering (Priority Area)

**No formatter supports key sorting/ordering.** This is a notable gap. All three preserve input key order.

### 2.3 Quoting

| Feature | yamlfmt | prettier | yamlfix |
|---------|:---:|:---:|:---:|
| Force quote style | `force_quote_style` (single/double) | `singleQuote: false` | `quote_representation: '` |
| Quote unquoted values | N/A | N/A | `quote_basic_values: false` |
| Smart quote selection | N/A | Yes (minimize escaping) | N/A |

### 2.4 Architecture

| Property | yamlfmt | prettier | yamlfix |
|----------|---------|---------|---------|
| **Approach** | Decode → Transform AST → Encode | AST → IR (doc builders) → Render | Text preprocess → ruyaml round-trip → Text postprocess |
| **Comment preservation** | Via yaml.v3 node attachment | 5 distinct comment types in AST | Via ruyaml round-trip mode |
| **Line breaking** | Encoder-controlled | IR-driven (group/ifBreak/fill) | Line-length check → force block |
| **Plugin system** | Formatter Registry | Prettier plugin arch | None |

### 2.5 Unique Features by Tool

**yamlfmt only:** `retain_line_breaks`, `disallow_anchors`, `drop_merge_tag`, `scan_folded_as_literal`, `regex_exclude`, `#!yamlfmt!ignore`

**prettier only:** IR-based formatting, `proseWrap`, `bracketSpacing`, `trailingComma`, `# prettier-ignore` node-level, `embeddedLanguageFormatting`, smart quote selection

**yamlfix only:** `none_representation`, `quote_basic_values`, `comments_require_starting_space`, `comments_whitelines`, truthy normalization, Jinja2/Ansible vault handling

---

## 3. Feature Matrix: Parsers

### 3.1 Architecture Comparison

| Feature | yaml-rust2 | saphyr | serde-yaml |
|---------|:---:|:---:|:---:|
| YAML spec | 1.2 | 1.2 | 1.1 (libyaml) |
| Parser type | Event-based | Event-based + spans | Event-based (libyaml FFI) |
| Memory model | Owned (`String`) | Borrowed (`Cow<'input, str>`) | Mixed |
| no_std | No | **Yes** | No |
| Zero-copy | No | **Yes** | Partial |
| Comment preservation | **No** | **No** | **No** |
| Span tracking | Point only (`Marker`) | **Full spans** (`Span` start+end) | Point only (`Mark`) |
| Error recovery | None | None | None |
| Maintenance | Active | Active | **UNMAINTAINED** |

### 3.2 Data Models

| Node Type | yaml-rust2 | saphyr | serde-yaml |
|-----------|-----------|--------|-----------|
| Null | `Yaml::Null` | `Scalar::Null` | `Value::Null` |
| Boolean | `Yaml::Boolean(bool)` | `Scalar::Boolean(bool)` | `Value::Bool(bool)` |
| Integer | `Yaml::Integer(i64)` | `Scalar::Integer(i64)` | `Number::PosInt(u64)` / `NegInt(i64)` |
| Float | `Yaml::Real(String)` | `Scalar::FloatingPoint(OrderedFloat<f64>)` | `Number::Float(f64)` |
| String | `Yaml::String(String)` | `Scalar::String(Cow<str>)` | `Value::String(String)` |
| Sequence | `Vec<Yaml>` | `Vec<Yaml>` | `Vec<Value>` |
| Mapping | `LinkedHashMap<Yaml,Yaml>` | `LinkedHashMap<Yaml,Yaml>` | `IndexMap<Value,Value>` |
| Tagged | N/A | `Yaml::Tagged(Tag, Box<Yaml>)` | `Value::Tagged(Box<TaggedValue>)` |
| Bad/Sentinel | `Yaml::BadValue` | `YamlData::BadValue` | N/A (serde errors) |
| Annotated | N/A | **`MarkedYaml` with `Span`** | N/A |
| Lazy/Raw | N/A | **`YamlData::Representation`** | N/A |

### 3.3 Critical Gaps for Biome (All Parsers Share)

1. **No CST / lossless parsing** — all produce ASTs that discard formatting
2. **No comment model** — comments universally discarded
3. **No whitespace/trivia preservation** — needed for formatter
4. **No error recovery** — all stop on first error
5. **No incremental parsing** — no re-parsing of changed regions

### 3.4 Recommended Starting Point for Biome

**saphyr's parser architecture** because:
- Most modern Rust idioms (no_std, zero-copy via Cow, trait-based `Input`)
- Full span tracking (`MarkedYaml` with start+end markers)
- `LoadableYamlNode` trait allows building different tree types from same parser
- `Representation` variant demonstrates lazy/lossless scalar handling

Biome must add on top: CST layer, trivia model, error recovery, incremental parsing.

---

## 4. Feature Matrix: Validators & Language Server

### 4.1 Validation Approach

| Feature | kubeconform (Go) | yaml-validator (Rust) | action-validator (Rust) | vscode-yaml (TS) |
|---------|:---:|:---:|:---:|:---:|
| Schema type | JSON Schema Draft 4 | Custom YAML-native | JSON Schema (valico) | JSON Schema (yaml-lang-server) |
| Schema sourcing | Remote HTTP + local | Programmatic only | Embedded (`include_bytes!`) | SchemaStore + user-defined + extensions |
| Schema caching | In-memory + on-disk (SHA-256) | None | N/A (embedded) | On-disk (MD5) + ETag |
| Multi-file | Yes (parallel, N workers) | No | Yes (sequential) | Yes (LSP per-document) |
| Output formats | text, JSON, JUnit, TAP, pretty | Display only | Debug / JSON / WASM | LSP diagnostics |
| Custom validators | Duration format | N/A | Glob paths, job deps | Style checks, key ordering |

### 4.2 Schema Matching Strategies

| Strategy | kubeconform | yaml-validator | action-validator | vscode-yaml |
|----------|:---:|:---:|:---:|:---:|
| File name/pattern | N/A | N/A | `action.yml` → Action, else Workflow | `yaml.schemas` glob mappings |
| File content | `kind` + `apiVersion` fields | N/A | N/A | Regex labels against content |
| Schema Store catalog | N/A | N/A | Submodule | Yes (JSON Schema Store) |
| Extension API | N/A | N/A | N/A | `registerContributor()` |

### 4.3 Error Path Tracking

| Feature | kubeconform | yaml-validator | action-validator | vscode-yaml |
|---------|:---:|:---:|:---:|:---:|
| Path format | JSON pointer (`/spec/containers/0`) | Dot notation (`#.items[0].num`) | JSON pointer | LSP line/column |
| Line/column | No | No | Parse errors only | Yes |
| Nested aggregation | Flat list | Recursive `Multiple` | Recursive `AnyOf`/`OneOf` | LSP diagnostics |

### 4.4 Key Patterns for Biome

1. **Schema Registry** (kubeconform): Template URL pattern with fallthrough across registries
2. **Breadcrumb Path Tracking** (yaml-validator): Stack-based path construction for nested errors
3. **Schema Extension API** (vscode-yaml): Plugin architecture for schema providers with content-based matching
4. **Concurrent Pipeline** (kubeconform): Worker pool with context-based cancellation

---

## 5. Consensus Features (Biome Candidates)

Ranked by prevalence across all 13 tools.

### 5.1 Tier 1: Universal (6+ tools)

| Feature | Linters | Formatters | Parsers | Validators | Total |
|---------|:---:|:---:|:---:|:---:|:---:|
| Indentation control | 3/3 | 3/3 | - | - | 6 |
| Line length / print width | 3/3 | 2/3 | - | - | 5 |
| Comment handling/preservation | 3/3 | 3/3 | 0/3 | - | 6 |
| Multi-document support | 3/3 | 3/3 | 3/3 | 2/4 | 11 |
| Trailing whitespace | 3/3 | 2/3 | - | - | 5 |
| EOF newline | 3/3 | 3/3 | - | - | 6 |

### 5.2 Tier 2: Strong Consensus (3-5 tools)

| Feature | Tools | Notes |
|---------|:---:|-------|
| Key duplicate detection | 3 linters + 1 formatter (yamlfix) | Universal in linters, partial in formatters |
| Quote style enforcement | 2 linters + 3 formatters | Different approaches: lint vs. format |
| Colon spacing | 3 linters | All enforce `max-spaces-before`/`after` |
| Hyphen spacing | 3 linters | `max-spaces-after` |
| Empty line control | 3 linters + 1 formatter | `max`, `max-start`, `max-end` |
| Document start marker (`---`) | 3 linters + 2 formatters | Require/forbid/preserve |
| Truthy value enforcement | 3 linters | Prevent `yes`/`no`/`on`/`off` ambiguity |
| Anchor/alias handling | 2 linters + 1 formatter + 3 parsers | Undeclared, unused, duplicate detection |
| JSON Schema validation | 3 validators + 1 language server | Draft 4 most common |
| Span/position tracking | 1 parser + 1 LSP | Essential for diagnostics |

### 5.3 Tier 3: Present in 2 Tools

| Feature | Tools | Notes |
|---------|:---:|-------|
| Key ordering | yamllint + yamllint-rs | Alphabetical enforcement |
| Brace/bracket spacing | yamllint + yamllint-rs + prettier | `forbid` or `min/max-spaces-inside` |
| Comma spacing | yamllint + yamllint-rs | `min/max-spaces-before/after` |
| Comment indentation | yamllint + yamllint-rs | Comments aligned with content |
| Empty values | yamllint + yamllint-rs | Forbid empty mapping values |
| Float value restrictions | yamllint + yamllint-rs | Forbid NaN, Inf, scientific notation |
| Octal value restrictions | yamllint + yamllint-rs | Forbid implicit/explicit octal |
| New line type | yamllint + yamllint-rs + prettier | Unix/DOS/platform |
| Sequence style forcing | yamlfmt + yamlfix | Flow vs block |
| Schema store integration | kubeconform + vscode-yaml | Remote schema fetching |
| Schema caching | kubeconform + vscode-yaml | On-disk with hash-based naming |

---

## 6. Unique and Notable Features

### 6.1 Worth Implementing in Biome

| Feature | Source | Rationale |
|---------|--------|-----------|
| **Inline disable directives** | yamllint (`# yamllint disable-line rule:X`) | Essential UX for selective rule suppression. Biome already has `// biome-ignore` for JS. |
| **Auto-fix support** | yamllint-rs (7 rules fixable) | Aligns with Biome's safe/unsafe fix model. Trailing spaces, truthy, newline, doc markers, indentation, key ordering. |
| **`# prettier-ignore` node-level** | prettier | Node-level formatting skip is powerful UX. |
| **IR-based formatting** | prettier | Biome already uses IR for JS/CSS. Natural fit for YAML. |
| **`Representation` lazy parsing** | saphyr | Preserving raw text + style enables lossless round-tripping. |
| **Schema extension API** | vscode-yaml | Plugin-driven schema resolution for ecosystem extensibility. |
| **Breadcrumb path tracking** | yaml-validator | Clean nested error path display for deeply nested YAML. |
| **Worker pool validation** | kubeconform | Parallel validation for large projects. |

### 6.2 Domain-Specific (Consider Later)

| Feature | Source | Domain |
|---------|--------|--------|
| Jinja2 template handling | yamlfix | Ansible/DevOps |
| Ansible vault detection | yamlfix | Ansible |
| Kubernetes CRD catalog | vscode-yaml | Kubernetes |
| GitHub Actions schema | action-validator | CI/CD |
| Merge key (`<<`) support | serde-yaml | YAML 1.1 compat |

---

## 7. Architectural Observations

### 7.1 Linting Architecture

All three linters operate on **token streams**, not ASTs. This is significant for Biome:
- yamllint (Py) receives individual pyyaml tokens with prev/next context
- yamllint-rs uses yaml-rust scanner tokens + `ContentAnalysis` shared analysis
- yaml-lint-rs falls back to line-level text analysis for simplicity

**Biome implication:** Biome should lint on its CST (which includes tokens), not on a simplified AST. This gives maximum information for rules.

### 7.2 Formatting Architecture

Three distinct approaches exist in the wild:
1. **Decode-transform-encode** (yamlfmt): Limited by YAML library's output format. Uses "hotfixes" (placeholder comments) to work around library limitations.
2. **IR-based** (prettier): Most powerful. AST → intermediate document → rendered string. Handles line-breaking decisions automatically.
3. **Round-trip + text fixup** (yamlfix): Leverages round-trip YAML library + regex-based post-processing. Fragile but feature-rich.

**Biome implication:** IR-based formatting is the clear winner. Biome already uses this for JS/CSS and should extend it to YAML.

### 7.3 Parser Architecture

All three Rust parsers share lineage from yaml-rust/libyaml:
- Event-based (push) model with scanner → parser → loader pipeline
- None preserve comments or whitespace trivia
- saphyr has the best foundation for extension (spans, zero-copy, trait-based input)

**Biome implication:** Biome needs to build its own CST parser inspired by saphyr's architecture but fundamentally different in its output — a lossless syntax tree with trivia, not an event stream that discards formatting.

### 7.4 Validation Architecture

Two patterns dominate:
1. **JSON Schema** (kubeconform, action-validator, vscode-yaml): Mature, standard, wide schema ecosystem
2. **Custom schema** (yaml-validator): More flexible but no ecosystem

**Biome implication:** JSON Schema validation is the standard approach. Biome should support it, with schema sourcing from Schema Store and user configuration.

### 7.5 Configuration Design Patterns

| Pattern | Tools | Description |
|---------|-------|-------------|
| Rule-level enable/disable + severity | All linters | Per-rule `enable`/`disable`/`warning`/`error` |
| Preset inheritance | yamllint (`extends: default`) | Base configuration that users override |
| Per-rule ignore patterns | yamllint | File patterns to skip specific rules |
| Inline disable comments | yamllint, yamllint-rs | `# yamllint disable-line` |
| File-level ignore | yamlfmt, prettier | `#!yamlfmt!ignore`, `# @noprettier` |
| YAML-based config | All linters, yamlfmt | `.yamllint`, `.yamlfmt` |
| TOML-based config | yamlfix | `pyproject.toml`, `.yamlfix.toml` |

**Biome implication:** Biome already has a mature configuration system (`biome.json`). YAML rules should follow existing patterns: rule-level enable/disable/warn/error, preset groups (recommended, all), and `biome-ignore` comment directives.

---

## 8. Recommended Next Steps

### Phase 1: Parser Foundation
1. **Design Biome's YAML CST** — Study saphyr's span tracking and `Representation` variant. Build a lossless tree with trivia (comments, whitespace) as first-class nodes.
2. **Implement scanner** — Port saphyr's scanner architecture with `Input` trait, but emit trivia tokens.
3. **Add error recovery** — Implement synchronization points (document markers, block boundaries) for parser recovery.

### Phase 2: Linter (High Priority Rules)
Priority order based on consensus + user value:

1. `indentation` — Universal, highest-impact rule
2. `key-duplicates` — Universal, catches real bugs
3. `trailing-spaces` — Universal, trivial to implement
4. `line-length` — Universal, simple
5. `empty-lines` — Universal
6. `new-line-at-end-of-file` — Universal
7. `truthy` — Common source of YAML bugs
8. `key-ordering` — User priority, in 2/3 linters
9. `colons` / `hyphens` — Spacing rules, in 3/3 linters
10. `comments` / `comments-indentation` — Comment hygiene

### Phase 3: Formatter (High Priority Features)
1. **Indent control** (spaces, separate sequence indent)
2. **Quote style** (single/double/preserve)
3. **Line width** with automatic flow/block conversion
4. **Document start/end markers**
5. **Comment preservation and spacing**
6. **Key ordering** (no existing formatter does this — differentiator for Biome)

### Phase 4: Validation
1. **JSON Schema integration** — Schema Store support
2. **Schema association** — File pattern matching + in-file markers
3. **Schema caching** — On-disk with ETag/hash-based invalidation

---

## Appendix A: Default Configuration Comparison (Linters)

### yamllint `default` preset

| Rule | Level | Key Options |
|------|-------|-------------|
| anchors | error | defaults |
| braces | error | `min/max-spaces-inside: 0` |
| brackets | error | `min/max-spaces-inside: 0` |
| colons | error | `max-spaces-before: 0, max-spaces-after: 1` |
| commas | error | `max-spaces-before: 0, min/max-spaces-after: 1` |
| comments | warning | `require-starting-space: true, min-spaces-from-content: 2` |
| comments-indentation | warning | — |
| document-end | **disabled** | — |
| document-start | warning | `present: true` |
| empty-lines | error | `max: 2, max-start: 0, max-end: 0` |
| empty-values | **disabled** | — |
| float-values | **disabled** | — |
| hyphens | error | `max-spaces-after: 1` |
| indentation | error | `spaces: consistent, indent-sequences: true` |
| key-duplicates | error | `forbid-duplicated-merge-keys: false` |
| key-ordering | **disabled** | — |
| line-length | error | `max: 80, allow-non-breakable-words: true` |
| new-line-at-end-of-file | error | — |
| new-lines | error | `type: unix` |
| octal-values | **disabled** | — |
| quoted-strings | **disabled** | — |
| trailing-spaces | error | — |
| truthy | warning | `allowed-values: [true, false], check-keys: true` |

## Appendix B: Formatter Defaults Comparison

| Option | yamlfmt | prettier | yamlfix |
|--------|:---:|:---:|:---:|
| Indent | `2` | `2` | `mapping=2, seq=4` |
| Line width | `0` (off) | `80` | `80` |
| Line ending | OS | `lf` | `\n` |
| Doc start (`---`) | `false` | preserve | `true` |
| Quote style | preserve | double | single |
| Array style | preserve | preserve | flow |
| EOF newline | `false` | always | always |
| Comment padding | 1 space | preserve | 2 spaces |
| Duplicate keys | parser-level | allowed | disallowed |

## Appendix C: Parser Capability Comparison

| Capability | yaml-rust2 | saphyr | serde-yaml |
|------------|:---:|:---:|:---:|
| YAML 1.2 | Y | Y | N (1.1) |
| Zero-copy | N | Y | Partial |
| no_std | N | Y | N |
| Span tracking | Point | **Start+End** | Point |
| Comment preservation | N | N | N |
| Error recovery | N | N | N |
| serde integration | N | N | Y |
| Annotated nodes | N | **Y (MarkedYaml)** | N |
| Lazy scalar parsing | N | **Y (Representation)** | N |
| Merge key support | N | N | **Y** |
| Active maintenance | Y | Y | **N** |
