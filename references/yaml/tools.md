# YAML Tool Inventory

Reference repositories for YAML language tooling research. All repos are cloned under ~/Clones/.

## Linters

### yamllint (Python)
- **Path:** ~/Clones/adrienverge/yamllint/
- **Type:** Linter
- **Language:** Python
- **Feature locations:**
  - `yamllint/rules/` — 25 rule files (anchors, braces, brackets, colons, commas, comments, comments-indentation, document-end, document-start, empty-lines, empty-values, float-values, hyphens, indentation, key-duplicates, key-ordering, line-length, new-line-at-end-of-file, new-lines, octal-values, quoted-strings, trailing-spaces, truthy)
  - `yamllint/rules/common.py` — shared rule infrastructure
  - `yamllint/conf/` — default configuration and config loading
  - `yamllint/cli.py` — CLI interface and options
- **Notes:** The most mature YAML linter. Comprehensive rule set. Each rule file is self-contained with enable/disable, severity, and rule-specific options.

### yaml-lint-rs (Rust)
- **Path:** ~/Clones/hiromaily/yaml-lint-rs/
- **Type:** Linter
- **Language:** Rust
- **Feature locations:**
  - `core/src/rules/` — 11 rule files (colons, comments, document-start, empty-lines, hyphens, indentation, key-duplicates, line-length, new-line-at-end-of-file, trailing-spaces, truthy)
  - `core/src/linter.rs` — linter engine
  - `core/src/config.rs` — configuration loading
- **Notes:** Rust port of yamllint. Subset of yamllint rules. Useful as a reference for Rust-based linting patterns.

### yamllint-rs (Rust)
- **Path:** ~/Clones/AvnerCohen/yamllint-rs/
- **Type:** Linter
- **Language:** Rust
- **Feature locations:**
  - `src/rules/` — 25 rule files (anchors, braces, brackets, colons, commas, comments, comments-indentation, document-end, document-start, empty-lines, empty-values, float-values, hyphens, indentation, key-duplicates, key-ordering, line-length, new-line-at-end-of-file, new-lines, octal-values, quoted-strings, trailing-spaces, truthy)
  - `src/rules/factory.rs` — rule registration and instantiation
  - `src/rules/registry.rs` — rule lookup
  - `src/rules/base.rs` — base rule trait
  - `src/rules/macros.rs` — rule macros
  - `src/config.rs` — configuration
- **Notes:** More complete Rust port of yamllint than yaml-lint-rs. Factory/registry patterns for rule management.

## Formatters

### yamlfmt (Go)
- **Path:** ~/Clones/google/yamlfmt/
- **Type:** Formatter
- **Language:** Go
- **Feature locations:**
  - `engine.go` — formatting engine core
  - `formatter.go` — formatter interface
  - `feature.go` — feature flag system
  - `formatters/` — formatter implementations (basic formatter)
  - `internal/features/` — individual feature implementations
  - `cmd/yamlfmt/` — CLI entry point
  - `content_analyzer.go` — content analysis
- **Notes:** Google's YAML formatter. Modular engine/formatter architecture. Feature flag system for optional behaviors.

### prettier (JavaScript) — YAML plugin
- **Path:** ~/Clones/prettier/prettier/
- **Type:** Formatter
- **Language:** JavaScript
- **Feature locations:**
  - `src/language-yaml/parser-yaml.js` — YAML parser integration
  - `src/language-yaml/printer-yaml.js` — YAML printer (main formatting logic)
  - `src/language-yaml/print/` — print subroutines for different YAML constructs
  - `src/language-yaml/options.js` — YAML-specific formatting options
  - `src/language-yaml/embed.js` — embedded language handling
- **Notes:** Part of the larger prettier ecosystem. IR-based formatting (doc builders). Handles YAML as one of many languages.

### yamlfix (Python)
- **Path:** ~/Clones/lyz-code/yamlfix/
- **Type:** Formatter/Fixer
- **Language:** Python
- **Feature locations:**
  - `src/yamlfix/config.py` — all formatting configuration options
  - `src/yamlfix/adapters.py` — adapters for YAML libraries
  - `src/yamlfix/services.py` — formatting service logic
  - `src/yamlfix/model.py` — data models
- **Notes:** Opinionated YAML fixer. Configuration-driven. Combines formatting with fixing (e.g., quoting style enforcement, comment normalization).

## Parsers

### yaml-rust2 (Rust)
- **Path:** ~/Clones/Ethiraric/yaml-rust2/
- **Type:** Parser
- **Language:** Rust
- **Feature locations:**
  - `src/parser.rs` — YAML parser (event-based)
  - `src/scanner.rs` — YAML scanner/tokenizer
  - `src/emitter.rs` — YAML emitter (serialization)
  - `src/yaml.rs` — high-level YAML document API
  - `src/lib.rs` — public API
- **Notes:** Maintained fork of yaml-rust. Event-based parser following the YAML spec. Key reference for scanner/parser architecture.

### saphyr (Rust)
- **Path:** ~/Clones/saphyr-rs/saphyr/
- **Type:** Parser
- **Language:** Rust
- **Feature locations:**
  - `saphyr/src/` — high-level API (lib.rs, loader.rs, emitter.rs, annotated.rs, scalar.rs)
  - `parser/src/parser.rs` — YAML parser
  - `parser/src/scanner.rs` — YAML scanner/tokenizer
  - `parser/src/input.rs` — input handling
- **Notes:** Modern Rust YAML library, workspace with separate parser and high-level crates. Annotated node support. Active development.

### serde-yaml (Rust) — UNMAINTAINED
- **Path:** ~/Clones/dtolnay/serde-yaml/
- **Type:** Serializer/Deserializer
- **Language:** Rust
- **Feature locations:**
  - `src/de.rs` — deserialization
  - `src/ser.rs` — serialization
  - `src/loader.rs` — YAML loading
  - `src/mapping.rs` — mapping types
  - `src/value/` — value types
  - `src/error.rs` — error types
- **Notes:** UNMAINTAINED (archived). Serde integration for YAML. Useful reference for Rust YAML data model and error reporting patterns only.

## Validators

### kubeconform (Go)
- **Path:** ~/Clones/yannh/kubeconform/
- **Type:** Validator (Kubernetes-focused)
- **Language:** Go
- **Feature locations:**
  - `pkg/validator/` — validation engine
  - `pkg/registry/` — schema registry (fetching, caching)
  - `pkg/resource/` — YAML resource parsing
  - `pkg/cache/` — schema caching
  - `pkg/output/` — output formatting
- **Notes:** Kubernetes YAML validator using JSON Schema. Schema registry pattern (local + remote schemas). Relevant for schema-based validation approach.

### yaml-validator (Rust)
- **Path:** ~/Clones/MathiasPius/yaml-validator/
- **Type:** Validator
- **Language:** Rust
- **Feature locations:**
  - `yaml-validator/src/lib.rs` — validation engine
  - `yaml-validator/src/modifiers/` — validation modifiers
  - `yaml-validator/src/types/` — type definitions for validation
  - `yaml-validator/src/errors/` — error types
  - `yaml-validator/src/breadcrumb.rs` — path tracking through YAML structure
- **Notes:** Schema-based YAML validator in Rust. Modifier pattern for validation customization. Breadcrumb-based error path reporting.

### action-validator (Rust)
- **Path:** ~/Clones/mpalmer/action-validator/
- **Type:** Validator (GitHub Actions-focused)
- **Language:** Rust
- **Feature locations:**
  - `src/lib.rs` — validation core
  - `src/schemas.rs` — schema definitions
  - `src/schemastore/` — embedded schema storage
  - `src/config.rs` — configuration
  - `src/validation_error.rs` — error types
  - `src/validation_state.rs` — validation state tracking
- **Notes:** GitHub Actions YAML validator. Embedded schema approach (schemas compiled in). Validation state pattern for multi-file validation.

## Language Server

### vscode-yaml (TypeScript)
- **Path:** ~/Clones/redhat-developer/vscode-yaml/
- **Type:** Language Server / IDE Extension
- **Language:** TypeScript
- **Feature locations:**
  - `src/extension.ts` — VS Code extension entry point
  - `src/schema-extension-api.ts` — schema extension API
  - `src/json-schema-cache.ts` — JSON schema caching
  - `src/json-schema-content-provider.ts` — schema content provider
- **Notes:** Red Hat's YAML language server for VS Code. JSON Schema validation integration. Schema extension API allows plugins to provide schemas. Relevant for IDE integration features.

## Summary

| # | Repo | Type | Language | Key Feature Count |
|---|------|------|----------|-------------------|
| 1 | yamllint | Linter | Python | 25 rules |
| 2 | yaml-lint-rs | Linter | Rust | 11 rules |
| 3 | yamllint-rs | Linter | Rust | 25 rules |
| 4 | yamlfmt | Formatter | Go | Engine + feature flags |
| 5 | prettier | Formatter | JavaScript | IR-based printer |
| 6 | yamlfix | Formatter/Fixer | Python | Config-driven fixes |
| 7 | yaml-rust2 | Parser | Rust | Event-based parser |
| 8 | saphyr | Parser | Rust | Annotated parser |
| 9 | serde-yaml | Serializer | Rust | Serde integration (unmaintained) |
| 10 | kubeconform | Validator | Go | Schema registry |
| 11 | yaml-validator | Validator | Rust | Modifier-based validation |
| 12 | action-validator | Validator | Rust | Embedded schemas |
| 13 | vscode-yaml | Language Server | TypeScript | Schema extension API |

**Total: 13 repositories across 5 tool types (3 linters, 3 formatters, 3 parsers, 3 validators, 1 language server)**
