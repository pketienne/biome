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

**Problem:** The agent consistently needs an explicit reminder to write the plan to `kb/tasks/{language}/phase{N}-*.md` before starting implementation. The plan exists in conversation context but doesn't get persisted to disk as a first action.

**Impact:** Creates friction and wastes a conversational round-trip on every phase transition.

**Fix applied:** Add a directive to CLAUDE.md requiring plan file creation as the first action when entering any new phase. The directive should be:

```
## Workflow: Plan persistence
When starting a new phase of work (planning or implementation), the FIRST action
is to write the plan to `kb/tasks/{language}/phase{N}-{description}.md`. Do not begin
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
- **Manual:** `kb/tasks/{language}/phase{N}-*.md` — these were created only when the user explicitly asked for plan persistence. They are durable git-tracked artifacts. They survived across all sessions.

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
1. GATE: Capture plan to kb/tasks/{language}/phase{N}-{description}-plan.md
2. GATE: Verify prerequisites (previous phase artifacts exist)
3. Execute phase work
4. GATE: Capture phase summary to kb/tasks/{language}/phase{N}-{description}-summary.md
5. GATE: Update agent-evolution-model.md with discoveries
```

### Deterministic gates (hard triggers)

The plan persistence problem recurred in every phase because there was no enforcement mechanism. The following gates should be **deterministic** — they execute unconditionally, not when the agent "remembers" to.

**Gate 1: Plan capture (start of phase)**

Before any implementation work begins, the command must:
1. Write the plan to `kb/tasks/{language}/phase{N}-{description}.md`
2. Verify the file exists on disk
3. Only then proceed to execution

This is enforceable in a command definition by making the plan-writing step a prerequisite that blocks subsequent steps. In a command `.md` file, this looks like:

```markdown
## Step 1: Plan Capture (BLOCKING)
1. Generate the implementation plan for this phase.
2. Write it to `kb/tasks/{language}/phase{N}-{description}.md`.
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
1. Write a summary to `kb/tasks/{language}/phase{N}-{description}-summary.md` containing:
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

Cross-language methodology docs live in `kb/tasks/`. Per-language phase plans and summaries live in `kb/tasks/{language}/`.

```
kb/tasks/
├── agent-evolution-model.md              # Living document, updated every phase (cross-language)
├── agent-leverage-options.md              # Decision record (cross-language, static after Phase 1)
├── agent-design-references.md             # Reference patterns (cross-language, static after Phase 1)
├── yaml/
│   ├── phase1-feature-extraction.md       # Phase 1 plan
│   ├── phase1-feature-extraction-summary.md  # Phase 1 outcomes (NEW)
│   ├── phase2-architecture-analysis.md    # Phase 2 plan
│   ├── phase2-architecture-analysis-summary.md  # Phase 2 outcomes (NEW)
│   ├── phase3-spec-writing.md             # Phase 3 plan
│   ├── phase3-spec-writing-summary.md     # Phase 3 outcomes (NEW)
│   ├── phase4-implementation.md           # Phase 4 plan
│   ├── phase4-implementation-summary.md   # Phase 4 outcomes (NEW)
│   └── phase5-review-summary.md           # Phase 5 checkpoint (NEW)
├── shell/                                 # (future) second language
│   ├── phase1-feature-extraction.md
│   └── ...
└── ...
```

Convention: `phase{N}-{description}-plan.md` for plans, `phase{N}-{description}-summary.md` for outcomes. Language-specific files go in `kb/tasks/{language}/`, methodology files stay at `kb/tasks/`.

Note: existing YAML files use inconsistent naming (`phase-1-feature-extraction-toolkit.md` vs `phase2-architecture-analysis-plan.md` vs `phase4-implementation-plan.md`). For the second language, standardize on the convention above.

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

## Open questions — explored

### 1. Context management: `/clear` over `/compact`

**Problem:** Compaction loses debugging context, exploration results, and decision rationale. The current mitigation (persist everything to disk) works but requires discipline.

**Questions explored:**

**Can deterministic gates serve as context-reset checkpoints?**

Yes. The 4 gates defined above (plan capture, prerequisite check, phase summary, test status) each produce a disk artifact. After any gate fires, the conversation can be fully cleared because all decision-relevant state is on disk. The gate output files are the reset boundary markers.

**`/clear` is strictly better than `/compact` when gates are working.**

| | `/compact` | `/clear` |
|---|---|---|
| Context freed | Partial | 100% |
| What survives | Lossy summary (uncontrolled) | Nothing — disk artifacts only |
| Predictability | Low — you don't know what was kept | High — clean slate every time |
| Re-orientation cost | Low (some context remains) | Bounded (read plan + resumption instructions) |
| Risk of lost info | Medium — compression may drop key details | Zero if gates fired; total if they didn't |
| Timing control | Automatic (can happen mid-work) | Intentional (you choose when) |

The reasoning: if gates enforce writing everything decision-relevant to disk, then a compressed summary of the conversation is just noise consuming context window space. A clean slate + disk artifacts is more predictable and more efficient — the agent starts fresh with maximum available context for the next phase.

The only reason to prefer `/compact` is if you don't trust the gates to capture everything. That means the gates need fixing, not the context strategy.

**`/compact` is a fallback, not a strategy.** Compaction happens automatically when the context window fills up. It's useful as a safety net when you've been working without gates and need to keep going. But it's not something to plan around — it's an emergency measure. The planned approach should be:

```
[phase work] → [gate fires: artifacts to disk] → [/clear] → [new context reads artifacts]
```

Not:

```
[phase work] → [hope compaction preserves the right things] → [continue with degraded context]
```

This is exactly what happened during Phase 4: the plan was in `kb/tasks/yaml/phase4-implementation-plan.md` and `.claude/plans/`, so recovery after compaction worked (discovery #9). But it worked *despite* compaction, not *because* of it. The disk artifacts were the recovery mechanism; the compressed context was irrelevant.

**What must be on disk before `/clear`?**

| Phase boundary | Required artifacts | Where they live |
|-------|--------------------------|----------------|
| 1 → 2 | Feature research report | `references/{language}/feature-research-report.md` |
| 2 → 3 | Extension contract + architecture notes | `references/biome/extension-contract.md` + `references/{language}/architecture-notes.md` |
| 3 → 4 | Support spec + implementation plan | `references/{language}/*-support-spec.md` + `kb/tasks/{language}/phase4-*.md` |
| Mid-stage (within Phase 4) | Plan file + stage status | `kb/tasks/{language}/phase4-*.md` + compiled crates on disk |
| 4 → 5 | Phase summary with completed/deferred/discovered work | `kb/tasks/{language}/phase4-*-summary.md` |

Compile status is implicit — if `cargo build -p biome_{language}_formatter` succeeds, Stage 1 is done. The build system is a durable checkpoint that doesn't need to be in conversation context.

**The `/clear` gate pattern:**

Each phase command should end with a clear-readiness check before signaling completion:

```markdown
## Phase Completion Gate (BLOCKING)
Before this phase is complete:
1. Verify phase summary written to `kb/tasks/{language}/phase{N}-{description}-summary.md`
2. Verify all reference artifacts committed to git
3. Summary must include resumption instructions (see template below)
4. Report to user: "Phase {N} complete. Safe to /clear. To resume, read the summary."
```

**Resumption instructions template** (included in every phase summary):

```markdown
## Resumption instructions
To continue from where this phase left off:
1. Read this summary for completed/deferred/discovered work
2. Read the plan at kb/tasks/{language}/phase{N}-{description}.md for remaining stages
3. Run `cargo build -p biome_{language}_formatter` to verify Stage 1 is intact
4. Current stage: Stage 3 (configuration module)
5. Next action: create crates/biome_configuration/src/{language}.rs
```

After a `/clear`, the next message can be as simple as: "Continue from `kb/tasks/{language}/phase{N}-summary.md`" — the agent reads the file, orients, and resumes. No compressed context needed; no guessing what survived.

**When `/compact` is still appropriate:**
- Mid-stage within Phase 4, when you haven't reached a gate boundary but context is getting large
- During research phases (1-2) where the work is exploratory and artifact boundaries are less defined
- As an automatic safety net — don't disable it, just don't rely on it

**Conclusion:** Plan for `/clear` at phase boundaries. Use gates to ensure everything is on disk. Use `/compact` only as an emergency fallback mid-phase. The overhead of re-reading a plan file + resumption instructions (~30 seconds) is negligible compared to the benefit of a full, clean context window.

### 2. Test timing and parallelization

**Problem:** Testing was deferred to Phase 4 Stage 5 and then not completed. Some tests could have been written earlier and in parallel with implementation.

**Analysis:**

Biome's testing infrastructure has 5 distinct layers, each with different timing constraints:

**Layer 1: Inline smoke tests (`#[test]` in `lib.rs`)**
- **When:** Write *during* each stage, as the last step before declaring the stage complete
- **Cost:** ~20 lines per crate, <1 minute to write
- **What they catch:** Basic compilation, API surface works, trivial round-trip
- **Current state:** Both formatter and analyzer have one inline test each. These were written during implementation and caught nothing that the compiler didn't already catch, but they serve as documentation of the API.
- **Parallelizable:** N/A — written by whoever implements the stage

**Layer 2: Quick tests (`tests/quick_test.rs`)**
- **When:** Create the file at Stage 1 start (formatter) or Stage 2 start (analyzer), marked `#[ignore]`. Un-ignore when debugging a specific case.
- **Cost:** ~45 lines scaffold, then ad-hoc modifications
- **What they catch:** Individual node formatting, individual rule behavior. The CONTRIBUTING.md recommends: "Remove or comment out the `#[ignore]` macro, modify the `let SOURCE` variable with test code, update rule filter, run with `cargo t quick_test -- --show-output`."
- **Current state:** Missing for both formatter and analyzer (the inline tests serve a similar purpose but live in `src/lib.rs` rather than `tests/`)
- **Parallelizable:** N/A — used interactively during development

**Layer 3: Snapshot/fixture tests (`tests/spec_tests.rs` + `tests/specs/`)**
- **When:** The test harness (`spec_tests.rs`, `spec_test.rs`, `language.rs`) should be created at Stage 1 start. It's pure boilerplate (~250 lines for formatter, ~255 lines for analyzer) copied from JSON with language-specific types swapped in. Fixture files should be added incrementally as nodes/rules are implemented.
- **Cost:** Harness is ~500 lines total (one-time). Each fixture is a `.yaml` file + expected `.snap` output.
- **What they catch:** Formatting correctness for specific input patterns. Regression detection. The `gen_tests!` macro auto-generates a test function for every `.yaml` file in the specs directory.
- **Current state:** Entirely missing — 0 test directories, 0 harness files, 0 fixtures
- **Parallelizable:** **Yes.** Formatter fixtures and analyzer fixtures are completely independent. Could be written by two separate agents simultaneously. The harness files could also be created in parallel with Stage 1/2 implementation (since they don't depend on the formatter/analyzer code, only on the generated syntax types which exist already).

**Layer 4: End-to-end / CLI integration tests**
- **When:** After Stage 4 (service integration) is complete. Cannot be written earlier because they require the full `biome` binary to accept YAML files.
- **Cost:** Variable — biome's existing CLI test infrastructure is in `crates/biome_cli/tests/`
- **What they catch:** Full pipeline: file detection → parsing → formatting/linting → output. This is where the three-registration-system bug and the formatter indentation bugs were caught manually.
- **Current state:** Missing. Manual end-to-end testing was performed but not captured as automated tests.
- **Parallelizable:** No — depends on all prior stages

**Round-trip property testing (`format(format(x)) == format(x)`):**
- **When:** After the snapshot test harness exists. Can be a property of the spec_test.rs runner (format twice, assert idempotent). The JSON formatter already does this in its test harness.
- **Cost:** ~5 lines added to the test runner
- **Current state:** Missing. The inline smoke test verifies one case, but there's no systematic idempotency check.

**Layer 5: Fuzz testing (`fuzz/fuzz_targets/rome_*_yaml.rs`)**

Biome has an existing fuzz infrastructure in `fuzz/` using `libfuzzer-sys`. It has parser and formatter fuzz targets for JS and JSON. The JSON formatter fuzzer (`rome_common.rs:fuzz_json_formatter`) checks three properties on random input: (a) formatting doesn't introduce parse errors, (b) formatting is idempotent, (c) formatting doesn't introduce new linter errors. No YAML fuzz targets exist yet.

Fuzzing has a fundamentally different timing pattern from the other 4 layers. Testing (layers 1-4) is **point-in-time**: you write a test, you run it, it passes or fails. Fuzzing is **continuous**: you set up infrastructure once, seed a corpus, and let it run in the background — potentially for hours or days. Each time you add new formatter nodes or lint rules, the fuzzer is already exercising them. When it finds a crash or assertion failure, it produces a minimal reproducer that becomes a permanent regression fixture in layer 3.

This makes fuzzing more like CI than like unit testing — it's infrastructure that pays off over time, not a one-shot check.

**Four levels of fuzzing, built up incrementally:**

| Level | Technique | What it catches | When to create | Effort |
|-------|-----------|----------------|---------------|--------|
| 1a | **Unstructured parser fuzzing** (random bytes → parse_yaml) | Parser panics/crashes on malformed input | **Pre-Phase 4** — parser already exists | ~20 lines, copy from `fuzz_json_parser` |
| 1b | **Unstructured formatter fuzzing** (random bytes → parse → format → re-parse → re-format) | Formatter crashes, formatter introduces parse errors, formatter is non-idempotent | **End of Stage 1** — requires formatter | ~40 lines, copy from `fuzz_json_formatter` |
| 2 | **Corpus-based fuzzing** (mutate real YAML files as seeds) | Edge cases near valid YAML — partial documents, unusual indentation, mixed block/flow styles | **End of Stage 4** — seed with real-world files | Low — add files to `fuzz/corpus/` directory |
| 3 | **Property-based testing** (`proptest` — generate valid YAML structures programmatically) | Semantic preservation on valid input, structural round-trip fidelity | **Post-Phase 4** — when formatter is stable | Medium — need a YAML AST generator (~200 lines) |

Level 4 (grammar-guided structure-aware fuzzing) exists conceptually but is high effort and unlikely to be needed for the first or second language.

**Why YAML fuzzing is higher value than JSON fuzzing:**

- **Indentation is semantic.** A formatter bug that shifts indentation by one space can silently restructure data — a nested key becomes a sibling, a sequence item becomes a mapping value. JSON formatting can't change semantics (braces/brackets are explicit). YAML formatting can. The idempotency check in the fuzzer (`format(format(x)) == format(x)`) catches unstable formatting. But the more critical check for YAML is **semantic preservation**: `parse(format(parse(input))).tree_structure == parse(input).tree_structure`. This verifies that formatting doesn't change what the YAML *means*, not just what it looks like.

- **YAML's grammar is ambiguous.** Block scalars with chomping indicators (`|+`, `>-`), anchor/alias references, tag directives, multi-document streams, implicit keys, flow-in-block contexts. Handwritten fixtures will cover the common cases; fuzzing covers the combinatorial explosion of interactions between these features.

- **YAML spec version differences.** `yes`/`no`/`on`/`off` are booleans in YAML 1.1 but strings in 1.2. `0777` is octal in 1.1 but a string in 1.2. Fuzzed input generates these ambiguous patterns naturally.

**Fuzzing timing and parallelization:**

Fuzzing is **fully parallelizable with everything** because:
- Level 1a (parser fuzzing) has zero dependencies on Phase 4 work — the parser already exists
- Level 1b (formatter fuzzing) depends only on Stage 1 completion, then is independent
- Corpus collection (Level 2) is independent of all implementation — just gathering files
- Fuzzing *runs* are background processes — they run while other work happens

The optimal strategy is **infrastructure early, run continuously, harvest results as fixtures:**

```
Pre-Phase 4:
  └── Level 1a: create rome_parse_yaml.rs fuzz target (~20 lines)
       Can run immediately — parser exists. Catches parser panics.

Stage 1 (Formatter) END:
  └── Level 1b: create rome_format_yaml.rs fuzz target (~40 lines)
       Add biome_yaml_parser + biome_yaml_formatter to fuzz/Cargo.toml.
       Checks: format doesn't crash, re-parses cleanly, idempotent.
       Also add: create fuzz_yaml_formatter_with_linting (after Stage 2)
       to check formatting doesn't introduce lint errors.

Stage 2 (Analyzer) END:
  └── Update Level 1b: add linter stability check to formatter fuzzer
       (format → re-analyze → no new diagnostics)

Stage 4 (Service Integration) END:
  └── Level 2: seed fuzz/corpus/yaml/ with real-world YAML files
       Sources: YAML Test Suite, Kubernetes manifests, GitHub Actions
       workflows, Docker Compose files, Ansible playbooks.
       Run fuzzer with corpus — better coverage than random bytes.

Post-Phase 4 (ongoing):
  └── Level 3: property-based testing with proptest
       Generate random valid YAML ASTs, format them, verify properties.
       This is the most thorough level but requires a YAML structure generator.

Continuous (background during any phase):
  └── When fuzzer finds a crash/assertion failure:
       1. Fuzzer produces minimal reproducer file
       2. Add reproducer as a Layer 3 snapshot fixture (permanent regression test)
       3. Fix the bug
       4. Reproducer ensures it never recurs
```

**The harvest loop** is the key efficiency insight. Fuzzing doesn't just find bugs — it **generates test cases**. Each minimized reproducer from the fuzzer becomes a permanent snapshot fixture. Over time, the fuzzer builds up a regression suite that no human would write, covering edge cases that emerge from the combinatorial interaction of YAML features. This means:

- Layer 3 (snapshot fixtures) grows organically from Layer 5 (fuzzing), not just from manual authoring
- The fixture suite gets stronger over time without additional human effort
- Edge cases discovered by fuzzing in one language inform fixture design for the next language

**Gate integration:**

Add to the Environment Readiness Gate (Pre-Phase 4):
```
- [ ] `cargo install cargo-fuzz` succeeds (or `cargo +nightly fuzz --version`)
- [ ] `fuzz/fuzz_targets/rome_parse_{language}.rs` exists
```

Add to the Code Quality Gate (end of Phase 4):
```
- [ ] `fuzz/fuzz_targets/rome_format_{language}.rs` exists
- [ ] Fuzzer runs for 5+ minutes with zero crashes on default corpus
- [ ] Any crash reproducers added as snapshot fixtures
```

**Current state for YAML:** No fuzz targets exist. `fuzz/Cargo.toml` has no YAML dependencies. This is uncaptured work alongside the testing gap.

**Recommended test timeline for second language (updated with all 5 layers):**

```
Pre-Phase 4 (environment + infrastructure):
  ├── Layer 5: create parser fuzz target, start running in background
  └── GATE: verify cargo-fuzz, cargo-expand, cargo-insta installed

Stage 1 (Formatter):
  ├── START: create test harness (spec_tests.rs, spec_test.rs, language.rs, quick_test.rs)
  ├── DURING: add fixture file after each group of formatter nodes is implemented
  ├── DURING: use quick_test.rs for interactive debugging
  ├── END: inline smoke test in lib.rs, verify all fixtures pass
  └── END (Layer 5): create formatter fuzz target, start running in background

Stage 2 (Analyzer):
  ├── START: create test harness (spec_tests.rs)
  ├── DURING: add valid.yaml + invalid.yaml per rule
  ├── DURING: use quick_test in lib.rs for debugging
  ├── END: verify all fixtures pass, add suppression tests
  └── END (Layer 5): update formatter fuzzer with linter stability check

Stage 3 (Configuration):
  └── END: accept updated configuration snapshots (these break automatically)

Stage 4 (Service Integration):
  ├── END: manual end-to-end test (minimum), ideally add CLI integration test
  └── END (Layer 5): seed corpus with real-world files, run fuzzer 5+ min

Stage 5 (Review):
  ├── Verify: round-trip idempotency, all fixtures pass, snapshot review
  └── Harvest: add any fuzzer reproducers as permanent fixtures

Ongoing (background, any phase):
  └── Fuzzer runs continuously; reproducers become regression fixtures
```

**Key difference between fuzzing and other layers:**
- Layers 1-4 are **written once, run at test time** — their value is fixed at creation
- Layer 5 is **created once, runs continuously** — its value grows over time as it discovers edge cases
- Layers 1-4 catch known failure modes. Layer 5 catches **unknown unknowns**.

**Key change from YAML-first:** Fuzzing infrastructure should exist before Phase 4 implementation begins (parser fuzz target), and grow with each stage. Don't defer it — the earlier the fuzzer starts running, the more edge cases it finds by the time you need them.

### 3. Lightweight container impact

**Problem:** Development used `.devcontainer/erasimus/devcontainer.json` (base image `mcr.microsoft.com/devcontainers/rust:1`) instead of the full Biome devcontainer (base image `mcr.microsoft.com/devcontainers/universal:5` with `just` feature).

**Concrete differences:**

| Tool | Full devcontainer | Erasimus lightweight | Impact on YAML development |
|------|------------------|---------------------|---------------------------|
| `just` | Installed via feature | **Not installed** | **High** — see below |
| `cargo-binstall` | Installed by `just install-tools` | Not installed | Low — only needed to install other tools faster |
| `cargo-insta` | Installed by `just install-tools` | Installed manually mid-session | **Medium** — delayed snapshot workflow |
| `cargo-expand` | Not in either | Not installed | **Would have been high** — see debugging section |
| `tombi` | Installed by `just install-tools` | Not installed | None — TOML formatting irrelevant |
| `wasm-bindgen-cli` | Installed by `just install-tools` | Not installed | None — no WASM needed |
| `wasm-opt` | Installed by `just install-tools` | Not installed | None — no WASM needed |
| Rust toolchain | `complete` profile | Default (presumably `default` or `minimal`) | Low — `complete` adds `clippy`, `rustfmt`, cross-compile targets. Core compilation unaffected |
| Claude CLI + config | Not included | **Mounted from host** | The Erasimus container is purpose-built for Claude Code development |

**`just` absence — detailed impact:**

The Justfile (281 lines) defines the canonical Biome development workflows. Every command in CONTRIBUTING.md references `just`:

| `just` command | What it does | How we worked around it | What we missed |
|---------------|-------------|----------------------|---------------|
| `just gen-formatter` | `cargo run -p xtask_codegen -- formatter` | Ran the raw cargo command | Nothing — exact equivalent |
| `just gen-analyzer` / `just gen-rules` | `cargo run -p xtask_codegen -- analyzer` | Ran the raw cargo command | Nothing — exact equivalent |
| `just gen-configuration` | `cargo run -p xtask_codegen -- configuration` | Ran the raw cargo command | Nothing — exact equivalent |
| `just test` | Full test suite with `cargo nextest` or `cargo test` | `cargo test -p <crate>` per crate | **Missed cross-crate test failures** — never ran full suite |
| `just f` / `just format` | `cargo fmt` + `tombi format` | **Never ran** | **Potentially unformatted Rust code** in contributed files |
| `just l` / `just lint` | `cargo clippy` with specific flags | **Never ran** | **Potentially missed clippy warnings** |
| `just ready` | `gen-all` + `documentation` + `format` + `lint` + `test` + `test-doc` | **Never ran** | **No pre-PR validation was performed** |
| `just test-lintrule` | Runs snapshot tests for a specific rule | Not used (no snapshot tests exist) | N/A |
| `just new-crate` | Creates new crate with workspace config | Created crates manually | Possibly inconsistent Cargo.toml formatting |

**The `just ready` gap is the most significant.** This is Biome's equivalent of CI locally. It runs codegen, formats, lints, and tests everything. By never running it, we have no assurance that the contributed code passes the project's quality gates. Before any PR, this must be run — which requires either installing `just` in the lightweight container or switching to the full devcontainer.

**Would the full devcontainer have changed engineering agent usefulness?**

Minimally. The agents don't invoke build tools directly — they work through the main conversation's Bash tool. But the validation loop would have been tighter:
- `just test-lintrule noDuplicateKeys` would have immediately shown whether the rule fires in snapshot tests (if they existed)
- `just l` after each stage would have caught clippy warnings early
- `just ready` at the end of Phase 4 would have been a one-command validation gate

The agents' value isn't gated by `just` — it's gated by context (as analyzed in the engineering agent section). But the *validation step* that confirms whether agent output is correct would have been easier with the full toolchain.

**Recommendation:** For the second language, either:
- (a) Install `just` into the Erasimus container via `postCreateCommand`: add `cargo install just` (or use the system package manager)
- (b) Add a `just ready` equivalent as a raw cargo command sequence to CLAUDE.md so it can be run without `just`
- (c) Switch to the full devcontainer for implementation phases (Phases 4+), keep the lightweight container for research phases (Phases 1-3) where build tools aren't needed

Option (a) is simplest — one line added to the devcontainer's `postCreateCommand`.

### 4. Debugging practices: timing, gated infrastructure, and CONTRIBUTING.md enforcement

**Problem:** Debugging during Phase 4 was entirely reactive — tools were reached for only after something broke, and the most valuable tool (`cargo expand`) wasn't even installed. The CONTRIBUTING.md documents systematic practices, but reading a document doesn't enforce behavior. Debugging infrastructure should be **proactive**: set up before it's needed, like testing infrastructure.

#### What CONTRIBUTING.md recommends vs. what was used

| Technique | Recommended by | Used during Phase 4? | Would it have helped? |
|-----------|---------------|---------------------|---------------------|
| `dbg!()` macro | Analyzer CONTRIBUTING | No — used `eprintln!` instead | **Equivalent** — `dbg!` is slightly better because it prints expression + file:line automatically |
| `cargo test -- --show-output` | Analyzer CONTRIBUTING | No — used `cargo t` without flags | **Yes** — would have shown debug output from passing tests without needing manual prints |
| `--profile debugging` | Root CONTRIBUTING + `Cargo.toml` `[profile.debugging]` | No | **Marginal** — preserves debug symbols for stack traces. The registration bug was a logic error, not a crash. Would help for panics/crashes. |
| `cargo expand` | Not in CONTRIBUTING (would be an improvement) | No — **not installed** | **Significantly** — the single highest-value debugging tool for the registration bug (see below) |
| `RUST_LOG` / tracing | Not in CONTRIBUTING | No | **Moderate** — useful for service-layer debugging. Not applicable to the registration bug. |
| `cargo insta review` | Analyzer + Formatter CONTRIBUTING | Yes (for configuration snapshots) | **Yes** — used correctly. More valuable once snapshot tests exist. |
| Quick test (`tests/quick_test.rs`) | Both CONTRIBUTINGs | Partially — inline `quick_test` in `lib.rs` but not in `tests/` | **Yes** — the `tests/quick_test.rs` pattern is better: run with `--show-output`, doesn't pollute `lib.rs` |

#### The `cargo expand` gap

The three-registration-system bug (Phase 4 discovery #1) consumed 2+ hours. The debugging was:

1. Add `eprintln!` to `yaml.rs` handler → confirmed parse/lint functions are called
2. Add `eprintln!` to analyzer `lib.rs` → confirmed `analyze()` is called
3. Manually count enabled rules → found 213, expected 214
4. Search `rules.rs` for `noDuplicateKeys` → found it registered in the enum
5. Search for `recommended_rules_as_filters()` → found it in generated group structs
6. Trace from group struct to proc macro → found `collect_lint_rules()` doesn't visit YAML

`cargo expand -p biome_configuration_macros` would have jumped from step 3 to the answer — showing the generated `Suspicious` struct without `noDuplicateKeys`. Instead, steps 4-6 required manual reading of generated code.

`cargo expand` requires the nightly toolchain. It is not installed in either devcontainer. This should be a Phase 4 prerequisite.

#### Debugging infrastructure timeline

Like testing, debugging infrastructure has timing dependencies. The right approach is proactive setup:

```
Pre-Phase 4 (environment setup):
  ├── GATE: Verify cargo-expand installed (rustup toolchain install nightly && cargo install cargo-expand)
  ├── GATE: Verify cargo-insta installed
  ├── GATE: Verify [profile.debugging] exists in root Cargo.toml
  └── GATE: Verify cargo test -- --show-output works

Stage 1 (Formatter):
  ├── START: create tests/quick_test.rs with #[ignore] — available from day one
  ├── DURING: use quick_test.rs + --show-output for node debugging (not eprintln!)
  ├── DURING: use dbg!() (not eprintln!) — auto-prints file:line, caught by clippy in release
  └── END: verify no dbg!()/eprintln!() remain in committed code

Stage 2 (Analyzer):
  ├── START: create tests/quick_test.rs with #[ignore] (separate from formatter's)
  ├── DURING: use quick_test.rs + --show-output for rule debugging
  ├── DURING: if rule doesn't fire → cargo expand -p biome_configuration_macros FIRST
  └── END: verify no debug artifacts remain

Stage 3 (Configuration):
  └── DURING: cargo insta review for snapshot changes (already available)

Stage 4 (Service Integration):
  ├── DURING: if end-to-end test fails, use cargo expand before manual tracing
  ├── DURING: if panic/crash, use --profile debugging for stack traces
  ├── DURING: if file not recognized, check RUST_LOG output from service layer
  └── END: verify no debug artifacts remain

Post-Stage 4 (before commit):
  └── GATE: grep for dbg!(), eprintln!(), println!() in modified files — must be zero
```

**Key change from YAML-first:** Don't wait for a bug to install debugging tools. Verify them at Phase 4 start. Don't wait for confusion to create quick_test.rs. Create it at Stage 1/2 start. The 5 minutes spent setting up debugging infrastructure saves hours when something goes wrong.

#### Gated CONTRIBUTING.md practices

The CONTRIBUTING.md is 2376+ lines across three files. Reading it doesn't enforce compliance. The following practices should be **gated** — verified at specific points, not left to interpretation:

**Gate: Environment readiness (Pre-Phase 4)**

Verify before any implementation begins:

```markdown
## Environment Readiness Gate (BLOCKING)
Before starting implementation, verify ALL of the following:
- [ ] `cargo fmt --version` succeeds (rustfmt installed)
- [ ] `cargo clippy --version` succeeds (clippy installed)
- [ ] `cargo insta --version` succeeds (cargo-insta installed)
- [ ] `cargo expand --version` succeeds (cargo-expand installed, nightly toolchain present)
- [ ] `just --version` succeeds OR raw cargo equivalents are documented in CLAUDE.md
- [ ] `[profile.debugging]` section exists in root Cargo.toml
If any check fails, install the missing tool before proceeding.
```

**Gate: Stage start (per stage)**

Before implementing any formatter node or lint rule:

```markdown
## Stage Start Gate (BLOCKING)
- [ ] tests/quick_test.rs exists for this crate (with #[ignore])
- [ ] Test harness exists (spec_tests.rs + language.rs for formatter, spec_tests.rs for analyzer)
- [ ] Previous stage compiles: `cargo build -p <previous_crate>` succeeds
```

**Gate: Debug hygiene (per commit)**

Before any commit within Phase 4:

```markdown
## Debug Hygiene Gate (BLOCKING)
- [ ] `grep -rn 'dbg!\|eprintln!\|println!' <modified_files>` returns zero matches
- [ ] No `#[ignore]` removed from quick_test.rs (it should remain ignored in committed code)
- [ ] No test files contain hardcoded paths or debug-specific configuration
```

**Gate: Code quality (end of Phase 4)**

Before declaring implementation complete:

```markdown
## Code Quality Gate (BLOCKING)
- [ ] `cargo fmt -- --check` passes for all modified crates
- [ ] `cargo clippy -p biome_{language}_formatter -p biome_{language}_analyze` passes with zero warnings
- [ ] All snapshot tests pass: `cargo insta test -p biome_{language}_formatter -p biome_{language}_analyze`
- [ ] No TODO/FIXME/HACK comments in production code (or each is tracked as a deferred work item)
- [ ] Inline smoke tests exist and pass for formatter + analyzer
```

**Gate: PR readiness (before contribution)**

The full validation equivalent to `just ready`:

```markdown
## PR Readiness Gate (BLOCKING)
- [ ] `cargo fmt -- --check` (all crates, not just modified)
- [ ] `cargo clippy --workspace` (or at minimum, affected crates)
- [ ] `cargo test -p biome_{language}_formatter -p biome_{language}_analyze -p biome_service`
- [ ] Snapshot tests exist and are accepted
- [ ] `cargo test -p biome_configuration` passes (snapshot updates accepted)
- [ ] End-to-end: `biome format`, `biome lint`, `biome check` work on sample files
- [ ] No debug artifacts in code
```

#### Code contribution standards assessment

The current YAML implementation has these quality gaps relative to gated expectations:

| Standard | Expected | Current | Gap |
|----------|----------|---------|-----|
| Snapshot tests | Required (Stage Start Gate) | None | **Critical** |
| Quick test file | Required (Stage Start Gate) | Inline only | **Medium** — functional equivalent exists but wrong location |
| `cargo fmt` clean | Required (Code Quality Gate) | **Unknown** — never ran | **Must verify** |
| `cargo clippy` clean | Required (Code Quality Gate) | **Unknown** — never ran | **Must verify** |
| `cargo expand` available | Required (Environment Gate) | **Not installed** | **Must install** |
| Debug artifact check | Required (Debug Hygiene Gate) | Cleaned manually | **Gate would automate** |
| `just ready` / equivalent | Required (PR Readiness Gate) | **Never ran** | **Blocking for PR** |
| Doctests | Encouraged (not gated) | None | **Minor** |

**Recommendation:** The gates above should be embedded in the `/lang-implement` command definition and enforced at the specified points. They transform CONTRIBUTING.md from "read this and follow it" into "these checks must pass before proceeding." Add `cargo-expand`, `just`, and `cargo-insta` to the Erasimus devcontainer's `postCreateCommand`. Make `cargo fmt --check`, `cargo clippy`, and debug artifact scanning part of Gate 4.

## Standards

This section collects all standards governing development work. Standards fall into two categories: **external** (defined by the Biome project, enforced by CI or convention) and **internal** (defined by this methodology, enforced by gates).

### Biome Contribution Specification

**Document:** `kb/tasks/biome-contrib-spec.md`

The comprehensive extraction of all contribution instructions from Biome's CONTRIBUTING.md files (4 primary + 3 crate-specific), CI workflow files (3), configuration files (4), and governance documents (2). 18 source files total, 80+ individual instructions categorized as mandatory, optional, or not applicable.

Key sections and their relevance to implementation phases:

| Spec Section | Content | When It Matters |
|---|---|---|
| 1-2: Environment & Workflow | Toolchain, `just`, codegen, formatting, linting | Pre-Phase 4 (environment gate) |
| 3: Testing | `cargo insta`, snapshot tests, quick tests, spec_tests harness | Stages 1-2 (test infrastructure) |
| 4: Parser Development | Grammar conventions, naming, error recovery | Already complete for YAML |
| 5: Formatter Development | Crate creation, codegen, required types, verbatim stubs | Stage 1 (formatter) |
| 6: Analyzer / Lint Rules | Rule types, naming, diagnostics, documentation, code actions | Stage 2 (analyzer) |
| 7: Contribution Process | AI disclosure, commits, branches, changesets | PR readiness gate |
| 9: Code Style & Lint Policy | 60+ workspace clippy/rust lints, disallowed methods, `.editorconfig` | Every commit |
| 10: CI/CD Requirements | PR jobs (lint, test, udeps, e2e, docs), PR title validation | PR readiness gate |
| 11: Diagnostics | WHAT/WHY/FIX, `#[derive(Diagnostic)]`, advice types, category registration | Stage 2 (analyzer) |
| 12: Service Integration | Workspace trait, `WorkspaceServer` vs `WorkspaceClient` | Stage 4 (service wiring) |
| 13: Formatter Rules | Use AST tokens not literals, don't "fix" code, `dbg_write!` for IR | Stage 1 (formatter) |
| 14-15: PR & Governance | PR template, CodeRabbit config, liberal review philosophy | PR readiness gate |

The spec's 5 gate tables (Environment Readiness, Stage Start, Debug Hygiene, Code Quality, PR Readiness) map directly to the deterministic gates defined in this document.

### Documentation

**Standard type:** Implied (Biome has no explicit general documentation standard)

**What Biome enforces:**
- `RUSTDOCFLAGS='-D warnings'` in CI — broken intra-doc links fail the build
- Rule documentation: rigid structure enforced by codegen (first paragraph single-line, `## Examples`, `### Invalid` before `### Valid`, code blocks auto-validated)
- `cargo documentation` CI job builds docs for ~17 key packages

**What Biome does NOT enforce:**
- No `missing_docs` lint — documentation on public items is voluntary
- No standard for module-level `//!` comments
- No standard for struct/trait/function doc comments outside lint rules
- No prose style guide for doc comments

**Codebase survey results** (from sampling 9 representative crates):

| Maturity | Example Crates | Public Item Doc Coverage | Module Docs |
|---|---|---|---|
| Core infrastructure | `biome_formatter` | ~72% | Yes (structured sections) |
| Mature infrastructure | `biome_parser`, `biome_analyze` | 63-69% | No |
| Language-specific | `biome_json_formatter`, `biome_diagnostics` | 38-63% | No |
| Newer crates | `biome_yaml_*` | 40-48% | No |
| Foundational | `biome_rowan` | ~6% | One-liner only |

**Implied standard:** Document public API of core infrastructure crates. Language-specific crates, internal types, and trait implementations are undocumented by convention. Rules are the exception — they have rigid requirements because rustdoc generates user-facing website content.

**Why documentation doesn't need timing/parallelization analysis:** Unlike testing (5 layers with different timing) or debugging (proactive setup vs reactive use), documentation in Biome is either co-located with code (rustdoc on rules, validated by CI as part of the build) or deferred post-merge (website docs PR). There is no separate documentation phase, no background documentation process, and no documentation that should "start at Stage 2 but finish at Stage 4." It's binary: inline with implementation (and therefore gated by the same CI as code), or optional follow-up.

**For this methodology:** The existing gate structure is sufficient — Gate 4 (Code Quality) checks rule doc validity, Gate 5 (PR Readiness) checks changeset descriptions and PR sections. No additional documentation gates are needed beyond what the biome-contrib-spec already defines.

### Testing

**Standard type:** Mixed (Biome has explicit test infrastructure patterns; timing/layering is internal methodology)

**Reference:** Open question #2 in this document (test timing and parallelization)

Five testing layers, each with different timing:

| Layer | What | When to Create | Parallelizable |
|---|---|---|---|
| 1: Inline smoke tests | `#[test]` in `lib.rs` | During each stage | No |
| 2: Quick tests | `tests/quick_test.rs` | Stage start | No (interactive) |
| 3: Snapshot/fixture tests | `tests/spec_tests.rs` + `tests/specs/` | Stage start (harness), incremental (fixtures) | Yes |
| 4: E2E / CLI tests | Full pipeline tests | After Stage 4 | No |
| 5: Fuzz testing | `fuzz/fuzz_targets/rome_*_yaml.rs` | Pre-Phase 4 (parser), end of Stage 1 (formatter) | Yes (background) |

**Gate integration:** Environment Readiness gate verifies test tooling. Stage Start gate verifies harness exists. Code Quality gate verifies tests pass. See the test timeline in open question #2 for the detailed schedule.

### Debugging

**Standard type:** Internal methodology (Biome's CONTRIBUTING.md recommends tools but doesn't gate on them)

**Reference:** Open question #4 in this document (debugging practices)

Key principle: **proactive setup, not reactive installation.** Debugging tools (`cargo-expand`, `--profile debugging`, `quick_test.rs`) should be verified at Phase 4 start, not installed after a bug appears.

**Gate integration:** Environment Readiness gate verifies `cargo-expand`, `cargo-insta`, `[profile.debugging]`. Debug Hygiene gate (per commit) verifies no `dbg!()`/`eprintln!()` in production code.

### Process

**Standard type:** Internal methodology

**Reference:** Process issues section in this document; command structure and deterministic gates section

Key process standards:

| Standard | What It Enforces | Where Defined |
|---|---|---|
| Plan persistence | Plans written to disk before implementation begins | Process issue: Plan file persistence |
| `/clear` over `/compact` | Full context reset at phase boundaries, not lossy compression | Open question #1 |
| Deterministic gates | 4 hard gates at phase boundaries (plan, prereqs, summary, tests) | Command structure section |
| Phase summaries with resumption instructions | Every phase ends with a summary containing "how to continue" | Gate 3 (phase summary) |
| Silent failure detection | Three-registration-system check for new languages | Process issue: Silent registration failures |
| Convention commit format | `feat(crate): description`, lowercase subject | Biome-contrib-spec §7, §10.3 |
| AI disclosure | Required in all PRs | Biome-contrib-spec §7.1, §14.2 |

## Revised assessment

**Option E: Incremental toward a generalized plugin.** Build using YAML as the first language. Use language-agnostic naming. Let each phase inform the next. The plugin-dev toolkit's `${CLAUDE_PLUGIN_ROOT}` pattern and the skill-creator's progressive disclosure (`references/` directories per language) are the two most important design patterns — they are exactly how a reusable toolkit gets parameterized.
