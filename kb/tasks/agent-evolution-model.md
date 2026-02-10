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

**Crystallizes:** Nothing yet. The agent definition and its output are both drafts. The output format is a prototype.

### Phase 2: Architecture Analysis

**Build:**
- `lang-architecture-analyst` agent
- `references/biome/` directory capturing what the JSON/CSS study revealed about biome's extension contract

**Discover:**
- What questions the spec writer will need answered
- Whether the architecture analyst and feature extractor need to talk to each other (routing/chaining) or are truly independent
- What biome knowledge is language-specific vs. universal

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
