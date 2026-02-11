# YAML Architecture Notes

Per-language complement to the universal extension contract (`references/biome/extension-contract.md`). Captures YAML-specific integration state, parser capabilities, language concerns, and gap analysis for the missing layers.

## Layer Status Summary

| Layer | Crate | Status | Completeness |
|-------|-------|--------|--------------|
| 1 | `xtask/codegen/` (yaml.ungram, yaml_kinds_src.rs) | **Complete** | Grammar: 315 lines, 96 kinds |
| 2 | `biome_yaml_syntax` | **Complete** | `YamlLanguage`, `YamlSyntaxKind`, `YamlFileSource` (14+ extensions) |
| 3 | `biome_yaml_factory` | **Complete** | `YamlSyntaxFactory` with full slot validation |
| 4 | `biome_yaml_parser` | **Complete** | `parse_yaml()`, indentation-sensitive lexer, `AnyParse` conversion |
| 5 | `biome_yaml_formatter` | **Not started** | Crate does not exist |
| 6 | `biome_yaml_analyze` | **Not started** | Crate does not exist |
| 7 | `biome_service` (YAML wiring) | **Not started** | No `Yaml` variant, no handler, no settings |

**Summary:** 4 of 7 layers complete. The foundation (grammar → syntax → factory → parser) is solid. The remaining 3 layers (formatter, analyzer, service integration) constitute the full feature surface visible to users.

## Existing Parser Capabilities

The YAML parser (`crates/biome_yaml_parser/`) is the most complex of the completed layers. It handles:

1. **Indentation-sensitive lexing** — The lexer tracks indentation depth via a scope stack, emitting indent/dedent-like context transitions. This is the hardest part of YAML parsing and it's already done.
2. **Block styles** — Block mappings (`key: value`), block sequences (`- item`), and nested block structures with arbitrary indentation depth.
3. **Flow styles** — Flow mappings (`{a: 1}`), flow sequences (`[1, 2]`), and nested flow collections.
4. **Multi-document** — `---` document start and `...` document end markers, with multiple documents in a single file.
5. **Comments** — `#` comments captured as trivia tokens, attached to the syntax tree.
6. **Anchors and aliases** — `&anchor` definitions and `*alias` references parsed as distinct node types.
7. **Tags** — Verbatim (`!<tag:uri>`), shorthand (`!!str`), and named (`!custom`) tags.
8. **Scalar variants** — Plain scalars, single-quoted, double-quoted, literal block (`|`), and folded block (`>`).
9. **Block scalar indicators** — Chomping (`-`, `+`) and indentation indicators for literal/folded blocks.
10. **Directives** — `%YAML` version and `%TAG` handle directives.
11. **Error recovery** — Basic recovery at document boundaries and block-level synchronization points.

**CST fidelity:** The parser produces a lossless concrete syntax tree via `biome_rowan`. All whitespace, comments, and formatting details are preserved as trivia — this is the foundation that makes formatter and analyzer development possible.

## YAML-Specific Concerns

These characteristics distinguish YAML from JSON and affect formatter/analyzer design decisions.

### 1. Indentation-sensitive syntax

Unlike JSON (where braces/brackets define structure), YAML uses indentation as syntax. Implications:
- **Formatter cannot be purely token-based.** Reformatting indentation changes the semantic meaning. The formatter must understand the block structure to adjust indentation safely.
- **Indent width changes are structural changes.** Going from 2-space to 4-space indent requires recalculating all nested indentation, not just swapping tokens.
- **Mixed indent detection** must be a lint rule, not just a formatter concern.

### 2. Comment placement ambiguity

YAML comments (`# ...`) can appear in many positions, and their "ownership" (which node they belong to) is ambiguous:
- End-of-line comments: `key: value  # comment` — belongs to the mapping entry
- Above-node comments: `# comment` on a line before a key — belongs to the next node or the parent?
- Between-node comments: comments between sequence items — belong to the previous or next item?
- The `CommentStyle` implementation must define placement rules. JSON's comment model is simpler (fewer ambiguous positions). Reference: `biome_json_formatter/src/comments.rs`.

### 3. Multi-document files

YAML files can contain multiple documents separated by `---`. Implications:
- Formatter must handle document boundaries as formatting barriers.
- Lint rules must scope per-document (e.g., key duplicates within a single document, not across documents).
- The `%YAML` and `%TAG` directives apply per-document.

### 4. Anchors and aliases (graph structures)

`&anchor` / `*alias` create reference relationships that make YAML a graph, not a tree:
- Lint rules need to track anchor definitions and alias usages across the document.
- `noUnusedAnchors` requires cross-document-scope analysis.
- The `<<` merge key (YAML 1.1) creates implicit aliasing.
- Formatter must preserve anchor/alias names exactly.

### 5. Tag directives

`%YAML 1.2` and `%TAG !prefix! tag:uri:` directives affect interpretation:
- Formatter must preserve directives and their positioning before the first document.
- Analyzer rules that interpret scalar types (truthy, octal) need to consider the declared YAML version.

### 6. Scalar folding rules

Literal (`|`) and folded (`>`) block scalars have complex whitespace semantics:
- The content's indentation is relative to the indicator, not the parent.
- Chomping indicators (`-`, `+`, default) control trailing newlines.
- Formatter must understand these rules to avoid changing scalar values.
- These are the most complex formatting nodes in YAML.

### 7. Key ordering sensitivity

Unlike JSON (where key order is implementation-defined), YAML key ordering is often semantically significant in practice:
- Kubernetes manifests expect `apiVersion` before `kind` before `metadata`.
- CI/CD files have conventional ordering (`name`, `on`, `jobs`).
- A `useSortedKeys` rule needs configurable ordering strategies (alphabetical, preserve, custom).

## Gap Summary by Layer

### Layer 5: Formatter — Estimated complexity: **High**

**What's needed:**
- `crates/biome_yaml_formatter/` crate with `YamlFormatLanguage`, `YamlFormatContext`, `YamlFormatOptions`
- `YamlCommentStyle` implementing `CommentStyle` — the hardest part, due to comment placement ambiguity
- Per-node `FormatRule` implementations for ~30 node types from the grammar
- YAML-specific format options beyond the base 4 (indent_style, indent_width, line_width, line_ending)

**Special challenges:**
- Indentation is structural, not cosmetic — reformatting must preserve semantics
- Block scalar content must not be reformatted (it's literal text)
- Comment placement decisions have no single "correct" answer
- Flow ↔ block style conversion is a formatting decision with semantic implications

**Reference:** `crates/biome_json_formatter/` — but JSON formatting is significantly simpler (no indentation sensitivity, no block scalars, minimal comment ambiguity).

### Layer 6: Analyzer — Estimated complexity: **Medium**

**What's needed:**
- `crates/biome_yaml_analyze/` crate with `analyze()` entry point
- Suppression comment parsing (`# biome-ignore`)
- `YamlSuppressionAction` implementation
- ~22 lint rules across 3 tiers, organized by category
- Rule group registration and codegen (`just gen-analyzer`)

**Special challenges:**
- Some rules (truthy, octal) need YAML version awareness
- Cross-node analysis for anchors/aliases
- Suppression comments use `#` (same as YAML comments) — parser must distinguish

**Reference:** `crates/biome_json_analyze/` — structurally similar, but YAML has far more rules (22 vs JSON's ~5).

### Layer 7: Service Integration — Estimated complexity: **Low-Medium**

**What's needed:**
- `Yaml(YamlFileSource)` variant in `DocumentFileSource`
- `YamlFileHandler` implementing `ExtensionHandler`
- `ServiceLanguage for YamlLanguage` with settings types
- Language settings in `LanguageListSettings`
- Capability functions wiring parse/format/lint/code_actions

**Special challenges:**
- Mostly boilerplate following JSON's pattern
- YAML-specific format options need settings integration
- File extension mapping is already partially in `YamlFileSource` (14+ extensions)

**Reference:** `crates/biome_service/src/file_handlers/json.rs` — closest match, can be adapted directly.

## Reference Implementation Notes

JSON (`biome_json_formatter`, `biome_json_analyze`, `file_handlers/json.rs`) is the right reference, but YAML diverges in these areas:

| Aspect | JSON Reference | YAML Difference |
|--------|---------------|-----------------|
| Indentation | Cosmetic (braces define structure) | Structural (indentation IS structure) |
| Comments | `//` and `/* */`, clear placement | `#` only, ambiguous placement |
| Scalars | Strings, numbers, booleans, null | + block scalars, tagged values, anchors |
| Documents | Single root value | Multiple documents per file |
| Format options | 4 base options | 4 base + ~6 YAML-specific |
| Lint rules | ~5 rules | ~22 rules across 3 tiers |
| Style variants | None (one canonical style) | Block vs flow, 5 scalar styles, quote styles |

The JSON reference provides the structural skeleton (trait implementations, crate layout, codegen integration). YAML-specific logic must be built on top, particularly in comment handling, indentation management, and the much larger rule set.
