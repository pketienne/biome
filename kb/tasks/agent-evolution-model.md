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

**Discovered (from YAML retrospective):**

1. **Retrospective happened organically, not via a formal agent.** The plan called for a `lang-code-reviewer` agent. In practice, the retrospective was triggered by a single user question ("what did we learn for the agent-evolution-model?") after implementation was done. No dedicated agent was needed — the main conversation agent had full context and produced the 9 Phase 4 findings plus 3 process issues in one pass. This suggests Phase 5 needs a **prompt/checklist**, not an agent. A checklist like "after implementation, review: what broke, what was harder than expected, what the spec missed, what would save time next language" would capture the same value at lower cost.

2. **The agent-evolution-model document IS the retrospective.** The plan predicted a separate `kb/tasks/yaml-retrospective.md`. Instead, findings were written directly into `agent-evolution-model.md` under each phase's "Discovered" section. This is better — it keeps observations co-located with the phase they inform, rather than in a separate document that would need cross-referencing. A separate retrospective file is redundant when the evolution model already has per-phase discovery sections.

3. **No code review agent was needed for the first language.** The implementation was iterative: write code → compile → test end-to-end → fix → repeat. Each fix was immediately verified. A code reviewer would have had nothing to catch that wasn't already caught by the compile-test-fix loop. The code reviewer agent may become valuable when (a) multiple contributors work on the same language, (b) the implementation is large enough that the implementer loses track of cross-cutting concerns, or (c) conventions have been established and need enforcement. None of these applied to YAML-first.

4. **The three process issues are retrospective artifacts, not implementation artifacts.** Plan persistence, compaction timing, and silent registration failures were all written *after* implementation, reflecting on the process rather than discovered while coding. They live in standalone sections of this document rather than under Phase 4's numbered findings, which is the right placement — they're cross-cutting process observations, not phase-specific discoveries.

5. **Phase 5 should be a checkpoint, not a phase.** The other phases produce concrete artifacts (agents, specs, crates). Phase 5 produces observations that get folded into this document. It doesn't need its own timeline slot — it's a 15-minute activity after Phase 4 completes. Renaming it from "Phase 5" to "Post-implementation checkpoint" and embedding its checklist into the Phase 4 completion criteria would better reflect reality.

6. **The lang code reviewer's original intent was a post-implementation quality gate, not a PR reviewer.** It was designed to (a) check implementation against the spec — are all specified formatter cases handled, all Tier 1 lint rules implemented, all service capability methods wired? (b) check implementation against biome conventions — does the code follow patterns established by JSON/CSS? (c) use confidence scoring (from the `code-review` reference pattern) to rank findings and filter false positives. This was modeled on 4 parallel specialist agents (style, logic, security, performance) each producing scored findings. In practice, the tight compile-test-fix loop made this unnecessary for a single implementer on the first language. The agent may become relevant when: reviewing a PR from someone else, auditing a second language's implementation against patterns established by the first, or when the implementation is too large for one session's context to hold all cross-cutting concerns.

**Crystallizes:** Phase 5 collapses into a **post-implementation checklist** rather than a separate phase with its own agent. The checklist is: (1) update agent-evolution-model.md with discoveries, (2) identify process issues, (3) note hook candidates, (4) flag what the spec missed. This checklist could live in `CLAUDE.md` or as a step in a future `/lang-dev` command.

### Phase 6: Methodology Audit

**Audit the overall approach used to complete the first language.** This is distinct from Phase 5's retrospective (which captures implementation-level learnings). The audit evaluates the *methodology itself*: agents, orchestration, commands, reference materials, tools/utilities, documentation practices, and process structure.

**Scope:**

| Dimension | What to evaluate |
|-----------|-----------------|
| **Agents** | Which agents were built? Which were used? Which were useful? Which were unused and why? (See engineering agent analysis above.) Are the agent definitions accurate to what was actually needed? |
| **Commands** | Does `/lang-research` work as designed? Are the proposed commands for Phases 2-5 worth building? What command structure would the second language actually benefit from? |
| **Orchestration** | Was the phase sequencing correct? Were parallelization opportunities exploited? Would a single orchestrating command (`/lang-dev`) be better than per-phase commands? |
| **Reference materials** | Are `references/biome/extension-contract.md`, `references/yaml/architecture-notes.md`, and `references/yaml/yaml-support-spec.md` reusable templates or one-off documents? What needs to be parameterized for the next language? |
| **Tools / utilities** | Did the development environment support the work? (See open question #3 on container impact.) Were the right cargo tools available? Was codegen reliable? |
| **Documentation** | Is `kb/tasks/` the right structure? Is the naming convention consistent? Are the plan files useful artifacts or just process overhead? Is the agent-evolution-model document maintainable at its current size? |
| **Deterministic gates** | Were plans captured? Were summaries captured? Where did the process break down due to missing enforcement? (See gate analysis above.) |
| **Testing** | Is the testing gap acceptable for this stage? What testing should be mandatory before the phase is considered complete? (See open question #2.) |
| **Debugging** | Were debugging practices adequate? What systematic approaches were missed? (See open question #4.) |

**Discover:**
- Which parts of the methodology are load-bearing (remove them and the process breaks) vs. ceremonial (they exist but don't contribute)
- Whether the overhead of the methodology (agents, commands, reference docs, plan files) pays for itself in reduced errors and faster work, or whether a simpler approach (just start implementing, read reference code when stuck) would have produced the same result faster
- What the minimum viable methodology is for the second language — the smallest set of agents + commands + references + gates that would produce equivalent quality

**Crystallizes:** The audit produces a **methodology scorecard** — a document that rates each component (agent, command, reference doc, gate) on: (a) was it used, (b) did it prevent an error or save time, (c) would its absence have been noticed. Components that score low on all three are candidates for removal. Components that score high become the core of the reusable toolkit.

### Phase 7: Second Language (the real test)

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

Phase 5    no new agent needed (checklist, not agent)
           findings folded into agent-evolution-model.md directly
           Phase 5 collapses into post-implementation checkpoint
           ─────────────────────────────────────── retrospective complete

Phase 6    methodology scorecard produced
(audit)    each component rated: used? prevented errors? missed if absent?
           deterministic gates formalized
           testing gap, debugging practices, container impact assessed
           commands for phases 2-5 evaluated
           ─────────────────────────────────────── methodology validated/pruned

Phase 7    skills/feature-comparison/SKILL.md       ← crystallized from pattern
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

## Engineering agent analysis: cst-parser-engineer, biome-lint-engineer, ir-formatter-engineer

### What they have

Each agent has two layers of knowledge:
1. **Generic domain expertise** in its system prompt — parser theory, IR formatting algorithms, lint rule patterns
2. **Biome-specific CONTRIBUTING.md** referenced via `@` — step-by-step instructions for adding a parser (415 lines), formatter (337 lines), or lint rule (1624 lines) to biome

The CONTRIBUTING docs are genuinely valuable. They explain codegen steps, testing patterns, file naming conventions, and biome-specific macros (`declare_lint_rule!`, `FormatNodeRule`, grammar DSL). This is the right kind of knowledge to inject into an agent.

### What they're missing

Five categories of knowledge that were needed during Phase 4 but aren't in any agent:

1. **Language-specific syntax tree** — What nodes exist (`YamlBlockMapImplicitEntry` vs `YamlBlockMapExplicitEntry`)? What fields do they have? What's the AST shape for nested mappings vs sequences? The formatter agent can't format nodes it doesn't know about. This knowledge lives in the `.ungram` file and generated syntax types, not in any agent prompt.

2. **The extension contract** — How biome's 7 layers connect. Which traits to implement for a new language. How `ServiceLanguage` methods compose. This lives in `references/biome/extension-contract.md` but isn't referenced by any agent.

3. **Cross-crate registration** — The three-registration-system problem. None of the three agents spans more than one crate. The biome-lint-engineer knows how to write a rule but not how to register it across codegen, configuration, and the proc macro.

4. **Reference implementation patterns** — Not the abstract "here's how formatting works" but the concrete "JSON uses `block_indent` for nested objects, `soft_block_indent` for flow objects, and always adds a trailing newline." The CONTRIBUTING docs explain the framework; the reference implementations show how to use it idiomatically.

5. **End-to-end integration context** — The hard work during Phase 4 was wiring 4+ crates together, not implementing individual nodes or rules. The agents are scoped to single-crate work.

### The fundamental mismatch

These agents are designed as **narrow specialists** (format this node, write this lint rule, add this grammar production). But new-language integration work is **cross-cutting** (wire everything together across formatter + analyzer + configuration + service crates). The narrow tasks they're good at are the *easy* parts of adding a new language. The hard parts — registration, service wiring, debugging silent failures, choosing the right IR primitives for a language's semantics — are outside their scope.

Put differently: if you already know *what* to implement (which nodes, which rules, which traits), the individual implementations are straightforward enough to not need a specialist agent. And if you *don't* know what to implement, the specialist agents can't help because they lack the integration context to tell you.

### When they would become valuable

The agents' value increases along three axes:

1. **Repetition within a language** — After the first 3 formatter nodes are implemented, the remaining 48 follow a pattern. After the first lint rule, subsequent rules are templated. At this scale, a specialist agent doing parallel node/rule implementation saves real time. Phase 4 didn't reach this scale (1 rule, ~10 formatter nodes).

2. **Second language onward** — Once patterns are established from YAML, the agents can be enhanced with "here's how YAML did it, follow the same pattern for {language}." The reference implementation shifts from JSON (which requires reverse-engineering) to YAML (which was built with the agent workflow and is well-documented).

3. **Multiple contributors** — If different people (or parallel sessions) work on formatter vs. analyzer vs. parser, the agents' narrow scope becomes a feature: each contributor gets focused guidance without cross-crate noise.

None of these conditions applied during YAML-first. All three may apply during the second language.

### Options

**Option 1: Inject context, keep agents narrow (Recommended for next language)**

Add `references/biome/extension-contract.md` and `references/{language}/architecture-notes.md` as `@` references to each agent. Don't broaden their scope — let them remain formatter/analyzer/parser specialists, but with integration awareness. Also add the `.ungram` file for the target language so they know the syntax tree shape.

Cost: ~3 additional `@` references per agent. Risk: context bloat (~1500 lines added to each agent's prompt). Mitigation: only inject when working on a specific language, not at agent definition time.

**Option 2: Use them only for batch narrow tasks**

Don't use them during integration scaffolding (Stages 3-4 of the plan). Use them *after* scaffolding is complete, for batch work: "implement these 20 formatter nodes" or "write these 5 lint rules." Pass the first completed example as in-context reference.

Cost: zero changes to agent definitions. Requires manual coordination (know when to switch from main-conversation to delegated work). This is how they'd be used in practice regardless of other changes.

**Option 3: Replace with a single `biome-language-engineer` agent**

Merge all three into one agent that understands parser + formatter + analyzer + integration. Reference all three CONTRIBUTING docs plus the extension contract.

Cost: one broader agent to maintain. Risk: prompt becomes too large and unfocused. The breadth-vs-depth tradeoff works against this — a 3000-line system prompt dilutes each domain's guidance.

**Option 4: Demote to documentation, remove agent wrappers**

The CONTRIBUTING docs are the real value. The agent wrapper adds an indirection layer that provides minimal benefit when the main conversation agent already has tool access and can read the same docs. Remove the agents; add `references/biome/contributing-analyzer.md`, `references/biome/contributing-formatter.md`, `references/biome/contributing-parser.md` as reference documents that any agent (or the main conversation) can read on demand.

Cost: lose the ability to delegate narrow tasks in parallel. Gain: simpler system, no maintenance of agent definitions that don't get used. Honest about the current value proposition.

**Option 5: Keep as-is, defer decision to second language**

The agents exist and don't hurt anything. Their value proposition is untested because the conditions for their value (repetition, multiple contributors, established patterns) haven't occurred yet. Wait until the second language to see if Option 1 or Option 2 naturally emerges.

Cost: zero. Risk: they remain unused artifacts that create false confidence in the toolkit's capability.

### Evaluation

**Current utility: low.** During Phase 4 (the only implementation phase so far), all work was done by the main conversation agent. The specialist agents were never invoked. The CONTRIBUTING docs they reference were read directly when needed.

**Projected utility for second language: medium.** If the second language involves >20 formatter nodes or >5 lint rules, Option 2 (batch narrow tasks) would save time. The agents would act as parallelizable workers following established patterns.

**Projected utility at scale (plugin used by others): high.** If this becomes a toolkit others use, the agents provide guided entry points for contributors who don't have full codebase context. A new contributor told "use biome-lint-engineer to add this rule" gets CONTRIBUTING.md + domain expertise + tool access in one package.

**Recommendation:** Option 5 (defer) for now. The honest answer is we don't know if they're worth investing in until the second language reveals whether the narrow-specialist pattern or the cross-cutting-integration pattern dominates. If the second language's hard parts are again integration (likely), Option 4 (demote to docs) is the right call. If the hard parts shift to volume (many nodes, many rules), Option 2 (batch tasks) justifies keeping them.

## Crystallization heuristics

- **Skills** crystallize when a pattern is used a second time
- **Hooks** crystallize when a mistake happens once
- **Commands** crystallize when a manual sequence is repeated enough to formalize
- **The plugin manifest** crystallizes when there are enough components to bundle
- None of these are speculative — they are all responses to observed needs

## Command structure and deterministic gates

### Current state

One command exists: `/lang-research` — it orchestrates Phase 1 (feature extraction). It lives at `.claude/commands/lang-research.md` and is fully functional: it loads the tool inventory, launches parallel extraction agents, synthesizes results, and suggests saving the report. No commands exist for Phases 2-5.

Plans were stored in two places during development:
- **Internal:** `/home/vscode/.claude/plans/` — Claude Code's plan mode writes here automatically. The file (`cached-prancing-gem.md`) survives compaction within a session and persists across sessions in the same workspace. It is re-injected into conversation context at session start (visible in the system prompt). However, this is a single-file store — entering plan mode for a new task overwrites the previous plan. It is **not** a reliable archive, but it **is** referenceable after compaction within the same planning scope.
- **Manual:** `kb/tasks/phase{N}-*.md` — these were created only when the user explicitly asked for plan persistence. They are durable git-tracked artifacts. They survived across all sessions.

The internal plan store worked for session recovery (Phase 4 discovery #9), but the `kb/tasks/` files are the only reliable archive. The internal store is a working buffer, not a record.

### Where `/lang-research` fits

`/lang-research` IS Phase 1 — it is the Phase 1 trigger. It doesn't precede Phase 1; it embodies it. The command's internal phases (setup → clarifying questions → extraction → synthesis → completion) are sub-steps within the overall Phase 1.

### Phase execution order (actual, from git history)

All phases ran **sequentially** despite some being parallelizable:

```
Phase 1 (Feature Extraction)    → commit afb563fc06 (2026-02-10)
Phase 2 (Architecture Analysis) → commit b22b91c6fe (2026-02-10)
Phase 3 (Spec Writing)          → commit 8ce9ab365e (2026-02-10/11)
Phase 4 (Implementation)        → commit 91b7ef0b81 (2026-02-11)
Phase 5 (Retrospective)         → commit 1eb4cc28ad (2026-02-11)
```

Phases 1 and 2 are **independent and parallelizable** (the evolution model confirms: "Extractor and analyst are fully independent. No shared inputs, state, or outputs."). They were run sequentially because a single conversation agent handled both.

Phases 3 and 4 are **strictly sequential** — Phase 3 (spec) requires Phase 1 and 2 outputs; Phase 4 (implementation) requires the Phase 3 spec. These never ran concurrently.

Within Phase 4, Stages 1 (formatter) and 2 (analyzer) are parallelizable but were run sequentially in one session.

### Proposed command structure

| Command | Phase | Trigger | What it does |
|---------|-------|---------|-------------|
| `/lang-research <language>` | 1 | User invocation | Feature extraction (already exists) |
| `/lang-architecture <language>` | 2 | User invocation | Architecture analysis against biome internals |
| `/lang-spec <language>` | 3 | User invocation | Spec writing from Phase 1+2 outputs |
| `/lang-implement <language>` | 4 | User invocation | Implementation from spec |
| `/lang-review <language>` | 5 | User invocation | Post-implementation checkpoint |
| `/lang-audit <language>` | 6 | User invocation | Methodology audit — evaluate agents, commands, gates, docs |

Each command would follow the same internal structure:

```
1. GATE: Capture plan to kb/tasks/phase{N}-{description}.md
2. GATE: Verify prerequisites (previous phase artifacts exist)
3. Execute phase work
4. GATE: Capture phase summary to kb/tasks/phase{N}-{description}-summary.md
5. GATE: Update agent-evolution-model.md with discoveries
```

### Deterministic gates (hard triggers)

The plan persistence problem recurred in every phase because there was no enforcement mechanism. The following gates should be **deterministic** — they execute unconditionally, not when the agent "remembers" to.

**Gate 1: Plan capture (start of phase)**

Before any implementation work begins, the command must:
1. Write the plan to `kb/tasks/phase{N}-{description}.md`
2. Verify the file exists on disk
3. Only then proceed to execution

This is enforceable in a command definition by making the plan-writing step a prerequisite that blocks subsequent steps. In a command `.md` file, this looks like:

```markdown
## Step 1: Plan Capture (BLOCKING)
1. Generate the implementation plan for this phase.
2. Write it to `kb/tasks/phase{N}-{description}.md`.
3. Read the file back to confirm it was written.
4. If the file does not exist or is empty, STOP and report the failure.
5. Only after confirmation, proceed to Step 2.
```

**Gate 2: Prerequisite check (start of phase)**

Before starting Phase N, verify Phase N-1 artifacts exist:
- Phase 2 requires: `references/{language}/feature-research-report.md`
- Phase 3 requires: `references/biome/extension-contract.md` + `references/{language}/architecture-notes.md`
- Phase 4 requires: `references/{language}/*-support-spec.md`
- Phase 5 requires: compiled crates (check via `cargo build -p biome_{language}_formatter -p biome_{language}_analyze`)

**Gate 3: Phase summary (end of phase)**

After phase work completes, the command must:
1. Write a summary to `kb/tasks/phase{N}-{description}-summary.md` containing:
   - **Completed work:** What was built, with file paths
   - **Planned but deferred work:** Items from the plan that weren't implemented
   - **Discovered work:** New tasks found during execution that weren't in the original plan
   - **Artifacts produced:** Files created or modified, committed or uncommitted
2. Update `kb/tasks/agent-evolution-model.md` "Discovered" section for this phase

**Gate 4: Test status (end of implementation phase)**

After Phase 4, specifically capture:
- Which test types exist (inline, snapshot, fixture, e2e)
- Which test types are missing
- A checklist of test files that need to be created

### Documentation directory convention

All documentation lives in `kb/tasks/` with this naming:

```
kb/tasks/
├── agent-evolution-model.md          # Living document, updated every phase
├── agent-leverage-options.md          # Decision record (static after Phase 1)
├── agent-design-references.md         # Reference patterns (static after Phase 1)
├── phase1-feature-extraction.md       # Phase 1 plan
├── phase1-feature-extraction-summary.md  # Phase 1 outcomes (NEW)
├── phase2-architecture-analysis.md    # Phase 2 plan (exists as phase2-architecture-analysis-plan.md)
├── phase2-architecture-analysis-summary.md  # Phase 2 outcomes (NEW)
├── phase3-spec-writing.md             # Phase 3 plan (exists as phase3-spec-writing-plan.md)
├── phase3-spec-writing-summary.md     # Phase 3 outcomes (NEW)
├── phase4-implementation.md           # Phase 4 plan (exists as phase4-implementation-plan.md)
├── phase4-implementation-summary.md   # Phase 4 outcomes (NEW)
└── phase5-review-summary.md           # Phase 5 checkpoint (NEW)
```

Convention: `phase{N}-{description}.md` for plans, `phase{N}-{description}-summary.md` for outcomes. No `-plan` suffix needed since the phase number already implies it's a plan.

Note: existing files use inconsistent naming (`phase-1-feature-extraction-toolkit.md` vs `phase2-architecture-analysis-plan.md` vs `phase4-implementation-plan.md`). For the second language, standardize on the convention above.

### Uncaptured work: testing gap

Phase 4 Stage 5 (Tests and polish) was planned but **not implemented**. The current state:

**What exists:**
- 1 inline smoke test in `biome_yaml_formatter/src/lib.rs` (formats `"key: value\n"`)
- 1 inline quick_test in `biome_yaml_analyze/src/lib.rs` (detects duplicate keys)

**What's missing (per the Phase 4 plan and the JSON reference implementation):**

Formatter tests:
- [ ] `crates/biome_yaml_formatter/tests/spec_tests.rs` — test harness with `gen_tests!` macro
- [ ] `crates/biome_yaml_formatter/tests/spec_test.rs` — test implementation
- [ ] `crates/biome_yaml_formatter/tests/language.rs` — language test helper
- [ ] `crates/biome_yaml_formatter/tests/quick_test.rs` — ad-hoc testing
- [ ] `crates/biome_yaml_formatter/tests/specs/yaml/` — fixture directory with 10-15 `.yaml` + `.snap` pairs (simple_mapping, nested_mapping, simple_sequence, nested_sequence, mixed, comments, empty_document, flow_style, scalars, multi_document)

Analyzer tests:
- [ ] `crates/biome_yaml_analyze/tests/spec_tests.rs` — test harness
- [ ] `crates/biome_yaml_analyze/tests/specs/suspicious/noDuplicateKeys/valid.yaml`
- [ ] `crates/biome_yaml_analyze/tests/specs/suspicious/noDuplicateKeys/invalid.yaml` + `.snap`
- [ ] Suppression comment tests

Integration tests:
- [ ] CLI integration tests (format/lint/check YAML files)
- [ ] Configuration resolution tests
- [ ] File detection tests

The JSON reference has ~504 lines of test infrastructure and ~200 fixture files. The YAML implementation has 0 test files and 0 fixtures outside of inline tests.

This is the largest uncaptured work item from Phase 4.

## Open questions for further exploration

### 1. Context loss and compaction optimization

**Problem:** Compaction loses debugging context, exploration results, and decision rationale. The current mitigation (persist everything to disk) works but requires discipline.

**Questions to explore:**
- Can deterministic gates (the plan/summary capture gates above) serve double duty as compaction-safe checkpoints?
- What is the minimum context that must survive compaction for each phase to resume? (Likely: the plan file + the last compile/test output)
- Should there be an explicit "compact now" gate between phases that forces artifact persistence before allowing compaction?
- Would a `kb/tasks/context-snapshot.md` file (updated at each gate) provide enough state for a fresh session to resume without the conversation history?

### 2. Test timing and parallelization

**Problem:** Testing was deferred to Phase 4 Stage 5 and then not completed. Some tests could have been written earlier and in parallel with implementation.

**Questions to explore:**
- **Inline unit tests** (smoke tests): should be written *during* each stage, not deferred. They verify the immediate work and cost almost nothing.
- **Snapshot/fixture tests**: depend on the test harness infrastructure. The harness itself (spec_tests.rs, language.rs) could be created at the start of Stage 1, with fixtures added as each formatter node is implemented.
- **Analyzer fixture tests**: could be written in parallel with Stage 2 (analyzer implementation), since the test harness follows the same pattern.
- **End-to-end tests**: require Stage 4 (service integration) to be complete. These are inherently serial.
- **Parallelization opportunity**: formatter fixture tests and analyzer fixture tests are fully independent and could be written by separate agents simultaneously.
- When should round-trip property testing (`format(format(x)) == format(x)`) be added? This is a formatter-specific concern that could catch bugs early.

### 3. Lightweight container impact

**Problem:** Development used `erasimus/devcontainer.json` instead of the full Biome devcontainer. The full devcontainer includes `just` (task runner), `cargo-binstall`, `cargo-insta`, `tombi`, `wasm-bindgen-cli`, and `wasm-opt`.

**Questions to explore:**
- **`just` absence**: All `just` commands (`just gen-formatter`, `just gen-analyzer`, `just gen-rules`, `just test`, `just f`, `just l`, `just ready`) were unavailable. We used raw `cargo` commands instead. This meant: (a) codegen was run via `cargo run -p xtask_codegen -- formatter` instead of `just gen-formatter`, (b) formatting checks (`just f`) and linting checks (`just l`) were not run, (c) the `just ready` command (full pre-PR validation) was never executed. Impact: potentially missed formatting/linting issues in contributed code.
- **`cargo-insta` absence**: Initially missing, installed manually via `cargo install cargo-insta`. This delayed snapshot test acceptance. In the full devcontainer, it would have been available from the start.
- **Would dependencies have helped the engineering agents?** The agents themselves don't invoke `just` or `cargo-insta` — they work through the main conversation's tool access. But the full devcontainer would have made the *validation* step easier, which is where the agents' output gets checked. The agents' usefulness is gated by the ability to verify their work, and the lightweight container weakened that verification step.
- **Missing `wasm-bindgen-cli` and `wasm-opt`**: Not relevant for YAML support (no WASM build needed). No impact.
- **Missing `tombi`**: Not relevant for YAML support (TOML formatting only). No impact.

### 4. Debugging practices

**Problem:** Debugging during Phase 4 relied on `eprintln!` debug prints and manual inspection of output. The CONTRIBUTING.md documents more systematic approaches.

**Questions to explore:**
- **CONTRIBUTING.md debugging guidance**: The guide recommends `dbg!()` macro with `--show-output`, and a `debugging` cargo profile that preserves stack traces. Neither was used during Phase 4. The `debugging` profile (`cargo t --profile debugging some_test`) would have provided better stack traces when tracing the registration system bug.
- **Systematic approaches that could help:**
  - `cargo test -- --show-output` for seeing debug output from passing tests
  - `RUST_LOG` / `tracing` for structured logging during end-to-end runs
  - `cargo expand` for inspecting proc macro output — this would have directly shown the missing `visit_registry` call in `biome_configuration_macros`, cutting the 2+ hour debugging session significantly
  - `cargo test -p biome_configuration_macros` with expanded output to verify the proc macro generates expected code
- **What was actually used:** `eprintln!` statements added to `yaml.rs` and `lib.rs`, manually counting enabled rules, manually tracing call chains through source code. All debug prints were removed before commit.
- **Recommendation for future development:** Add a "debugging checklist" to the Phase 4 command that includes: (a) use `cargo expand` when proc macro behavior is unexpected, (b) use `--profile debugging` when stack traces are needed, (c) use `cargo test -- --show-output` instead of `eprintln!` for test debugging, (d) check `RUST_LOG` output for service-layer issues.
- **Code contribution standards:** Biome's CONTRIBUTING.md sets expectations for testing (`cargo insta` snapshots, doctests, quick_test patterns). The current YAML implementation meets the functional requirements but falls short on testing standards. The testing gap (documented above) should be addressed before any PR contribution.

## Revised assessment

**Option E: Incremental toward a generalized plugin.** Build using YAML as the first language. Use language-agnostic naming. Let each phase inform the next. The plugin-dev toolkit's `${CLAUDE_PLUGIN_ROOT}` pattern and the skill-creator's progressive disclosure (`references/` directories per language) are the two most important design patterns — they are exactly how a reusable toolkit gets parameterized.
