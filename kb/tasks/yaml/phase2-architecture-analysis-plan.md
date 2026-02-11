# Phase 2: Architecture Analysis — Implementation Plan

## Context

We're executing Phase 2 of the agent evolution model (`kb/tasks/agent-evolution-model.md`). Phase 1 (Feature Extraction) is complete — the `lang-feature-extractor` agent, `/lang-research` command, and `references/yaml/` directory are validated. Phase 2 builds the architecture analysis capability: understanding Biome's extension contract so that a spec writer (Phase 3) knows exactly what needs to be built.

Three exploration agents studied Biome's codebase and identified a 7-layer integration stack with distinct trait contracts at each boundary. YAML already has 4 of 7 layers scaffolded.

## Deliverables

### 1. Create `references/biome/extension-contract.md` (~300 lines)

Structured reference document covering Biome's 7-layer language integration contract. Sections:

1. **Purpose and Audience** — Who uses this, what it covers
2. **7-Layer Overview** — Table mapping each layer to its crate, key trait, and status
3. **Layer 1: Grammar & Code Generation** — `.ungram`, `KindsSrc`, `LanguageKind`, codegen pipeline
4. **Layer 2: Syntax Crate** — `SyntaxKind` impl, `Language` type, `FileSource`, trivia mapping
5. **Layer 3: Factory Crate** — `SyntaxFactory` impl, `make.rs` constructors
6. **Layer 4: Parser Crate** — Entry point, `Parse` struct, `AnyParse` conversion, lexer/parser/tree-sink architecture
7. **Layer 5: Formatter Crate** — `FormatLanguage`, `FormatContext`, `FormatOptions`, `FormatRule<T>`, comment handling
8. **Layer 6: Analyzer Crate** — `Rule` trait, `declare_lint_rule!`, groups, registry, suppression actions
9. **Layer 7: Service Integration** — `DocumentFileSource`, `ExtensionHandler`, `Capabilities`, `ServiceLanguage`, configuration
10. **Integration Status: YAML** — Per-layer existence/completeness matrix
11. **Dependency Graph** — Implementation order diagram
12. **Reference Implementations** — JSON as simplest reference, with key file paths

Each section includes concrete file paths from the Biome codebase.

Key source files to reference:
- `crates/biome_service/src/file_handlers/mod.rs` — `DocumentFileSource`, `ExtensionHandler`, `Capabilities`, `Features`
- `crates/biome_service/src/file_handlers/json.rs` — Complete reference implementation
- `crates/biome_formatter/src/lib.rs` — `FormatLanguage` trait
- `crates/biome_analyze/src/rule.rs` — `Rule` trait
- `crates/biome_yaml_syntax/src/lib.rs` — Existing YAML syntax implementation
- `xtask/codegen/src/yaml_kinds_src.rs` — YAML syntax kinds
- `xtask/codegen/yaml.ungram` — YAML grammar

### 2. Create `.claude/agents/lang-architecture-analyst.md` (~95 lines)

Agent definition following `lang-feature-extractor.md` format:

- **Frontmatter**: name, description with 3 examples (gap analysis, dependency ordering, completeness audit), tools (Glob, Grep, Read, Task, LS, TodoWrite), model: sonnet, color: orange
- **Role**: Expert architecture analyst for Biome's language integration
- **Responsibilities**: Assess integration state, compare against extension contract, produce gap analyses, generate ordered checklists, identify reference implementations
- **Process**: Reference → Inventory → Compare → Analyze → Report (5 steps)
- **Output format**: Current State Summary → Layer-by-Layer Assessment → Gap Checklist → Dependency Graph → Risk Areas

References `references/biome/extension-contract.md` as primary knowledge source.

### 3. Update `kb/tasks/agent-evolution-model.md`

Insert `**Discovered (from YAML architecture study):**` section between lines 61 and 63, with 5 findings:

1. **7 distinct layers, not monolithic** — Each has a separate trait contract; the extension contract organizes naturally by layer
2. **Extractor and analyst are fully independent** — No shared inputs/state/outputs; can run in parallel; spec writer is the first consumer of both
3. **Biome knowledge splits: universal / per-language** — Universal in `references/biome/extension-contract.md`, per-language in `references/{language}/architecture-notes.md`
4. **YAML has significant existing scaffolding** — 4 of 7 layers done (grammar, syntax, factory, parser stub); gaps are formatter, analyzer, service integration
5. **JSON is the right reference implementation** — Simplest end-to-end integration; CSS/JS have complexity YAML doesn't need

## Implementation Order

1. Create `references/biome/` directory
2. Write `references/biome/extension-contract.md` (the agent references this)
3. Write `.claude/agents/lang-architecture-analyst.md`
4. Update `kb/tasks/agent-evolution-model.md` with Phase 2 discoveries
5. Commit and push all changes

## Verification

- Read the extension contract document and verify all file paths are valid using Glob
- Verify the agent definition follows the same frontmatter format as `lang-feature-extractor.md`
- Verify the evolution model update matches the format of the Phase 1 "Discovered" section
- Run the new agent against YAML to produce a gap analysis (optional smoke test)
