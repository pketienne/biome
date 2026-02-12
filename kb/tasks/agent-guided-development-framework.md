# Agent-Guided Development Framework

How agent-guided software development evolves from first implementation to reusable methodology. Validated on Biome language integration (YAML as first language), designed to generalize across projects requiring systematic feature extraction, architecture analysis, specification, and implementation.

## How to use this document

### Purpose

This document serves three roles:

1. **Blueprint** — a step-by-step process for integrating a new language into the Biome toolchain (or analogous multi-phase development tasks in other projects)
2. **Methodology** — reusable patterns for agent-guided development: how to structure work across phases, what gates prevent rework, what standards maintain quality
3. **Agent reference** — concrete lessons about what agents need to know, when they're useful, and how to structure their work

### Document architecture

Concerns are organized into three tiers, following the pattern established by the [Quality Audit Framework](quality-audit-framework.md):

| Tier | What it contains | Role |
|------|-----------------|------|
| **Phase architecture** (1-7) | Temporal progression from requirements through implementation, with concrete discoveries | What to build and when |
| **Development standards** (A-D) | Practices governing testing, debugging, documentation, and external compliance | What to maintain throughout |
| **Cross-cutting concerns** (I-III) | Gates, context management, and process patterns that apply across all phases | How to enforce quality |

Phases are temporal — they happen in order. Standards are persistent — they apply throughout. Cross-cutting concerns are enforcement mechanisms — they ensure phases meet standards.

### Phase structure

Every phase includes at minimum:

| Section | Purpose | Populated when |
|---------|---------|----------------|
| **Build** | What artifacts this phase creates | At phase start (planned) |
| **Discover** | What questions this phase should answer | At phase start (planned) |
| **Discovered** | What was actually learned (numbered findings) | At phase end (retrospective) |
| **Crystallizes** | What pattern became reusable | At phase end (retrospective) |

Phases 1-5 have populated "Discovered" sections from the YAML implementation. Phase 6 has partial findings. Phase 7 is speculative.

### Priority evaluation

Within each phase, work items are prioritized using four axes adapted from the [Quality Audit Framework](quality-audit-framework.md):

| Axis | Determined by | Varies per project? |
|------|--------------|---------------------|
| **Exposure** | How much of the risk surface the project touches | Yes — assessed per project |
| **Gap** | How far current state is from acceptable | Yes — measured during work |
| **Leverage** | Whether improving this area enables improvement in others | No — inherent to the area |
| **Accumulation** | How fast neglect compounds | No — inherent to the area |

### Maturity progression

Standards follow a four-level maturity model:

| Level | State | Indicators |
|-------|-------|------------|
| 0 — None | No capability | No tooling, no process |
| 1 — Manual | Capability exists, manually applied | Agent remembers to check |
| 2 — Automated | Integrated into workflow | Command enforces check |
| 3 — Gated | Blocks progress on failure | Gate prevents continuation |

The methodology itself progresses through these levels. Phase 1-5 observations identified which practices need to move from Level 1 (manual) to Level 3 (gated).

---

## Foundational insight: process as reusable asset

The workflow repeats across multiple targets (for Biome: 6+ languages — shell/bash, turtle, toml, ruby, rust, etc.). What stays the same:

- The workflow phases (discover tools → extract features → study internals → write specs → implement → review)
- The agent roles (feature extractor, architecture analyst, spec writer, code reviewer)
- The skills (feature comparison templates, extension contract knowledge, spec methodology/rubric)
- The hooks (require spec before implementation, post-edit validation)
- The commands (research, architecture, spec, status, review)

What changes per target:

- Which repos to clone and scan
- Target-specific grammar/parsing concerns
- Which internal crates or modules to reference
- The feature matrix content

The **process itself** is the reusable asset, not the target-specific knowledge. This shifts the architecture from a one-off workflow to a generalized toolkit, built incrementally using the first target as validation.

Naming should be target-agnostic from the start: `lang-feature-extractor` not `yaml-feature-extractor`. Target-specific knowledge goes into `references/{target}/` directories, not into agent prompts.

---

## Phase Architecture

### Phase 1: Requirements Extraction

The first phase for any new target. For Biome language integration, this means extracting features from existing tools in the language ecosystem.

**Build:**
- `lang-feature-extractor` agent + `/lang-research` command
- A `references/{target}/` directory with tool inventory

**Discover:**
- What structured output actually looks like for a feature comparison (the template is unknown until extraction is done once)
- Whether parallel subagents per source are worth it or if sequential is fine
- What reference knowledge the extractor needs per source vs. what's generic
- Whether specification grounding (e.g. YAML 1.1 vs 1.2) needs to be part of extraction or can be added in synthesis

**Discovered (from first YAML run):**

1. **Parallel by tool type, not per-repo.** 4 parallel agents (linters, formatters, parsers, validators) across 13 repos completed in ~260s wall-clock. Sequential would have been ~892s — a 3.4x speedup. Grouping by tool type was better than per-repo because cross-tool comparison (linter vs linter) is the core output, so each agent produced a coherent comparison directly. Per-repo agents would have required heavier synthesis to merge 13 separate reports.

2. **Structured output template emerged.** The report settled into: executive summary → feature matrices by category → consensus features ranked by prevalence → unique features with rationale → architectural observations → spec grounding → recommended next steps. Appendices for default config comparisons. This template is reusable for other targets.

3. **Reference knowledge splits cleanly.** The tool inventory (`references/yaml/tools.md`) with repo paths and feature file locations was all the per-tool knowledge agents needed. Generic extraction instructions (what to look for in linters vs formatters vs parsers) were provided in the agent prompt. No tool-specific knowledge leaked into the agent definition.

4. **Spec grounding must be part of the workflow, not an afterthought.** The first run produced a report without spec classification. It was immediately apparent that features are meaningless without understanding specification divergences. The agent and command were updated to require spec basis classification (spec-mandated, spec-divergence, spec-ambiguity, tool-opinion) for each feature.

**Crystallizes:** The structured output template is now a reusable pattern. The agent definition and `/lang-research` command are no longer drafts — they've been exercised and refined. The `references/{target}/` directory pattern (tools.md + feature-research-report.md) is validated.

### Phase 2: Architecture Analysis

**Build:**
- `lang-architecture-analyst` agent
- `references/{project}/` directory capturing what the reference implementation study revealed about the project's extension contract

**Discover:**
- What questions the spec writer will need answered
- Whether the architecture analyst and feature extractor need to talk to each other (routing/chaining) or are truly independent
- What project knowledge is target-specific vs. universal

**Discovered (from YAML architecture study):**

1. **7 distinct layers, not monolithic.** Each layer has a separate trait contract (`Language`, `SyntaxFactory`, `Parser`, `FormatLanguage`, `Rule`, `ExtensionHandler`, `ServiceLanguage`). The extension contract document organizes naturally by layer, with each section mapping to a specific crate boundary.

2. **Extractor and analyst are fully independent.** No shared inputs, state, or outputs between the feature extractor and architecture analyst. They can run in parallel. The spec writer (Phase 3) is the first consumer of both — it needs the feature matrix (what to build) and the extension contract (how to build it).

3. **Project knowledge splits cleanly: universal vs. per-target.** Universal knowledge (the 7-layer contract, trait signatures, file patterns) lives in `references/biome/extension-contract.md`. Per-target knowledge (what exists, what's missing) goes in `references/{language}/architecture-notes.md`. This split means the analyst agent's prompt stays target-agnostic.

4. **YAML has significant existing scaffolding.** 4 of 7 layers are complete (grammar, syntax, factory, parser). The parser includes an indentation-sensitive lexer — the hardest part of YAML parsing. Gaps are formatter (Layer 5), analyzer (Layer 6), and service integration (Layer 7). Layers 5 and 6 can be built in parallel.

5. **JSON is the right reference implementation.** Simplest end-to-end integration: no embedded languages, no semantic model, minimal formatting options, straightforward lint rules. CSS has embedded language complexity; JS has module graphs, type inference, and JSX. JSON's scope matches YAML's: data serialization format with comments and basic structure.

**Crystallizes:** The `references/{project}/` directory stabilizes into something that looks like reference material for a reusable skill. Not formalized as a skill yet — but recognizably reusable.

### Phase 3: Specification

**Build:**
- `lang-spec-writer` agent (or use doc-coauthoring directly and see where it falls short)

**Discover:**
- What a good spec looks like — the first spec IS the template
- What the evaluator-optimizer loop needs to check (the rubric emerges from the first real evaluation)
- Where the spec writer needs to pull in research output — this reveals the **data flow** between agents

**Discovered (from YAML spec writing):**

1. **Architecture notes are a prerequisite, not optional.** The extension contract is universal — it describes what any target needs. But the spec writer also needs target-specific state: what the parser already handles, what concerns differ from the reference implementation, where the gaps are. This drove creation of target-specific architecture notes as a mandatory input. Future targets should create architecture notes as the final Phase 2 deliverable, not defer to Phase 3.

2. **Spec organizes by layer, then by phase within each layer.** The natural structure mirrors both the extension contract's layer ordering and the research report's tier ranking. The template is now validated.

3. **Monolithic spec is correct for a single target.** At ~700 lines, the YAML spec covers all missing layers in one document. Splitting into per-layer specs would lose the cross-cutting sections (implementation order, testing strategy, open questions) that tie the layers together. A single document also makes it easier for an implementer to understand the full scope.

4. **Opus is the right model for spec writing.** The task requires synthesizing three documents (521 + 392 + 150 lines), maintaining internal consistency across 22 rules and 10+ format options, and making judgment calls about priorities and edge cases. This is a one-time artifact per target where quality matters more than speed.

5. **The spec-as-template pattern works.** The structure (Overview → Prerequisites → Layer-by-layer → Implementation Order → Testing → Open Questions) is reusable. The per-rule table format (name, category, severity, what it checks, config, edge cases, reference, target file) is the minimum information an implementer needs.

6. **Data flow is strictly sequential: extractor → analyst → spec-writer.** No shared state between agents. The spec writer reads files produced by the other two, never invokes them. Manual sequencing (run Phase 1, run Phase 2, run Phase 3) is sufficient — no orchestrating command needed yet.

**Crystallizes:** The first spec becomes a template. The rubric becomes the seed of the spec-methodology skill. The data flow between extractor → analyst → spec-writer reveals that manual sequencing is fine — no orchestrating command needed yet.

### Phase 4: Implementation

**Use:** Existing specialized agents (parser, formatter, lint rule engineers) — enhanced with `references/{target}/` directories containing spec output.

**Discover:**
- Where the existing agents fall short (missing grammar knowledge? missing project-specific patterns?)
- Mistakes that a hook could have prevented ("I started implementing before the spec covered this area")
- Whether the spec was detailed enough to guide implementation

**Discovered (from YAML implementation):**

1. **Three separate registration systems, not one.** Adding a language to Biome requires registering it in three independent codegen/macro systems: (a) `xtask/codegen` analyzer codegen, (b) `xtask/codegen` configuration codegen, and (c) `biome_configuration_macros` proc macro. Missing any one causes rules to silently not fire — no error, no warning, just absent from the enabled list. This was the hardest bug: `noDuplicateKeys` passed unit tests but was invisible to `biome lint` because the proc macro didn't know about `biome_yaml_analyze`. **Generalized lesson:** Any system with multiple independent registration mechanisms needs a cross-check gate. See [Cross-Cutting Concerns § III](#iii-process-patterns).

2. **The spec scoped correctly but the reference implementation was the real implementation guide.** The spec told us *what* to build. But *how* to build it required deep reading of the reference implementation at every step. The extension contract describes trait boundaries, not implementation patterns within each trait. Future targets need less spec and more annotated reference walkthrough.

3. **Formatter is harder than analyzer, by a wide margin.** The analyzer was straightforward: declare rule → query AST → collect state → emit diagnostic. One rule took ~100 lines and worked on the first try at the unit level. The formatter required understanding IR primitives, semantic indentation, and several debugging iterations. Formatter bugs produce "wrong output" not "error messages," making them harder to diagnose.

4. **Existing specialized agents were not used.** Implementation was done entirely by the main conversation agent. The specialized agents lack the project-specific context needed: the extension contract, the target's syntax tree structure, the registration system details. They're too generic for integration work. See [Agent Architecture](#agent-architecture) for full analysis.

5. **End-to-end testing caught what unit tests missed.** A rule passed its unit test on the first attempt. But the full tool reported zero diagnostics. The gap was the registration system (discovery #1). Similarly, a formatter smoke test passed but the full tool destroyed indentation. Every stage needs an end-to-end verification step, not just unit tests.

6. **Target-specific defaults must diverge from project globals.** YAML requires spaces for indentation (the spec forbids tabs). Biome's global default is tabs. Future specs should include a "defaults that differ from project globals" section.

7. **Stages 1-2 are truly parallel; Stages 3-4 are serial.** The formatter crate and analyzer crate have zero compile-time dependencies on each other. Configuration depends on both. Service integration depends on all three.

8. **Snapshot tests are configuration surface area guards.** Adding a new target as a valid config key caused existing snapshot tests to fail. This is good — it proves the test suite catches when new targets widen the configuration surface. Expect this and accept updated snapshots.

9. **Session recovery worked because of plan persistence.** The implementation spanned 3 sessions due to context compaction. Each time, the cached plan file plus the on-disk plan allowed seamless continuation. Without these files, each session would have required re-explaining the full strategy. This validates the plan persistence gate.

**Crystallizes:** The three-registration-system problem is the strongest candidate for a pre-implementation hook — it's non-obvious, silent on failure, and will recur for every new target. The end-to-end testing requirement is the second candidate. The specialized agents need project-specific knowledge injection before they're useful for this workflow.

### Phase 5: Post-Implementation Checkpoint

Renamed from "Review" per its own findings. This is a 15-minute checkpoint after Phase 4 completes, not a separate phase with its own agent.

**Checklist:**
1. Update methodology document with discoveries
2. Identify process issues
3. Note hook candidates
4. Flag what the spec missed

**Discovered (from YAML retrospective):**

1. **Retrospective happened organically, not via a formal agent.** Triggered by a single question after implementation was done. No dedicated agent was needed — the main conversation agent had full context. This suggests Phase 5 needs a **prompt/checklist**, not an agent.

2. **The methodology document IS the retrospective.** Findings were written directly into per-phase "Discovered" sections rather than a separate retrospective file. This is better — it keeps observations co-located with the phase they inform.

3. **No code review agent was needed for the first target.** The implementation was iterative: write code → compile → test end-to-end → fix → repeat. Each fix was immediately verified. The code reviewer may become valuable when (a) multiple contributors work on the same target, (b) the implementation is large enough that cross-cutting concerns are lost, or (c) conventions have been established and need enforcement.

4. **Process issues are cross-cutting observations, not phase-specific discoveries.** Plan persistence, compaction timing, and silent registration failures were all written after implementation, reflecting on the process. They live in [Cross-Cutting Concerns](#cross-cutting-concerns) rather than under any single phase.

5. **The lang code reviewer's original intent was a post-implementation quality gate.** It was designed to (a) check implementation against the spec, (b) check implementation against project conventions, (c) use confidence scoring to rank findings. This was modeled on 4 parallel specialist agents (style, logic, security, performance). In practice, the tight compile-test-fix loop made this unnecessary for a single implementer on the first target. The agent may become relevant for: reviewing a PR from someone else, auditing a second target's implementation against established patterns, or when the implementation exceeds one session's context.

6. **Phase 5 should be a checkpoint, not a phase.** The other phases produce concrete artifacts. Phase 5 produces observations folded into the methodology document. It doesn't need its own timeline slot.

**Crystallizes:** Phase 5 collapses into a post-implementation checklist rather than a separate phase with its own agent.

### Phase 6: Methodology Audit

Audit the overall approach used to complete the first target. Distinct from Phase 5's retrospective (implementation-level learnings) — this evaluates the methodology itself.

**Scope:**

| Dimension | What to evaluate |
|-----------|-----------------|
| **Agents** | Which were built, used, useful? Are definitions accurate to what was needed? |
| **Commands** | Does the research command work? Are per-phase commands worth building? |
| **Orchestration** | Was phase sequencing correct? Would a single orchestrating command be better? |
| **Reference materials** | Are reference docs reusable templates or one-off documents? What needs parameterization? |
| **Tools / utilities** | Did the development environment support the work? |
| **Documentation** | Is the file structure right? Are plan files useful artifacts or process overhead? |
| **Gates** | Were plans captured? Where did the process break down? |
| **Standards** | Testing, debugging, documentation — were they followed? Were they sufficient? |

**Discover:**
- Which parts of the methodology are load-bearing (remove them and the process breaks) vs. ceremonial (they exist but don't contribute)
- Whether the overhead pays for itself in reduced errors and faster work, or whether a simpler approach would have produced the same result faster
- What the minimum viable methodology is for the second target

**Crystallizes:** A methodology scorecard rating each component (agent, command, reference doc, gate) on: (a) was it used, (b) did it prevent an error or save time, (c) would its absence have been noticed. Components scoring low on all three are candidates for removal. Components scoring high become the core of the reusable toolkit.

### Phase 7: Replication

Run the same process for the second target. This is where the methodology proves or disproves its reusability.

**Discover:**
- Which parts of the process were truly reusable vs. accidentally first-target-specific
- Which reference documents need parameterization
- Whether the command structure works for a different target or needs redesign

**Crystallizes:** This is where the plugin manifest earns its existence. There are now enough components that bundling them makes sense. The `plugin.json`, the `${CLAUDE_PLUGIN_ROOT}` paths, the namespaced commands — all justified by actual reuse.

---

## Growth Path

```
Phase 1    agents/lang-feature-extractor.md
           commands/lang-research.md
           references/{target}/tools.md
           ─────────────────────────────────────── minimal viable toolkit

Phase 2    agents/lang-architecture-analyst.md
           references/{project}/extension-contract.md
           references/{target}/architecture-notes.md
           ─────────────────────────────────────── two-agent system

Phase 3    agents/lang-spec-writer.md
           references/{target}/architecture-notes.md   ← prerequisite discovered
           references/{target}/*-support-spec.md        ← first spec = template
           ─────────────────────────────────────── spec capability added

Phase 4    implementation crates built
           multiple registration systems identified as critical gotcha
           specialized agents NOT used (too generic for integration work)
           end-to-end testing > unit testing for integration bugs
           ─────────────────────────────────────── implementation complete, hooks identified

Phase 5    no new agent needed (checklist, not agent)
           findings folded into methodology document directly
           Phase 5 collapses into post-implementation checkpoint
           ─────────────────────────────────────── retrospective complete

Phase 6    methodology scorecard produced
(audit)    each component rated: used? prevented errors? missed if absent?
           deterministic gates formalized
           testing, debugging, environment gaps assessed
           ─────────────────────────────────────── methodology validated/pruned

Phase 7    skills/feature-comparison/SKILL.md       ← crystallized from pattern
(2nd       skills/integration/SKILL.md              ← crystallized from reuse
target)    skills/spec-methodology/SKILL.md         ← crystallized from rubric
           hooks/pre-implementation-check            ← crystallized from pain
           commands/lang-dev.md                      ← orchestrator emerges
           plugin.json                               ← bundle justified
           ─────────────────────────────────────── full plugin
```

---

## Cross-Cutting Concerns

### I. Deterministic Gates

Gates are enforcement mechanisms that fire at specific points in the workflow. They convert "we should remember to..." into "this must pass before continuing." Each gate is BLOCKING — failure halts progress until resolved.

**Origin:** The plan persistence problem recurred in every phase of the YAML implementation because there was no enforcement mechanism. All gates below were identified from concrete failures or near-misses.

#### Gate taxonomy

| Gate | Fires at | What it enforces | Maturity origin |
|------|----------|-----------------|-----------------|
| [Plan Capture](#gate-1-plan-capture) | Phase start | Plan written to disk before work begins | Process issue: plan persistence |
| [Prerequisite Check](#gate-2-prerequisite-check) | Phase start | Previous phase artifacts exist | Phase 3 discovery #1 |
| [Environment Readiness](#gate-3-environment-readiness) | Pre-implementation | Required tools installed and functional | Open question #3 + #4 findings |
| [Stage Start](#gate-4-stage-start) | Each implementation stage | Test harness and debugging workspace exist | Testing standard + debugging standard |
| [Debug Hygiene](#gate-5-debug-hygiene) | Each commit | No debug artifacts in committed code | Debugging standard |
| [Phase Summary](#gate-6-phase-summary) | Phase end | Summary with resumption instructions written | Context management standard |
| [Code Quality](#gate-7-code-quality) | Implementation end | Format, lint, snapshot, and inline tests pass | External standards compliance |
| [PR Readiness](#gate-8-pr-readiness) | Pre-contribution | Full validation equivalent to project CI | External standards compliance |

#### Gate 1: Plan Capture

Before any implementation work begins:
1. Generate the implementation plan for this phase
2. Write it to `kb/tasks/{target}/phase{N}-{description}.md`
3. Read the file back to confirm it was written
4. If the file does not exist or is empty, STOP and report the failure
5. Only after confirmation, proceed

**Why this is gated:** The agent optimizes for getting to implementation quickly. Plan writing feels like overhead rather than a deliverable. This gate reframes it: the plan file IS a deliverable, not a side effect. This problem recurred in every phase of the YAML implementation.

#### Gate 2: Prerequisite Check

Before starting Phase N, verify Phase N-1 artifacts exist. For Biome language integration:

| Phase | Required artifacts |
|-------|--------------------|
| 2 | `references/{language}/feature-research-report.md` |
| 3 | `references/{project}/extension-contract.md` + `references/{language}/architecture-notes.md` |
| 4 | `references/{language}/*-support-spec.md` |
| 5 | Compiled crates (verify via build command) |

#### Gate 3: Environment Readiness

Verify before any implementation begins:

- [ ] Formatter/linter installed (`cargo fmt --version`, `cargo clippy --version`)
- [ ] Snapshot test tool installed (`cargo insta --version`)
- [ ] Macro expansion tool installed (`cargo expand --version`, nightly toolchain present)
- [ ] Task runner installed (`just --version`) OR raw equivalents documented
- [ ] Debug profile configured (`[profile.debugging]` in root config)
- [ ] Test output flags work (`cargo test -- --show-output`)
- [ ] Fuzz testing tool installed (`cargo-fuzz`)

If any check fails, install the missing tool before proceeding.

**Why `cargo expand` is gated:** The three-registration-system bug during YAML implementation cost 2+ hours. `cargo expand` would have shown the generated struct missing the new rule immediately — jumping from "rule doesn't fire" to the answer in one command. Reactive tool installation wastes hours.

#### Gate 4: Stage Start

Before implementing any component within a stage:

- [ ] `tests/quick_test.rs` exists for this crate (with `#[ignore]`)
- [ ] Test harness exists (snapshot test infrastructure for the crate)
- [ ] Previous stage compiles successfully

#### Gate 5: Debug Hygiene

Before any commit within implementation:

- [ ] No `dbg!()`, `eprintln!()`, or `println!()` in modified source files
- [ ] No `#[ignore]` removed from `quick_test.rs` (should remain ignored in committed code)
- [ ] No hardcoded paths or debug-specific configuration in test files

#### Gate 6: Phase Summary

After phase work completes:

1. Write summary to `kb/tasks/{target}/phase{N}-{description}-summary.md` containing:
   - **Completed work:** What was built, with file paths
   - **Planned but deferred work:** Items from the plan not implemented
   - **Discovered work:** New tasks found during execution
   - **Artifacts produced:** Files created or modified
   - **Resumption instructions:** How to continue (see template)
2. Update methodology discoveries if applicable

**Resumption instructions template:**

```markdown
## Resumption instructions
To continue from where this phase left off:
1. Read this summary for completed/deferred/discovered work
2. Read the plan at kb/tasks/{target}/phase{N}-{description}.md
3. Verify previous stage: [specific verification command]
4. Current stage: [stage name and number]
5. Next action: [specific next step]
```

#### Gate 7: Code Quality

Before declaring implementation complete:

- [ ] Formatter check passes for all modified crates
- [ ] Linter passes with zero warnings on modified crates
- [ ] All snapshot tests pass and are accepted
- [ ] No TODO/FIXME/HACK comments in production code (or each tracked as deferred work)
- [ ] Inline smoke tests exist and pass for each major component
- [ ] Fuzz targets exist and run clean for 5+ minutes

#### Gate 8: PR Readiness

Full validation equivalent to project CI:

- [ ] Formatter check passes (all crates)
- [ ] Linter passes (workspace-wide or affected crates)
- [ ] Test suite passes for all affected crates
- [ ] Snapshot tests accepted
- [ ] End-to-end verification on sample inputs
- [ ] No debug artifacts in code
- [ ] Changeset/changelog entry created (if required by project)

### II. Context Management

**Decision:** Use `/clear` at phase boundaries, not `/compact`.

When deterministic gates ensure all decision-relevant state is on disk, a clean context reset is strictly better than lossy compression:

| | `/compact` | `/clear` |
|---|---|---|
| Context freed | Partial | 100% |
| What survives | Lossy summary (uncontrolled) | Nothing — disk artifacts only |
| Predictability | Low — don't know what was kept | High — clean slate |
| Risk of lost info | Medium — compression may drop key details | Zero if gates fired; total if they didn't |

**The pattern:**

```
[phase work] → [gate fires: artifacts to disk] → /clear → [new context reads artifacts]
```

After a `/clear`, the next message can be as simple as: "Continue from `kb/tasks/{target}/phase{N}-summary.md`" — the agent reads the file, orients, and resumes. No compressed context needed.

**Required artifacts per boundary:**

| Boundary | Required on disk |
|----------|-----------------|
| Phase 1 → 2 | Feature research report |
| Phase 2 → 3 | Extension contract + architecture notes |
| Phase 3 → 4 | Support spec + implementation plan |
| Mid-stage | Plan file + compiled output |
| Phase 4 → 5 | Phase summary with resumption instructions |

**When `/compact` is still appropriate:**
- Mid-stage when no gate boundary has been reached but context is large
- During research phases where artifact boundaries are less defined
- As an automatic safety net — don't disable it, just don't plan around it

**Key principle:** Compaction is safe when all decision-relevant information exists on disk, not just in conversation memory. If `/compact` is the recovery mechanism, the gates need fixing.

### III. Process Patterns

**Plan persistence:** Plans must be written to disk as the FIRST action when entering any new phase. The plan file IS a deliverable, not a side effect of planning. See [Gate 1](#gate-1-plan-capture).

**Silent failure detection:** Some systems fail silently — code compiles, unit tests pass, but the feature doesn't work at runtime. For Biome, three independent registration mechanisms (codegen, configuration codegen, proc macro) mean a new language's rules can be invisible without any error. A post-codegen verification step that checks all registration points prevents hours of debugging. **Generalized:** Any system with multiple independent registration mechanisms needs a cross-check gate.

**Session recovery:** Implementation spanning multiple sessions works when durable artifacts exist on disk. The cached plan file plus `kb/tasks/` files enable seamless continuation. Without these, each session requires re-explaining the full strategy.

**Phase execution dependencies:**

```
Phase 1 ──┐
           ├──→ Phase 3 ──→ Phase 4 ──→ Phase 5 ──→ Phase 6 ──→ Phase 7
Phase 2 ──┘
```

Phases 1 and 2 are independent and parallelizable. Phase 3+ are strictly sequential. Within Phase 4, component stages (formatter, analyzer) are parallelizable; integration stages (configuration, service wiring) are serial.

---

## Development Standards

### A. Testing

**What it governs:** Test infrastructure creation, timing, layering, and coverage expectations.

**Why it matters:** End-to-end testing catches what unit tests miss. The YAML implementation's most critical bug (silent rule registration failure) passed unit tests but failed end-to-end. Testing infrastructure created early pays compound returns; deferred testing creates compounding debt.

**Leverage:** High — testing findings cross-reference into every other standard.
**Accumulation:** High — untested code compounds risk with every change.

#### Five testing layers

| Layer | Type | When to create | What it catches | Parallelizable |
|-------|------|----------------|-----------------|----------------|
| 1 | Inline smoke tests (`#[test]` in `lib.rs`) | During each stage | API surface, basic round-trip | No |
| 2 | Quick tests (`tests/quick_test.rs`) | Stage start | Individual behavior, debugging aid | No (interactive) |
| 3 | Snapshot/fixture tests (`tests/spec_tests.rs` + `tests/specs/`) | Stage start (harness), incremental (fixtures) | Regression, correctness | **Yes** |
| 4 | Integration/E2E tests | After all components wired | Full pipeline, cross-module interaction | No |
| 5 | Fuzz tests | As early as possible, runs continuously | Unknown unknowns, edge cases | **Yes** (background) |

**Key distinctions:**
- Layers 1-4 are **point-in-time**: written once, run at test time, value fixed at creation
- Layer 5 is **continuous**: created once, runs in background, value grows over time as it discovers edge cases
- Layers 1-4 catch known failure modes. Layer 5 catches **unknown unknowns**

#### Fuzzing for format-sensitive targets

Languages where formatting is semantic (YAML, Python, Haskell) get higher value from fuzzing than those with explicit delimiters (JSON, JavaScript):

1. **Formatting can change meaning** — shifting indentation silently restructures data
2. **Grammar ambiguity** — combinatorial interactions between features (block scalars, implicit keys, flow-in-block contexts)
3. **Spec version differences** — ambiguous patterns surface naturally in fuzzed input

Four fuzzing levels, built incrementally:

| Level | Technique | When to create | Effort |
|-------|-----------|----------------|--------|
| 1a | Unstructured parser fuzzing (random bytes → parse) | Pre-implementation | ~20 lines |
| 1b | Unstructured formatter fuzzing (parse → format → re-parse → re-format) | End of formatter stage | ~40 lines |
| 2 | Corpus-based (mutate real files as seeds) | End of integration stage | Low |
| 3 | Property-based (generate valid structures programmatically) | Post-implementation | Medium |

**The harvest loop:** Fuzzing generates test cases, not just bug reports. Each minimized reproducer becomes a permanent snapshot fixture (Layer 3). The fixture suite grows organically without additional human effort. Edge cases from one target inform fixture design for the next.

#### Test timing per implementation stage

```
Pre-implementation:
  ├── Layer 5: parser fuzz target (if parser exists)
  └── GATE: verify test tooling installed (Environment Readiness)

Stage 1 (first major component):
  ├── START: create test harness + quick_test.rs (Stage Start gate)
  ├── DURING: add fixtures after each group of implementations
  ├── END: inline smoke test, verify all fixtures pass
  └── END: create component fuzz target

Stage 2 (second major component):
  ├── START: create test harness (Stage Start gate)
  ├── DURING: add valid/invalid fixtures per feature
  ├── END: verify fixtures pass, add suppression tests
  └── END: update fuzzer with cross-component checks

Integration stage:
  ├── END: end-to-end verification (minimum manual, ideally automated)
  └── END: seed fuzz corpus with real-world files

Post-implementation:
  └── Harvest fuzzer reproducers as permanent fixtures
```

#### YAML implementation: current testing gap

The largest uncaptured work item from Phase 4. Current state:

**Exists:** 1 inline smoke test (formatter), 1 inline quick_test (analyzer).

**Missing:**
- Formatter: test harness (`spec_tests.rs`, `spec_test.rs`, `language.rs`, `quick_test.rs`), fixture directory with 10-15 `.yaml` + `.snap` pairs
- Analyzer: test harness, valid/invalid fixtures per rule, suppression comment tests
- Integration: CLI integration tests, configuration resolution tests, file detection tests
- Fuzz: no fuzz targets, no corpus

The reference implementation (JSON) has ~504 lines of test infrastructure and ~200 fixture files.

### B. Debugging

**What it governs:** Debugging tool availability, systematic approaches, and debug artifact hygiene.

**Why it matters:** Reactive debugging (install tools after a bug appears) wastes hours. The `cargo expand` gap during YAML implementation cost 2+ hours on a single bug that would have been immediately visible in expanded macro output. Proactive setup converts debugging from emergency response to routine verification.

**Leverage:** Medium — debugging findings inform testing and gate design.
**Accumulation:** Low — debugging debt doesn't compound (each bug is independent).

#### Tools and techniques

| Technique | Purpose | Setup timing |
|-----------|---------|-------------|
| `dbg!()` macro | Print expression + file:line automatically | Built-in (prefer over `eprintln!`) |
| `cargo test -- --show-output` | Show debug output from passing tests | Verify at Environment Readiness gate |
| `cargo expand` | Show macro-expanded code | Install at Environment Readiness gate (requires nightly) |
| `--profile debugging` | Preserve debug symbols for stack traces | Verify config exists at Environment Readiness gate |
| `cargo insta review` | Interactive snapshot review | Install at Environment Readiness gate |
| `tests/quick_test.rs` | Ad-hoc debugging workspace | Create at Stage Start gate |

#### Debugging decision tree

When a feature doesn't work at runtime:

1. Does the unit test pass? → If no, fix the implementation
2. Does `cargo expand` show the feature in generated code? → If no, check registration/codegen
3. Is the feature in the configuration/enum? → If no, run codegen
4. Is the feature enabled at runtime? → If no, check proc macro / runtime registration

**Key principle:** Don't wait for a bug to install debugging tools. Verify them at Phase 4 start. Don't wait for confusion to create `quick_test.rs`. Create it at Stage 1/2 start. The 5 minutes spent setting up debugging infrastructure saves hours when something goes wrong.

### C. Documentation

**What it governs:** Documentation coverage and quality at API, module, and architectural levels.

**Why it matters:** Documentation is the primary onboarding tool. Undocumented code forces every new contributor to reverse-engineer intent from implementation.

**Leverage:** Medium — documentation enables others to contribute.
**Accumulation:** Medium — documentation debt grows linearly with code changes.

For Biome specifically:

- `RUSTDOCFLAGS='-D warnings'` in CI — broken intra-doc links fail the build
- Rule documentation has rigid structure enforced by codegen
- No `missing_docs` lint — documentation on public items is voluntary
- Implied standard: document public API of core infrastructure crates; language-specific crates are undocumented by convention

Documentation does not need separate timing/parallelization analysis. It is either co-located with code (validated by CI) or optional follow-up. Existing gates (Code Quality, PR Readiness) cover documentation requirements.

### D. External Standards Compliance

**What it governs:** Adherence to the target project's contribution requirements.

For Biome, this is captured in `kb/tasks/biome-contrib-spec.md` — an extraction of 18 source files covering 80+ instructions. Key sections mapped to phases:

| Spec Section | When It Matters |
|---|---|
| Environment & Workflow | Pre-implementation (Environment Readiness gate) |
| Testing | Implementation stages (test infrastructure) |
| Formatter Development | Formatter stage |
| Analyzer / Lint Rules | Analyzer stage |
| Contribution Process | PR Readiness gate |
| Code Style & Lint Policy | Every commit |
| CI/CD Requirements | PR Readiness gate |
| Diagnostics | Analyzer stage |

**Generalized:** Any target project with contribution guidelines should have those guidelines extracted, categorized by phase relevance, and integrated into the appropriate gates.

### Environment requirements

**What it governs:** Development environment tooling completeness.

During the YAML implementation, a lightweight container was used instead of the project's full devcontainer. The most significant gap was `just` (the project's task runner) — its absence meant `just ready` (the pre-PR validation command) was never run. `cargo-expand` was also missing.

**Recommendations for future work:**
1. Install `just` in any development container (one line in `postCreateCommand`)
2. Ensure `cargo-expand` is available (requires nightly toolchain)
3. Run the project's equivalent of `just ready` before any PR
4. Keep the lightweight container for research phases (1-3) where build tools aren't needed; use the full devcontainer for implementation phases (4+)

---

## Agent Architecture

### Current inventory and utility assessment

Three specialized agents exist: `cst-parser-engineer`, `biome-lint-engineer`, `ir-formatter-engineer`. Each has two layers of knowledge:

1. **Generic domain expertise** in its system prompt — parser theory, IR formatting algorithms, lint rule patterns
2. **Project-specific CONTRIBUTING.md** referenced via `@` — step-by-step instructions for adding a parser, formatter, or lint rule

The CONTRIBUTING docs are genuinely valuable. They explain codegen steps, testing patterns, file naming conventions, and project-specific macros.

### What agents are missing

Five categories of knowledge needed during implementation but not in any agent:

1. **Target-specific syntax tree** — What nodes exist, what fields they have, what AST shapes look like. Lives in the `.ungram` file and generated syntax types, not in any agent prompt.
2. **The extension contract** — How the project's layers connect, which traits to implement, how service methods compose. Lives in reference docs but isn't referenced by agents.
3. **Cross-crate registration** — The multi-registration-system problem. None of the agents spans more than one crate.
4. **Reference implementation patterns** — Not abstract "here's how formatting works" but concrete "the reference uses `block_indent` for nested objects and always adds a trailing newline."
5. **End-to-end integration context** — The hard work is wiring multiple crates together, not implementing individual nodes or rules.

### The fundamental mismatch

These agents are designed as **narrow specialists** (format this node, write this lint rule). But new-target integration work is **cross-cutting** (wire everything together across formatter + analyzer + configuration + service). The narrow tasks they're good at are the *easy* parts. The hard parts — registration, service wiring, debugging silent failures, choosing the right IR primitives — are outside their scope.

If you already know *what* to implement, the individual implementations are straightforward enough to not need a specialist agent. If you *don't* know what to implement, the specialist agents can't help because they lack integration context.

### When agents become valuable

Value increases along three axes:

1. **Repetition within a target** — After the first 3 formatter nodes are implemented, the remaining follow a pattern. At scale (>20 nodes, >5 rules), a specialist agent doing parallel work saves real time.
2. **Second target onward** — Once patterns are established, agents can be enhanced with "here's how the first target did it, follow the same pattern."
3. **Multiple contributors** — If different people work on formatter vs. analyzer, the agents' narrow scope becomes a feature: each contributor gets focused guidance.

### Options

| Option | Description | Cost | Risk |
|--------|-------------|------|------|
| **1: Inject context** | Add extension contract and architecture notes as `@` references. Keep agents narrow but integration-aware. | ~3 references per agent (~1500 lines) | Context bloat |
| **2: Batch narrow tasks** | Don't use during scaffolding. Use after scaffolding for batch work ("implement these 20 nodes"). | Zero changes | Requires manual coordination |
| **3: Single broad agent** | Merge all three into one that understands parser + formatter + analyzer + integration. | One agent to maintain | Prompt too large, dilutes guidance |
| **4: Demote to docs** | Remove agent wrappers. Add CONTRIBUTING docs as reference documents any agent can read. | Simpler system | Lose parallel delegation capability |
| **5: Defer decision** | Keep as-is, evaluate after second target. | Zero | May remain unused artifacts |

**Current recommendation:** Option 5 (defer). The honest answer is that value is unknown until the second target reveals whether narrow-specialist or cross-cutting-integration work dominates. If the second target's hard parts are again integration (likely), Option 4 is the right call. If the hard parts shift to volume, Option 2 justifies keeping them.

**Evaluation summary:**

| Condition | Utility |
|-----------|---------|
| Current (first target, single implementer) | Low — never invoked |
| Second target (established patterns) | Medium — batch tasks save time if volume is high |
| At scale (toolkit used by others) | High — guided entry points for new contributors |

### Crystallization heuristics

Patterns for when informal practices become formalized components:

| Component | Crystallizes when | Trigger type |
|-----------|-------------------|-------------|
| **Skills** | A pattern is used a second time | Repetition |
| **Hooks** | A mistake happens once | Pain |
| **Commands** | A manual sequence is repeated enough to formalize | Routine |
| **Plugin manifest** | There are enough components to bundle | Mass |

None of these are speculative — they are all responses to observed needs.

---

## Process Architecture

### Command structure

**Current state:** One command exists: `/lang-research` — it orchestrates Phase 1 (feature extraction). It is fully functional and validated. No commands exist for Phases 2-5.

**Proposed commands:**

| Command | Phase | What it does |
|---------|-------|-------------|
| `/lang-research <target>` | 1 | Feature extraction (exists) |
| `/lang-architecture <target>` | 2 | Architecture analysis against project internals |
| `/lang-spec <target>` | 3 | Spec writing from Phase 1+2 outputs |
| `/lang-implement <target>` | 4 | Implementation from spec |
| `/lang-review <target>` | 5 | Post-implementation checkpoint |
| `/lang-audit <target>` | 6 | Methodology audit |

Each command follows the same internal structure:

```
1. GATE: Plan Capture — write plan to disk
2. GATE: Prerequisite Check — verify previous phase artifacts
3. Execute phase work
4. GATE: Phase Summary — write summary with resumption instructions
5. GATE: Update methodology discoveries
```

**Where `/lang-research` fits:** It IS Phase 1 — the Phase 1 trigger. Its internal phases (setup → clarifying questions → extraction → synthesis → completion) are sub-steps within the overall Phase 1.

### Plan storage

Plans are stored in two places:

| Location | Purpose | Durability |
|----------|---------|-----------|
| `.claude/plans/` | Claude Code's plan mode working buffer | Survives compaction within a session; single-file store (new plan overwrites previous) |
| `kb/tasks/{target}/phase{N}-*.md` | Durable git-tracked archives | Survives across all sessions |

The internal plan store works for session recovery but is not a reliable archive. The `kb/tasks/` files are the only reliable record.

### Documentation directory convention

```
kb/tasks/
├── agent-guided-development-framework.md  # This document (cross-target methodology)
├── quality-audit-framework.md             # Quality audit reference
├── biome-contrib-spec.md                  # External standards extraction
├── {target}/
│   ├── phase1-{description}.md            # Phase 1 plan
│   ├── phase1-{description}-summary.md    # Phase 1 outcomes
│   ├── phase2-{description}.md            # Phase 2 plan
│   ├── phase2-{description}-summary.md    # Phase 2 outcomes
│   ├── ...
│   └── phase5-review-summary.md           # Post-implementation checkpoint
├── {second-target}/                       # Second target follows same structure
│   └── ...
└── ...
```

Convention: `phase{N}-{description}.md` for plans, `phase{N}-{description}-summary.md` for outcomes. Target-specific files go in `kb/tasks/{target}/`, methodology files stay at `kb/tasks/`.

---

## Design Principles

Six principles observed across the first implementation and methodology development:

1. **Gates over memory** — enforce through mechanisms, not through the agent remembering to check. Every process failure during YAML implementation traced to a missing gate.

2. **Disk over context** — decision-relevant state belongs on disk, not in conversation memory. This enables clean context resets and multi-session recovery.

3. **Narrow agents, broad integration** — specialist agents work for batch tasks within established patterns. Integration work requires full-context agents. Don't force narrow tools onto cross-cutting problems.

4. **Spec for scope, reference for implementation** — specs tell you *what* to build; reference implementations tell you *how*. Both are needed. The spec alone is insufficient.

5. **End-to-end before unit** — as a verification strategy (not a development strategy). Unit tests passing is necessary but not sufficient. Every stage needs end-to-end verification to catch registration, wiring, and configuration gaps.

6. **Process through crystallization** — skills, hooks, commands, and plugin manifests emerge from observed needs, not from speculative design. Build the first instance, observe what's reusable, then formalize.

These principles complement the [Quality Audit Framework](quality-audit-framework.md)'s design principles. Where the QAF emphasizes **audit-first** and **severity-driven prioritization** for quality evaluation, this framework emphasizes **gate-driven enforcement** and **experience-driven formalization** for development methodology.

---

This document is an **attractor** in the dynamical systems sense — a set of states toward which the methodology tends to evolve, regardless of starting conditions. Once a project adopts deterministic gates and phase-structured development, deviation requires more effort than adherence. The gates don't enforce compliance — they make compliance the path of least resistance.

| Property of an attractor | Property of this framework |
|---|---|
| Acts at a distance — influences before contact | Shapes agent behavior before every section is read |
| Creates orbits — stable, productive trajectories | Phases settle into consistent patterns without rigid enforcement |
| Deviation requires energy — leaving is harder than staying | Once gates exist, skipping them is more work than following them |
| Proportional to mass — more complete means stronger pull | Each phase's discoveries strengthen the next phase's foundation |
| Doesn't require active enforcement | The gates themselves make the right thing the easy thing |
