# YAML Linting, Formatting & Validation — Gap Analysis & Implementation Plan

## Current State of Biome YAML

| Component | Status |
|-----------|--------|
| Parser (YAML 1.2.2) | **Complete** — 42 test files, full spec coverage, `---` document start fix applied |
| Syntax/Factory | **Complete** — code generated |
| Formatter | **Infrastructure only** — all nodes output verbatim (no formatting). `cargo codegen formatter yaml` not yet supported. |
| Analyzer/Linter | **18 lint rules implemented** — covering core, style, and anchor/alias categories |
| Configuration | **Complete** — formatter/linter/assist toggles |
| Service Integration | **Complete** — parse, format, lint, code actions wired |

### Implemented Lint Rules (18 total)

| Rule | Category | Severity | Recommended |
|------|----------|----------|-------------|
| `noDuplicateKeys` | Core / Bug Prevention | Error | Yes |
| `noDuplicateFlowKeys` | Core / Bug Prevention | Error | Yes |
| `noTabIndentation` | Core / Bug Prevention | Error | Yes |
| `noTruthyValues` | Core / Bug Prevention | Warning | Yes |
| `noEmptyValues` | Core / Bug Prevention | Warning | Yes |
| `noTrailingSpaces` | Core / Bug Prevention | Error | Yes |
| `useFinalNewline` | Core / Bug Prevention | Warning | Yes |
| `noImplicitOctalValues` | Core / Bug Prevention | Warning | Yes |
| `noConsecutiveBlankLines` | Core / Bug Prevention | Warning | Yes |
| `noDuplicateAnchors` | Anchor / Alias | Error | Yes |
| `noUnusedAnchors` | Anchor / Alias | Warning | Yes |
| `noUndeclaredAliases` | Anchor / Alias | Error | Yes |
| `useConsistentBooleanStyle` | Style | Warning | Yes |
| `useConsistentQuoteStyle` | Style | Warning | No |
| `useConsistentKeyOrdering` | Style | Warning | No |
| `useDocumentMarkers` | Style | Warning | No |
| `useKeyNamingConvention` | Style | Warning | No |
| `noEmptyDocument` | Style | Warning | No |

All 18 rules have passing tests (36 test files: valid + invalid per rule, with snapshot tests).

## Research: Popular YAML Tools

### yamllint (Python) — 23 Rules

The de facto standard YAML linter. Rules organized into structural/semantic, formatting/style, document structure, and whitespace categories.

#### Structural / Semantic Rules

| Rule | Description | Key Options |
|------|-------------|-------------|
| **anchors** | Detects duplicated anchors and aliases referencing undeclared anchors | `forbid-undeclared-aliases`, `forbid-duplicated-anchors`, `forbid-unused-anchors` |
| **key-duplicates** | Prevents duplicate keys in mappings | `forbid-duplicated-merge-keys` |
| **key-ordering** | Enforces alphabetical key sorting in mappings | `ignored-keys` |
| **empty-values** | Prevents nodes with implicit null values | `forbid-in-block-mappings`, `forbid-in-flow-mappings`, `forbid-in-block-sequences` |
| **octal-values** | Prevents unintended octal number interpretations (numbers starting with 0) | `forbid-implicit-octal`, `forbid-explicit-octal` |
| **float-values** | Restricts floating-point number representations | `forbid-inf`, `forbid-nan`, `forbid-scientific-notation`, `require-numeral-before-decimal` |
| **truthy** | Restricts non-standard boolean value representations (`yes`, `no`, `on`, `off`) | `allowed-values`, `check-keys` |

#### Formatting / Style Rules

| Rule | Description | Key Options |
|------|-------------|-------------|
| **indentation** | Enforces consistent indentation | `spaces`, `indent-sequences`, `check-multi-line-strings` |
| **braces** | Controls spacing inside flow mappings `{ }` | `forbid`, `min-spaces-inside`, `max-spaces-inside` |
| **brackets** | Controls spacing inside flow sequences `[ ]` | Same as braces |
| **colons** | Regulates spacing around `:` | `max-spaces-before`, `max-spaces-after` |
| **commas** | Controls spacing around `,` | `max-spaces-before`, `min-spaces-after`, `max-spaces-after` |
| **hyphens** | Controls spaces after `-` list markers | `max-spaces-after` |
| **quoted-strings** | Controls string quoting requirements and style | `quote-type`, `required`, `extra-required`, `extra-allowed`, `check-keys` |
| **comments** | Controls comment formatting and positioning | `require-starting-space`, `ignore-shebangs`, `min-spaces-from-content` |
| **comments-indentation** | Enforces comment indentation alignment with content | — |

#### Document Structure Rules

| Rule | Description | Key Options |
|------|-------------|-------------|
| **document-start** | Requires or forbids the `---` document start marker | `present` |
| **document-end** | Requires or forbids the `...` document end marker | `present` |

#### Whitespace Rules

| Rule | Description | Key Options |
|------|-------------|-------------|
| **line-length** | Maximum line length | `max`, `allow-non-breakable-words`, `allow-non-breakable-inline-mappings` |
| **empty-lines** | Controls consecutive blank lines | `max`, `max-start`, `max-end` |
| **trailing-spaces** | Disallows trailing whitespace at line endings | — |
| **new-line-at-end-of-file** | Requires newline at end of file | — |
| **new-lines** | Controls line ending type | `type` (unix/windows) |

#### Presets

- **default**: anchors, braces, brackets, colons, commas, empty-lines, hyphens, indentation, key-duplicates, line-length, new-line-at-end-of-file, new-lines, trailing-spaces at error; comments, comments-indentation, document-start, truthy at warning.
- **relaxed**: downgrades many errors to warnings, disables comments/document-start/truthy.

---

### Prettier — YAML Formatting

Added YAML support in v1.14.0. Opinionated with limited configuration.

| Option | Default | Description |
|--------|---------|-------------|
| **printWidth** | 80 | Line length at which Prettier wraps |
| **tabWidth** | 2 | Spaces per indentation level |
| **useTabs** | false | Tabs instead of spaces |
| **endOfLine** | "lf" | Line ending style |
| **singleQuote** | false | Prefer single quotes (uses whichever minimizes escapes) |
| **proseWrap** | "preserve" | How to wrap prose |
| **bracketSpacing** | true | Spaces between brackets in flow collections |
| **requirePragma** | false | Only format files with `# @prettier` pragma |
| **insertPragma** | false | Insert `# @format` pragma |

Formatting behaviors: normalizes key-value spacing, enforces consistent indentation, prose wrapping, quote normalization, supports `# prettier-ignore`.

#### prettier-plugin-yaml (Community)

| Option | Description |
|--------|-------------|
| **yamlBlockStyle** | Enforce `folded` or `literal` for multi-line strings |
| **yamlCollectionStyle** | Enforce `block` or `flow` for collections |
| **yamlQuoteKeys** | Quote all mapping keys |
| **yamlQuoteValues** | Quote all string values |

---

### eslint-plugin-yml — 28 Rules

The most comprehensive Node.js YAML linter.

#### YAML-Specific Rules

| Rule | Description |
|------|-------------|
| **yml/block-mapping** | Require or disallow block style mappings |
| **yml/block-mapping-colon-indicator-newline** | Consistent line breaks after `:` indicator |
| **yml/block-mapping-question-indicator-newline** | Consistent line breaks after `?` indicator |
| **yml/block-sequence** | Require or disallow block style sequences |
| **yml/block-sequence-hyphen-indicator-newline** | Consistent line breaks after `-` indicator |
| **yml/file-extension** | Enforce .yml vs .yaml |
| **yml/indent** | Consistent indentation |
| **yml/key-name-casing** | Naming convention for keys (camelCase, snake_case, etc.) |
| **yml/no-empty-document** | Disallow empty documents |
| **yml/no-empty-key** | Disallow empty mapping keys |
| **yml/no-empty-mapping-value** | Disallow empty mapping values |
| **yml/no-empty-sequence-entry** | Disallow empty sequence entries |
| **yml/no-tab-indent** | Disallow tabs for indentation |
| **yml/no-trailing-zeros** | Disallow trailing zeros for floats |
| **yml/plain-scalar** | Require or disallow plain style scalar |
| **yml/quotes** | Consistent double or single quotes |
| **yml/require-string-key** | Disallow non-string mapping keys |
| **yml/sort-keys** | Require sorted mapping keys |
| **yml/sort-sequence-values** | Require sorted sequence values |
| **yml/vue-custom-block/no-parsing-error** | Disallow parsing errors in Vue custom blocks |

#### Extension Rules

| Rule | Description |
|------|-------------|
| **yml/flow-mapping-curly-newline** | Consistent line breaks inside braces |
| **yml/flow-mapping-curly-spacing** | Consistent spacing inside braces |
| **yml/flow-sequence-bracket-newline** | Linebreaks after/before flow sequence brackets |
| **yml/flow-sequence-bracket-spacing** | Consistent spacing inside brackets |
| **yml/key-spacing** | Consistent spacing between keys and values |
| **yml/no-irregular-whitespace** | Disallow irregular whitespace |
| **yml/no-multiple-empty-lines** | Disallow multiple empty lines |
| **yml/spaced-comment** | Consistent spacing after `#` |

---

### YAML Language Server (Red Hat / VS Code)

#### Validation

| Capability | Description |
|------------|-------------|
| **Syntax validation** | Valid YAML detection |
| **JSON Schema validation** | Structure/value validation against schemas |
| **Unused anchors** | Anchors never referenced |
| **Flow style enforcement** | Restrict flow mappings/sequences |
| **Key ordering** | Alphabetical key validation |

#### Formatting Options

| Setting | Description |
|---------|-------------|
| **yaml.format.singleQuote** | Use single quotes |
| **yaml.format.bracketSpacing** | Spaces between brackets |
| **yaml.format.proseWrap** | Prose wrapping mode |
| **yaml.format.printWidth** | Line length for wrapping |

#### Editor Features

- Auto-completion from JSON schemas
- Hover descriptions from schemas
- Document outline/symbols
- Code lens for schema links
- Definition navigation for anchors/aliases
- Schema Store integration
- Kubernetes CRD support

---

### KubeLinter (Kubernetes-Specific)

Domain-specific Kubernetes YAML validator with 25+ default checks (security, best practices) and 15+ opt-in checks. Not relevant for general-purpose YAML but useful context for potential future schema-based validation.

### actionlint (GitHub Actions-Specific)

Domain-specific GitHub Actions workflow validator. Comprehensive expression type-checking, shellcheck integration, action input validation. Not relevant for general-purpose YAML.

---

## Gap Analysis: What's Missing from Biome

### Formatter — Per-Node Rules

The formatter currently passes all nodes through verbatim mode. `cargo codegen formatter yaml` is not yet supported by the codegen tool. Every popular tool handles these:

| Formatting Capability | Implemented By | Priority | Status |
|---|---|---|---|
| **Indentation normalization** (spaces, consistent depth) | yamllint, Prettier, eslint-plugin-yml, YAML LS | Critical | Not started |
| **Trailing whitespace removal** | yamllint | Critical | **Lint rule only** (`noTrailingSpaces`) |
| **Newline at end of file** | yamllint, Prettier | Critical | **Lint rule only** (`useFinalNewline`) |
| **Line ending normalization** (LF vs CRLF) | yamllint, Prettier | Critical | Not started |
| **Colon spacing** (`key: value` not `key :value`) | yamllint, eslint-plugin-yml | High | Not started |
| **Comma spacing** in flow collections | yamllint | High | Not started |
| **Brace spacing** `{ key: value }` vs `{key: value}` | yamllint, Prettier, eslint-plugin-yml | High | Not started |
| **Bracket spacing** `[ 1, 2 ]` vs `[1, 2]` | yamllint, Prettier, eslint-plugin-yml | High | Not started |
| **Hyphen spacing** after `-` in sequences | yamllint | High | Not started |
| **Quote normalization** (prefer single/double, minimize escapes) | Prettier, yamllint, eslint-plugin-yml | Medium | **Lint rule only** (`useConsistentQuoteStyle`) |
| **Consecutive blank line collapsing** | yamllint, eslint-plugin-yml | Medium | **Lint rule only** (`noConsecutiveBlankLines`) |
| **Comment spacing** (space after `#`) | yamllint, eslint-plugin-yml | Medium | Not started |
| **Line wrapping** at configured width | yamllint, Prettier | Medium | Not started |
| **Block vs flow style** enforcement | eslint-plugin-yml, prettier-plugin-yaml | Low | Not started |

### Core Lint Rules (Bug Prevention)

These catch real bugs and are universally present:

| Rule | Description | Present In | Priority | Status |
|---|---|---|---|---|
| **noDuplicateKeys** | Duplicate keys in mappings (last wins silently) | yamllint, eslint-plugin-yml, actionlint | Critical | **Done** |
| **noTabIndentation** | Tabs are forbidden in YAML spec | yamllint, eslint-plugin-yml | Critical | **Done** |
| **useFinalNewline** | Require trailing newline | yamllint | High | **Done** |
| **noTrailingSpaces** | No trailing whitespace | yamllint | High | **Done** |
| **noTruthyValues** | Ban `yes`/`no`/`on`/`off`/`y`/`n` as booleans (YAML 1.1 legacy footgun) | yamllint | High | **Done** |
| **noEmptyValues** | Disallow implicit null mapping values | yamllint, eslint-plugin-yml | High | **Done** |
| **noUnusedAnchors** | Anchors (`&name`) never referenced by aliases | yamllint, YAML LS | Medium | **Done** |
| **noUndeclaredAliases** | Aliases (`*name`) referencing non-existent anchors | yamllint | Medium | **Done** |
| **noDuplicateAnchors** | Multiple anchors with same name | yamllint | Medium | **Done** |
| **noImplicitOctalValues** | Numbers like `0777` silently become octal | yamllint | Medium | **Done** |

**Core lint coverage: 10/10 (100%)**

### Style Lint Rules (Consistency)

Opinionated rules for code consistency:

| Rule | Description | Present In | Priority | Status |
|---|---|---|---|---|
| **useConsistentQuoteStyle** | Enforce single or double quotes | yamllint, eslint-plugin-yml, Prettier | High | **Done** |
| **useConsistentBooleanStyle** | Enforce `true`/`false` over alternatives | yamllint (truthy rule) | High | **Done** |
| **useDocumentMarkers** | Require or forbid `---`/`...` | yamllint | Medium | **Done** |
| **useConsistentKeyOrdering** | Alphabetical key sorting | yamllint, eslint-plugin-yml, YAML LS | Medium | **Done** |
| **useKeyNamingConvention** | camelCase, snake_case, kebab-case for keys | eslint-plugin-yml | Medium | **Done** |
| **noConsecutiveBlankLines** | Max empty lines between content | yamllint, eslint-plugin-yml | Medium | **Done** |
| **useBlockStyle** / **useFlowStyle** | Enforce collection style | eslint-plugin-yml, YAML LS | Low | Not started |
| **useStringKeys** | Disallow non-string mapping keys | eslint-plugin-yml | Low | Not started |
| **noEmptyDocument** | Disallow empty YAML documents | eslint-plugin-yml | Low | **Done** |
| **noFloatTrailingZeros** | `1.0` -> `1` normalization | eslint-plugin-yml | Low | Not started |
| **useLineLength** | Maximum line length | yamllint | Low | Not started |
| **useCommentSpacing** | Space after `#`, min distance from content | yamllint, eslint-plugin-yml | Low | Not started |

**Style lint coverage: 7/12 (58%)**

### Advanced Features

| Feature | Description | Present In | Status |
|---|---|---|---|
| **JSON Schema validation** | Validate YAML structure against schemas | YAML LS | Not started |
| **Format range** | Format a selection, not whole file | YAML LS, VS Code | Not started |
| **Format on type** | Format as user types | YAML LS | Not started |
| **Suppress comments** | `# biome-ignore` (infrastructure already supports this) | — | Infrastructure ready |
| **Auto-fix code actions** | Fix-all for lint violations (infrastructure ready) | — | Infrastructure ready |

---

## Overall Coverage Summary

| Category | Implemented | Total | Coverage |
|----------|-------------|-------|----------|
| Formatter per-node rules | 0 | 14 | 0% |
| Core lint rules (bug prevention) | 10 | 10 | 100% |
| Style lint rules (consistency) | 7 | 12 | 58% |
| Advanced features | 0 (2 infra-ready) | 5 | 0% |
| **Overall** | **17** | **41** | **41%** |

**Note:** `noDuplicateFlowKeys` is implemented but not counted above as it overlaps with `noDuplicateKeys` (handles flow mappings specifically).

---

## Remaining Implementation Order

### Phase 1: Formatter (Blocked)

The YAML formatter codegen is not yet supported (`cargo codegen formatter yaml` returns an error). This blocks all formatter work. Options:
1. Extend the codegen tool to support YAML
2. Manually create per-node format rule stubs (labor-intensive but possible)
3. Wait for upstream codegen support

If unblocked, the priority order would be:
1. Indentation normalization
2. Whitespace handling (trailing spaces, blank lines, final newline)
3. Colon/comma/hyphen spacing
4. Flow collection spacing (braces, brackets)
5. Quote normalization
6. Comment formatting
7. Line wrapping
8. Block vs flow style enforcement

### Phase 2: Remaining Lint Rules (Ready to Implement)

**Low Priority (all remaining):**
1. `useBlockStyle` / `useFlowStyle` — enforce collection style
2. `useStringKeys` — disallow non-string mapping keys
3. `noFloatTrailingZeros` — `1.0` -> `1` normalization
4. `useLineLength` — maximum line length
5. `useCommentSpacing` — space after `#`, min distance from content

### Phase 3: Advanced Features (Future)
- JSON Schema validation
- Format range / format on type
- Editor integration enhancements

---

## Key Insights

1. **Linting is in excellent shape.** With 18 rules implemented, Biome covers 100% of core bug-prevention rules and 58% of style rules. All medium-priority gaps are closed; only low-priority style rules remain.

2. **The formatter is the critical gap.** Zero formatting actually happens — all nodes output verbatim. This is blocked by missing codegen support for YAML in the formatter pipeline.

3. **Parser `---` fix landed.** The lexer now correctly produces `DIRECTIVE_END` tokens for `---` (document start markers), fixing trivia attachment for comments after `---`.

4. **Anchor/alias rules have parser limitations.** The YAML parser doesn't yet support `&anchor`/`*alias` syntax at the AST level, so the anchor/alias rules (`noDuplicateAnchors`, `noUnusedAnchors`, `noUndeclaredAliases`) use text-based scanning as a workaround. These rules work but would benefit from proper parser support.
