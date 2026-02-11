# Phase 3: Spec Writing — Implementation Plan

## Context

We're executing Phase 3 of the agent evolution model (`kb/tasks/agent-evolution-model.md`). Phases 1 (Feature Extraction) and 2 (Architecture Analysis) are complete. Phase 3 bridges them: synthesizing "what features exist" (feature-research-report.md, 521 lines) with "how Biome integrates languages" (extension-contract.md, 392 lines) into an actionable implementation specification.

The evolution model says: "the first spec IS the template." The YAML spec's structure will be reused for future languages.

## Deliverables

### 1. Create `references/yaml/architecture-notes.md` (~150 lines)

YAML-specific architecture state — the per-language complement to the universal extension contract. This was listed in the Phase 2 growth path but not created. It's a prerequisite input for the spec writer.

Sections:
1. **Layer Status Summary** — Which layers exist and their completeness
2. **Existing Parser Capabilities** — What the YAML parser already handles (indentation-sensitive lexer, block/flow styles, multi-document, comments, anchors/aliases, tags, scalar variants, error recovery)
3. **YAML-Specific Concerns** — Characteristics that differ from JSON and affect implementation:
   - Indentation-sensitive syntax (formatting can't be token-based)
   - Comment placement ambiguity (no clear ownership model)
   - Multi-document files (`---` separators)
   - Anchors and aliases (graph structures, not just trees)
   - Tag directives (`%YAML`, `%TAG`)
   - Scalar folding rules (literal `|` and folded `>`)
   - Key ordering sensitivity
4. **Gap Summary by Layer** — What each missing layer (5, 6, 7) needs, estimated complexity, special challenges
5. **Reference Implementation Notes** — Where YAML diverges from JSON reference

### 2. Create `.claude/agents/lang-spec-writer.md` (~180 lines)

Agent definition following established frontmatter format:

- **Frontmatter**: name `lang-spec-writer`, description with 3 examples (YAML spec from research, GraphQL spec, spec revision), tools (Read, Glob, Grep, TodoWrite, Task), model: opus, color: purple
- **Role**: Expert specification writer translating research + architecture analysis into implementation specs
- **Primary knowledge sources**: feature-research-report.md, extension-contract.md, architecture-notes.md
- **Responsibilities**: Synthesize research, apply Biome patterns, prioritize by tier, account for language specifics, make specs actionable
- **Process**: Validate inputs → Read all inputs → Organize by layer → Detail each phase → Add implementation guidance (5 steps)
- **Output format**: Overview → Prerequisites → Layer 5 (Formatter) with phases → Layer 6 (Analyzer) with tiers → Layer 7 (Service Integration) → Implementation Order → Testing Strategy → Open Questions
- **Quality checklist**: Every option has type/default/reference, every rule has name/category/severity, language concerns addressed, file paths verifiable

Why opus: Spec writing is a synthesis task requiring deep reasoning about priorities and completeness. One-time artifact per language — quality matters more than speed.

### 3. Create `references/yaml/yaml-support-spec.md` (~700 lines)

The key deliverable. Covers all 3 missing layers with phased breakdowns:

**Layer 5: Formatter** (~200 lines)
- Phase 1 (MVP): Crate skeleton, core options (indent_style, indent_width, line_width, line_ending), YAML-specific options (sequence_indent_offset, quote_style, document_start), node formatting priorities (document → mapping → sequence → scalar → flow), comment handling strategy, `YamlCommentStyle` implementation
- Phase 2 (Advanced): Anchors/aliases, tags, literal/folded scalars, multi-document, merge keys
- Phase 3 (Edge cases): Mixed flow/block, special-character scalars, YAML 1.1 boolean ambiguity

**Layer 6: Analyzer** (~250 lines)
- Phase 1 — Tier 1 rules (10 rules, consensus + high-impact): `noKeyDuplicates`, `useConsistentIndentation`, `noTrailingSpaces`, `useLineEndingStyle`, `useFileEndNewline`, `noTruthyStrings`, `useExplicitLineLength`, `useConsistentColonSpacing`, `useConsistentHyphenSpacing`, `noExcessiveEmptyLines`
- Phase 2 — Tier 2 rules (8 rules, common): `useSortedKeys`, `useConsistentCommentSpacing`, `useConsistentCommentIndentation`, brace/bracket/comma spacing, `noEmptyValues`, `useConsistentDocumentMarkers`
- Phase 3 — Tier 3 rules (4 rules, valuable): `noOctalValues`, `useValidFloatValues`, `noUnusedAnchors`, `useConsistentQuoteStyle`
- Suppression comments: `# biome-ignore` syntax, parsing strategy, `YamlSuppressionAction`

Each rule includes: name, category, severity, what it checks, configuration options, edge cases, reference implementation pointer, target file path.

**Layer 7: Service Integration** (~80 lines)
- `DocumentFileSource` — add `Yaml(YamlFileSource)` variant, extension mapping
- `ExtensionHandler` — `YamlFileHandler` with capabilities
- `ServiceLanguage` — settings types, options resolution, `LanguageListSettings` wiring
- Capability functions: parse, format, format_range, lint, code_actions, fix_all, debug_*

**Cross-cutting sections:**
- Implementation order with dependency graph
- Testing strategy (snapshot for formatter, per-rule for analyzer, end-to-end for service)
- Open questions and deferred features

### 4. Update `kb/tasks/agent-evolution-model.md`

Add "Discovered (from YAML spec writing)" section to Phase 3 with findings about spec organization, model choice, architecture notes as prerequisite, monolithic vs split spec, and the spec-as-template pattern.

## Implementation Order

1. Write `references/yaml/architecture-notes.md` (the spec writer's prerequisite input)
2. Write `.claude/agents/lang-spec-writer.md` (the agent definition)
3. Produce `references/yaml/yaml-support-spec.md` (the spec itself — either via the agent or directly)
4. Update `kb/tasks/agent-evolution-model.md` with Phase 3 discoveries
5. Commit plan to `kb/tasks/phase3-spec-writing-plan.md`, then commit all deliverables

## Verification

- All file paths in the spec are valid (Glob check)
- Agent frontmatter matches `lang-feature-extractor.md` format
- Every Tier 1 rule has name, category, severity, config, reference
- Evolution model update matches Phase 1/2 "Discovered" format
- The spec is actionable: a Phase 4 implementer agent can start without clarification
