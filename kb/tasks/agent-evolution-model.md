# Agent Evolution Model

How the incremental approach (Option E) evolves toward the full plugin (Option C), and how that evolution gets captured. Informed by the discovery that this effort will be repeated for multiple languages (shell/bash, turtle, toml, ruby, rust, etc.).

## Key insight: multi-language reuse

The workflow repeats across 6+ languages. What stays the same:

- The workflow phases (discover tools → extract features → study biome internals → write specs → implement → review)
- The agent roles (feature extractor, architecture analyst, spec writer, code reviewer)
- The skills (feature comparison templates, biome extension contract knowledge, spec methodology/rubric)
- The hooks (require spec before implementation, post-edit validation)
- The commands (research, architecture, spec, status, review)

What changes per language:

- Which repos to clone and scan
- Language-specific grammar/parsing concerns
- Which biome crates to reference
- The feature matrix content

The **process itself** is the reusable asset, not the language-specific knowledge. This shifts the target architecture from Option B (YAML-specific 7-phase workflow) to Option C (generalized `biome-lang-support` plugin), built incrementally using YAML as the first instantiation.

Naming should be language-agnostic from the start: `lang-feature-extractor` not `yaml-feature-extractor`. Language-specific knowledge goes into `references/{language}/` directories, not into agent prompts.

## Phase-by-phase evolution

### Phase 1: Feature Extraction (YAML as first language)

**Build:**
- `lang-feature-extractor` agent + `/lang-research` command
- A `references/yaml/` directory with tool inventory (the 13 cloned repos)

**Discover:**
- What structured output actually looks like for a feature comparison (the template is unknown until extraction is done once)
- Whether parallel subagents per repo are worth it or if sequential is fine
- What reference knowledge the extractor needs per tool vs. what's generic
- Whether language spec grounding (e.g. YAML 1.1 vs 1.2) needs to be part of the extraction or can be added in synthesis

**Discovered (from first YAML run):**

1. **Parallel by tool type, not per-repo.** 4 parallel agents (linters, formatters, parsers, validators) across 13 repos completed in ~260s wall-clock. Sequential would have been ~892s — a 3.4x speedup. Grouping by tool type was better than per-repo because cross-tool comparison (linter vs linter) is the core output, so each agent produced a coherent comparison directly. Per-repo agents would have required heavier synthesis to merge 13 separate reports.

2. **Structured output template emerged.** The report settled into: executive summary → feature matrices by category → consensus features ranked by prevalence → unique features with rationale → architectural observations → spec grounding → recommended next steps. Appendices for default config comparisons. This template is reusable for other languages.

3. **Reference knowledge is split cleanly.** The tool inventory (`references/yaml/tools.md`) with repo paths and feature file locations was all the per-tool knowledge agents needed. Generic extraction instructions (what to look for in linters vs formatters vs parsers) were provided in the agent prompt. No tool-specific knowledge leaked into the agent definition.

4. **Spec grounding must be part of the workflow, not an afterthought.** The first run produced a report without spec classification. It was immediately apparent that features like `truthy` and `octal-values` are meaningless without understanding the YAML 1.1→1.2 divergence. The agent and command were updated to require spec basis classification (spec-mandated, spec-divergence, spec-ambiguity, tool-opinion) for each feature.

**Crystallizes:** The structured output template is now a reusable pattern. The agent definition and `/lang-research` command are no longer drafts — they've been exercised and refined. The `references/{language}/` directory pattern (tools.md + feature-research-report.md) is validated.

### Phase 2: Architecture Analysis

**Build:**
- `lang-architecture-analyst` agent
- `references/biome/` directory capturing what the JSON/CSS study revealed about biome's extension contract

**Discover:**
- What questions the spec writer will need answered
- Whether the architecture analyst and feature extractor need to talk to each other (routing/chaining) or are truly independent
- What biome knowledge is language-specific vs. universal

**Discovered (from YAML architecture study):**

1. **7 distinct layers, not monolithic.** Each layer has a separate trait contract (`Language`, `SyntaxFactory`, `Parser`, `FormatLanguage`, `Rule`, `ExtensionHandler`, `ServiceLanguage`). The extension contract document organizes naturally by layer, with each section mapping to a specific crate boundary.

2. **Extractor and analyst are fully independent.** No shared inputs, state, or outputs between the feature extractor and architecture analyst. They can run in parallel. The spec writer (Phase 3) is the first consumer of both — it needs the feature matrix (what to build) and the extension contract (how to build it).

3. **Biome knowledge splits cleanly: universal vs. per-language.** Universal knowledge (the 7-layer contract, trait signatures, file patterns) lives in `references/biome/extension-contract.md`. Per-language knowledge (what exists, what's missing) goes in `references/{language}/architecture-notes.md`. This split means the analyst agent's prompt stays language-agnostic.

4. **YAML has significant existing scaffolding.** 4 of 7 layers are complete (grammar, syntax, factory, parser). The parser includes an indentation-sensitive lexer — the hardest part of YAML parsing. Gaps are formatter (Layer 5), analyzer (Layer 6), and service integration (Layer 7). Layers 5 and 6 can be built in parallel.

5. **JSON is the right reference implementation.** Simplest end-to-end integration: no embedded languages, no semantic model, minimal formatting options, straightforward lint rules. CSS has embedded language complexity; JS has module graphs, type inference, and JSX. JSON's scope matches YAML's: data serialization format with comments and basic structure.

**Crystallizes:** The `references/biome/` directory stabilizes into something that looks like the **biome-language-integration skill's** reference material. Not formalized as a skill yet — but recognizably reusable.

### Phase 3: Spec Writing

**Build:**
- `lang-spec-writer` agent (or use doc-coauthoring directly and see where it falls short)

**Discover:**
- What a good biome language-support spec looks like — the first spec IS the template
- What the evaluator-optimizer loop needs to check (the rubric emerges from the first real evaluation)
- Where the spec writer needs to pull in research output — this reveals the **data flow** between agents

**Discovered (from YAML spec writing):**

1. **Architecture notes are a prerequisite, not optional.** The extension contract (`references/biome/extension-contract.md`) is universal — it describes what any language needs. But the spec writer also needs language-specific state: what the parser already handles, what concerns differ from JSON, where the gaps are. This drove creation of `references/yaml/architecture-notes.md` as a mandatory input. The Phase 2 growth path listed it but didn't create it. Future languages should create architecture notes as the final Phase 2 deliverable, not defer to Phase 3.

2. **Spec organizes by layer, then by phase within each layer.** The natural structure is: Layer 5 (Formatter) → Layer 6 (Analyzer) → Layer 7 (Service Integration), with each layer subdivided into phases (MVP → Advanced → Edge Cases for formatter; Tier 1 → Tier 2 → Tier 3 for analyzer). This mirrors both the extension contract's layer ordering and the research report's tier ranking. The template is now validated.

3. **Monolithic spec is correct for a single language.** At ~700 lines, the YAML spec covers all three missing layers in one document. Splitting into per-layer specs would lose the cross-cutting sections (implementation order, testing strategy, open questions) that tie the layers together. A single document also makes it easier for an implementer to understand the full scope. Per-layer specs might make sense if layers were being built by different teams, but that's not the case here.

4. **Opus is the right model for spec writing.** The task requires synthesizing three documents (521 + 392 + 150 lines), maintaining internal consistency across 22 rules and 10+ format options, and making judgment calls about priorities and edge cases. This is a one-time artifact per language where quality matters more than speed. Sonnet could produce the structure but would miss edge case analysis and cross-reference consistency.

5. **The spec-as-template pattern works.** The YAML spec's structure (Overview → Prerequisites → Layer 5 → Layer 6 → Layer 7 → Implementation Order → Testing → Open Questions) is reusable for any language. The per-rule table format (name, category, severity, what it checks, config, edge cases, reference, target file) is the minimum information an implementer needs. Future specs should follow this exact structure with language-specific content.

6. **Data flow is strictly sequential: extractor → analyst → spec-writer.** No shared state between agents. The spec writer reads files produced by the other two, never invokes them. Manual sequencing (run Phase 1, run Phase 2, run Phase 3) is sufficient — no orchestrating command needed yet. This may change when we add Phase 4 (implementation) which could benefit from a `/lang-dev` command that sequences all phases.

**Crystallizes:** The first spec becomes a template. The rubric becomes the seed of the **spec-methodology skill**. The data flow between extractor → analyst → spec-writer reveals that manual sequencing is fine — no orchestrating command needed yet.

### Phase 4: Implementation

**Use:** The existing cst-parser-engineer, biome-lint-engineer, ir-formatter-engineer — enhanced with `references/yaml/` directories containing spec output.

**Discover:**
- Where the existing agents fall short (missing YAML grammar knowledge? missing biome-specific patterns?)
- Mistakes that a hook could have prevented ("I started implementing before the spec covered this area")
- Whether the spec was detailed enough to guide implementation

**Discovered (from YAML implementation):**

1. **Three separate registration systems, not one.** Adding a language to Biome requires registering it in three independent codegen/macro systems: (a) `xtask/codegen` analyzer codegen (generates `registry.rs`, `lint.rs`, `build.rs` for the new analyzer crate), (b) `xtask/codegen` configuration codegen (generates the unified `Rules` enum in `rules.rs`), and (c) `biome_configuration_macros` proc macro (generates group structs like `Suspicious` with `recommended_rules_as_filters()` at compile time). Missing any one causes rules to silently not fire — no error, no warning, just absent from the enabled list. This was the hardest bug: `noDuplicateKeys` passed unit tests but was invisible to `biome lint` because the proc macro didn't know about `biome_yaml_analyze`. A future hook should verify all three registration points.

2. **The spec scoped correctly but the extension contract was the real implementation guide.** The spec told us *what* to build (which rules, which format options, which layers). But *how* to build it required deep reading of the JSON reference implementation at every step. The extension contract (`references/biome/extension-contract.md`) was necessary but insufficient — it describes the trait boundaries, not the implementation patterns within each trait. The actual implementation pattern (how `ServiceLanguage` methods compose, how `AnalyzerVisitorBuilder::finish()` collects rules, how the proc macro feeds `recommended_rules_as_filters()`) had to be reverse-engineered from `json.rs` each time. Future languages need less spec and more annotated reference walkthrough.

3. **Formatter is harder than analyzer, by a wide margin.** The analyzer was straightforward: declare rule → query AST → collect state → emit diagnostic. One rule (`noDuplicateKeys`) took ~100 lines and worked on the first try at the unit level. The formatter required understanding biome's IR primitives (`block_indent` vs `indent` vs `hard_line_break` vs `soft_line_break`), YAML's semantic indentation (spaces-only default, compact notation for `- key: value`), and several iterations of debugging where output collapsed or mis-indented structure. Formatter bugs produce "wrong output" not "error messages," making them harder to diagnose. The ir-formatter-engineer agent would need biome-specific IR knowledge injected to be useful.

4. **Existing specialized agents were not used.** Implementation was done entirely by the main conversation agent. The biome-lint-engineer and ir-formatter-engineer agents lack the biome-specific context needed: the extension contract, the YAML syntax tree structure, the registration system details. To be useful, they'd need either (a) access to `references/biome/` and `references/yaml/` directories in their prompts, or (b) to be invoked only for narrow subtasks with full context passed in. As-is, they're too generic for biome integration work.

5. **End-to-end testing caught what unit tests missed.** The `noDuplicateKeys` rule passed its unit test (quick_test in `lib.rs`) on the first attempt. But `biome lint test.yaml` reported zero diagnostics. The gap was the registration system (discovery #1 above). Similarly, the formatter's `smoke_test` passed but `biome format complex.yaml` destroyed indentation. The lesson: every stage needs an end-to-end verification step, not just `cargo test -p <crate>`. A checklist item: "run `biome check` on a real file before declaring a stage complete."

6. **Language-specific defaults must diverge from Biome globals.** YAML requires spaces for indentation (the spec forbids tabs). Biome's global default is tabs. This caused the formatter to output tab-indented YAML that violated the spec. The fix was a one-line change (`unwrap_or(IndentStyle::Space)` instead of `unwrap_or_default()`), but the spec didn't flag it. Future specs should include a "defaults that differ from Biome globals" section.

7. **Stages 1-2 are truly parallel; Stages 3-4 are serial.** The formatter crate and analyzer crate have zero compile-time dependencies on each other. Configuration (Stage 3) depends on both (it references format option types and rule metadata). Service integration (Stage 4) depends on all three. The plan's stage ordering was correct.

8. **Snapshot tests are configuration surface area guards.** Adding `yaml` as a valid config key caused 2 existing snapshot tests to fail in `biome_configuration`. This is good — it proves the test suite catches when new languages widen the configuration surface. The fix was accepting updated snapshots. Future languages should expect this and not be surprised.

9. **Session recovery worked because of plan persistence.** The implementation spanned 3 sessions due to context compaction. Each time, the cached plan file (`.claude/plans/`) plus the on-disk plan (`kb/tasks/`) allowed seamless continuation. This validates the "plan persistence" process issue (documented below). Without these files, each session would have required re-explaining the full implementation strategy.

**Crystallizes:** The three-registration-system problem is the strongest candidate for a **pre-implementation hook** — it's non-obvious, silent on failure, and will recur for every new language. The end-to-end testing requirement is the second candidate. The specialized agents need biome-specific knowledge injection before they're useful for this workflow.

### Phase 5: Review + Retrospective

**Build:** `lang-code-reviewer` agent (or enhance existing agents with review checklists).

**Discover:** The confidence scoring threshold, which conventions matter most, what false positives look like.

**Crystallizes:** The review agent's design is informed by real review experience, not theoretical checklists.

### Phase 6: Second Language (the real test)

**Run the same process for shell/bash (or whichever is next).**

**Discover:**
- Which parts of the process were truly reusable vs. accidentally YAML-specific
- Which reference documents need parameterization
- Whether the command structure actually works for a different language or needs redesign

**Crystallizes:** This is where the **plugin manifest** earns its existence. There are now enough components that bundling them makes sense. The `plugin.json`, the `${CLAUDE_PLUGIN_ROOT}` paths, the namespaced commands — all justified by actual reuse.

## How the evolution gets captured

| Mechanism | What it captures | When it updates |
|-----------|-----------------|----------------|
| **Agent definitions** (`.claude/agents/*.md`) | What each role needs to know and do | After each phase reveals gaps or refinements |
| **References directories** (`references/{language}/`, `references/biome/`) | Language-specific and biome-specific knowledge | Continuously as research and implementation produce findings |
| **Skill definitions** (`skills/*/SKILL.md`) | Stabilized patterns that have been used more than once | When a pattern is observed being repeated |
| **Command definitions** (`.claude/commands/*.md`) | Workflows that were initially manual | When a manual sequence becomes routine enough to formalize |
| **Hook definitions** | Rules that were initially "we should remember to..." | After a mistake makes the rule's value concrete |
| **kb/tasks/ documents** | Decision rationale, options considered, lessons learned | At phase boundaries — the retrospective layer |
| **Git history** | The actual evolution of every file above | Continuously |
| **CLAUDE.md** | Current state — what exists, what phase we're in, what's next | At phase transitions |

The **kb/tasks/ directory** is the retrospective layer — it captures *why* things evolved the way they did. The agent/skill/command files capture *what* exists now. Git captures *how* it got there.

## Growth path: E → C

```
Phase 1    agents/lang-feature-extractor.md
           commands/lang-research.md
           references/yaml/tools.md
           ─────────────────────────────────────── minimal viable toolkit

Phase 2    agents/lang-architecture-analyst.md
           references/biome/extension-contract.md
           references/yaml/architecture-notes.md
           ─────────────────────────────────────── two-agent system

Phase 3    agents/lang-spec-writer.md
           references/yaml/architecture-notes.md   ← prerequisite discovered
           references/yaml/yaml-support-spec.md    ← first spec = template
           ─────────────────────────────────────── spec capability added

Phase 4    biome_yaml_formatter, biome_yaml_analyze crates built
           3 registration systems identified as critical gotcha
           specialized agents NOT used (too generic for biome work)
           end-to-end testing > unit testing for integration bugs
           ─────────────────────────────────────── implementation complete, hooks identified

Phase 5    agents/lang-code-reviewer.md
           kb/tasks/yaml-retrospective.md           ← lessons learned
           ─────────────────────────────────────── review capability added

Phase 6    skills/feature-comparison/SKILL.md       ← crystallized from pattern
(2nd lang) skills/biome-integration/SKILL.md        ← crystallized from reuse
           skills/spec-methodology/SKILL.md         ← crystallized from rubric
           hooks/pre-implementation-check            ← crystallized from pain
           commands/lang-dev.md                      ← orchestrator emerges
           plugin.json                               ← bundle justified
           ─────────────────────────────────────── full plugin (Option C)
```

## Process issue: Plan file persistence

**Observed in:** Phases 1, 2, 3, and 4 (every phase).

**Problem:** The agent consistently needs an explicit reminder to write the plan to `kb/tasks/phase{N}-*.md` before starting implementation. The plan exists in conversation context but doesn't get persisted to disk as a first action.

**Impact:** Creates friction and wastes a conversational round-trip on every phase transition.

**Fix applied:** Add a directive to CLAUDE.md requiring plan file creation as the first action when entering any new phase. The directive should be:

```
## Workflow: Plan persistence
When starting a new phase of work (planning or implementation), the FIRST action
is to write the plan to `kb/tasks/phase{N}-{description}.md`. Do not begin
implementation before the plan file exists on disk. This is non-negotiable.
```

**Why this keeps happening:** The agent optimizes for getting to implementation quickly. Plan writing feels like overhead rather than a deliverable. The fix reframes it: the plan file IS a deliverable, not a side effect.

## Process issue: Conversation compaction timing

**Observed in:** Phase 4 (implementation), but applies to all phases.

**Problem:** By the time implementation starts, the conversation is heavy with exploration results, reference code snippets, and plan iterations. Only a fraction is needed for actual implementation.

**Optimal compaction points:**
1. **Between plan approval and implementation start** — plan is on disk in `kb/tasks/`, reference files are in `references/`. Safe to compress.
2. **Between stages within implementation** — after each stage compiles, the debug context (which imports failed, what was tried) is no longer needed. Stage output is on disk (compiled crates).
3. **Never mid-stage** — compacting while debugging a compilation error loses critical context.

**What enables safe compaction:** Durable artifacts on disk. The less that lives only in conversation context, the safer compaction is.

**Practical approach for next language:**
- Phases 1-3 (research → spec): one session, commit artifacts
- **Compact here** — fresh session for Phase 4
- Phase 4: start fresh, read plan from `kb/tasks/`, read reference files on demand
- Between stages: compact if context gets large, since each stage is independently verifiable

**Key principle:** Compaction is safe when all decision-relevant information exists on disk, not just in conversation memory.

## Process issue: Silent registration failures

**Observed in:** Phase 4 (implementation), Stage 2 and Stage 5.

**Problem:** Biome has three independent systems that must all know about a new language's analyzer for lint rules to fire at runtime:

1. `xtask/codegen` analyzer — generates `registry.rs`, `lint.rs`, group files for the analyzer crate
2. `xtask/codegen` configuration — generates the `Rules` enum in `rules.rs` with the rule name and group mapping
3. `biome_configuration_macros` proc macro — generates group structs (e.g., `Suspicious`) with `recommended_rules_as_filters()` by visiting all language analyzer registries at compile time

Missing #1 or #2 causes a compile error. Missing #3 causes **silent failure** — the rule compiles, passes unit tests, appears in the `Rules` enum, but is never included in the enabled rules at runtime. The proc macro generates a `Suspicious` struct that doesn't include the YAML rule because `biome_yaml_analyze::visit_registry()` was never called.

**Impact:** 2+ hours of debugging. The symptom ("213 rules enabled, noDuplicateKeys not among them") pointed everywhere except the proc macro crate. Unit tests passed. The configuration codegen showed the rule registered. Only by tracing the `recommended_rules_as_filters()` call chain to the proc macro's `collect_lint_rules()` function was the gap found.

**Fix applied:** Added `biome_yaml_analyze` dependency and `visit_registry` call to `biome_configuration_macros/src/lib.rs` and `visitors.rs`.

**Hook candidate:** A post-codegen or pre-build hook that verifies: for every `biome_{lang}_analyze` crate that exists, there must be a corresponding `visit_registry` call in all three locations. This is mechanical to check and would have saved the debugging entirely.

**Why this will recur:** Every new language needs the same three registrations. The first two are somewhat discoverable (codegen fails or rules don't appear). The third is invisible until you run end-to-end tests and notice the rule doesn't fire.

## Crystallization heuristics

- **Skills** crystallize when a pattern is used a second time
- **Hooks** crystallize when a mistake happens once
- **Commands** crystallize when a manual sequence is repeated enough to formalize
- **The plugin manifest** crystallizes when there are enough components to bundle
- None of these are speculative — they are all responses to observed needs

## Revised assessment

**Option E: Incremental toward a generalized plugin.** Build using YAML as the first language. Use language-agnostic naming. Let each phase inform the next. The plugin-dev toolkit's `${CLAUDE_PLUGIN_ROOT}` pattern and the skill-creator's progressive disclosure (`references/` directories per language) are the two most important design patterns — they are exactly how a reusable toolkit gets parameterized.
