# YAML Implementation — Next Phase Status

## Completed (Previous Phases)
- Parser (YAML 1.2.2 with anchors/tags/aliases, multiline plain scalars, directives)
- Formatter (58 per-node formatters, 24 snapshot tests, 7 stress tests, all bugs fixed)
- Linter (28 lint rules, all with docs and tests)
- Lint rules registered in CLI config system
- CLI integration tests (format, format --write, lint, check)
- Lint rule AST refactor
- Inline documentation on all rules
- Compiler warnings fixed (zero warnings across all YAML crates)

## All Plans

| Plan | Description | Status |
|------|------------|--------|
| 1-5 | Parser, formatter, linter, CLI integration, lint rule AST refactor | COMPLETE |
| 6 | YAML-specific config options + per-language overrides | COMPLETE |
| 7 | Parser improvements (error messages improved; multiline scalars confirmed working) | COMPLETE |
| 8 | Advanced formatter features (range formatting improved; quote_style deferred to 13) | COMPLETE |
| 9 | Override settings (per-path YAML configuration) | COMPLETE |
| 10 | Cleanup: stale TODO removed, warnings fixed, multiline plain scalar tests added | COMPLETE |
| 11 | Default YAML indent style to spaces (YAML spec compliance) | COMPLETE |
| 12 | Compact block sequence form (`- key: value` via `align(2)`) | COMPLETE |
| 13 | `quote_style` formatter option (single/double with safe conversion) | COMPLETE |
| 14 | JSON Schema validation (lint rule + jsonschema crate) | PENDING (future phase) |
| 15 | Additional formatter polish (flow collection spacing, test expansion) | COMPLETE |
| 16 | Lint rule expansion (5 new rules: noEmptyKeys, noEmptySequenceEntries, useConsistentIndentation, noAnchorReferences, useQuotedStrings) | COMPLETE |
| 17 | Multi-document support hardening (directive lexing, edge case tests) | COMPLETE |
| 18 | `useConsistentSequenceIndentation` lint rule (yamllint `indent-sequences: consistent`) | COMPLETE |

## Remaining Work

### Major Feature

- **JSON Schema validation (Plan 14)** — Validate YAML against JSON schemas (e.g., Kubernetes manifests, CI configs). Requires new `jsonschema` crate dependency, YAML-to-JSON converter, and error range mapping. Estimated 8-15 days.

### Infrastructure Gaps

- **Semantic model** — No `biome_yaml_semantic` crate exists. Current anchor/alias lint rules (`noDuplicateAnchors`, `noUndeclaredAliases`, `noUnusedAnchors`, `useValidMergeKeys`) each independently traverse the full syntax tree to collect anchors and aliases. A semantic model (following the simpler GraphQL pattern) would pre-compute anchor bindings, alias references, and document-level scoping in a single traversal, enabling O(1) lookups. Would also enable future rules like circular reference detection and forward-reference warnings. Performance benefit scales with the number of anchor-related rules.

- **Rename capability** — `rename: None` in `crates/biome_service/src/file_handlers/yaml.rs:227`. Only JavaScript implements rename. For YAML this would apply to anchors (`&name`) and aliases (`*name`) — renaming an anchor updates all referencing aliases. Existing lint rules already collect anchor/alias mappings that could be reused. Simpler than JS rename since YAML anchors have flat document-level scope (no nesting, closures, or module systems).

- **Search capability (GritQL)** — `search: SearchCapabilities { search: None }` in `yaml.rs:238`. Search in Biome is structural pattern matching using GritQL, not text search. Currently only JavaScript and CSS are supported as Grit target languages (`crates/biome_grit_patterns/src/grit_target_language.rs:207-210`). Implementing for YAML requires: a `YamlTargetLanguage` variant, a `GritYamlParser` that converts YAML AST to Grit format, and wiring in the CLI compatibility check. Depends on Grit ecosystem support for YAML.

- **Lexer `rewind()`** — `unimplemented!()` at `crates/biome_yaml_parser/src/lexer/mod.rs:1008`. Part of the `Lexer` trait; enables speculative parsing (try one interpretation, rewind on failure). Not needed because the YAML lexer uses eager disambiguation via a token buffer (`VecDeque<LexToken>`). GraphQL and Grit lexers also don't implement it. Would only matter if speculative parsing or `BufferedLexer` were ever needed.

### Cleanup

- **Unused compact notation syntax kinds** — Four ghost syntax kinds (`YAML_COMPACT_MAPPING`, `YAML_COMPACT_MAPPING_INDENTED`, `YAML_COMPACT_SEQUENCE`, `YAML_COMPACT_SEQUENCE_INDENTED`) defined in `xtask/codegen/src/yaml_kinds_src.rs` and generated into `crates/biome_yaml_syntax/src/generated/kind.rs:72-75`. Never parsed, no AST structs, no formatters. Remnants of an earlier design superseded by the `align(2, ...)` approach. Can be safely deleted.
