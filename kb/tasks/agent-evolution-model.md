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

**Crystallizes:** The first spec becomes a template. The rubric becomes the seed of the **spec-methodology skill**. The data flow between extractor → analyst → spec-writer reveals whether an orchestrating command is needed or whether manual sequencing is fine.

### Phase 4: Implementation

**Use:** The existing cst-parser-engineer, biome-lint-engineer, ir-formatter-engineer — enhanced with `references/yaml/` directories containing spec output.

**Discover:**
- Where the existing agents fall short (missing YAML grammar knowledge? missing biome-specific patterns?)
- Mistakes that a hook could have prevented ("I started implementing before the spec covered this area")
- Whether the spec was detailed enough to guide implementation

**Crystallizes:** Hook patterns become obvious — the exact rules are known because the pain was just experienced. The **pre-implementation-check hook** isn't speculative anymore; it's a direct response to something that went wrong.

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
           references/yaml/spec-template.md        ← emerged from first spec
           ─────────────────────────────────────── spec capability added

Phase 4    references/yaml/ on existing agents
           hooks/ identified but not yet built
           ─────────────────────────────────────── existing agents enhanced

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

## Crystallization heuristics

- **Skills** crystallize when a pattern is used a second time
- **Hooks** crystallize when a mistake happens once
- **Commands** crystallize when a manual sequence is repeated enough to formalize
- **The plugin manifest** crystallizes when there are enough components to bundle
- None of these are speculative — they are all responses to observed needs

## Revised assessment

**Option E: Incremental toward a generalized plugin.** Build using YAML as the first language. Use language-agnostic naming. Let each phase inform the next. The plugin-dev toolkit's `${CLAUDE_PLUGIN_ROOT}` pattern and the skill-creator's progressive disclosure (`references/` directories per language) are the two most important design patterns — they are exactly how a reusable toolkit gets parameterized.
