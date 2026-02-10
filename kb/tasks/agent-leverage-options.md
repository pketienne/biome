# Agent Leverage Options

Options for leveraging available agents and reference patterns toward the YAML support effort in Biome: feature extraction from cloned sources, research, planning/design/specs, documentation, code review, commands, agent development, and orchestration.

## Inventory of Available Assets

### Existing Biome Agents (in both `main` and `yaml-revisited`)

| Agent | Focus | Strengths | Gaps for YAML effort |
|-------|-------|-----------|---------------------|
| **cst-parser-engineer** | CST parser design, error recovery | Deep parser theory, biome parser patterns | Generic — no YAML-specific knowledge, no research capability |
| **biome-lint-engineer** | Lint rules, assist actions, analyzer | Biome analysis pipeline expertise | JS/TS-centric, no feature extraction or spec writing |
| **ir-formatter-engineer** | IR-based formatting | Biome formatter pipeline expertise | No research, no cross-tool comparison capability |

These three cover **implementation** well but have no support for **research**, **feature extraction**, **spec writing**, **orchestration**, or **progress tracking**.

### Reference Patterns from `~/Clones/anthropics/`

| Source | Pattern | Key Takeaway |
|--------|---------|-------------|
| **feature-dev** | 7-phase workflow with 3 agents (explorer, architect, reviewer) | Closest match to the overall workflow needed. Phases 1-4 (discovery through architecture) map directly to the research/design phase of YAML support |
| **plugin-dev** | 8-phase workflow with 3 agents + 7 skills | Shows how to build a **self-contained toolkit** — agents + skills + commands + hooks working together |
| **code-review** | 4 parallel agents with confidence scoring | Shows how to run **parallel specialist agents** and filter false positives — applicable to parallel feature extraction |
| **agent patterns** | Orchestrator-workers, evaluator-optimizer, prompt chaining, routing | Core coordination patterns. Orchestrator-workers for research delegation; evaluator-optimizer for spec refinement |
| **research lead/subagents** | Depth-first vs breadth-first research with OODA loop | Directly applicable to feature extraction across 13 cloned repos |
| **chief-of-staff** | CLAUDE.md memory, output styles, plan mode, hooks, subagent delegation | Model for the **tracking/coordination** agent — persistent state, multiple output audiences |
| **skill-creator** | Progressive disclosure, output patterns, workflow templates | Design methodology for any new skills |
| **doc-coauthoring** | 3-stage collaborative writing (gather → refine → test) | Directly applicable to spec writing |
| **cookbook-audit** | 4-dimension scoring rubric + validation script | Model for progress evaluation and quality gates |
| **hookify** | Rule-based behavior enforcement via hooks | Could enforce workflow discipline (e.g., require research before implementation) |
| **mcp-builder** | 4-phase development with evaluation harness | Evaluation harness pattern transferable to validating YAML support completeness |

## Options

### Option A: Minimal — Extend Existing Agents + Add Commands

**Add 2 new agents, 2 commands. No skills, no plugin structure.**

New agents:
- **yaml-research-analyst** — Based on feature-dev's `code-explorer` + research patterns' `research-lead`. Searches cloned repos, extracts features, compares tools, produces structured comparison reports. Read-only tools.
- **yaml-spec-writer** — Based on `doc-coauthoring` + `chief-of-staff`. Takes research output and produces specs, tracks progress against a rubric, maintains a living feature matrix.

New commands:
- `/yaml-research <topic>` — Orchestrates the research analyst against cloned repos and biome source
- `/yaml-status` — Generates a progress report using the spec writer agent

**Pros:** Fast to build, low overhead, easy to iterate. Builds on existing agents without disrupting them.
**Cons:** No automated quality gates, no parallel extraction, limited orchestration. Commands are simple dispatchers, not full workflows.

### Option B: Feature-Dev Fork — 7-Phase Workflow Adapted for YAML

**Fork the feature-dev pattern. 5 agents, 1 orchestrating command, structured phases.**

Agents:
1. **yaml-feature-extractor** — Based on `code-explorer` + `research-lead`. Runs breadth-first across cloned repos, extracts features per tool into structured output. Can spawn subagents per repo.
2. **yaml-architecture-analyst** — Based on `code-architect`. Studies biome internals (JSON/CSS as reference languages), maps extension points, produces architecture blueprints.
3. **yaml-spec-writer** — Based on `doc-coauthoring` + `chief-of-staff`. Produces specs from research, uses evaluator-optimizer loop for refinement.
4. **yaml-code-reviewer** — Based on `code-reviewer` + confidence scoring. Reviews YAML implementation code against biome conventions and spec requirements.
5. Existing **cst-parser-engineer**, **biome-lint-engineer**, **ir-formatter-engineer** unchanged — used during implementation phases.

Command:
- `/yaml-dev [phase] [topic]` — Orchestrates the 7-phase workflow:
  1. **Discovery** — What YAML features are we targeting?
  2. **Feature Extraction** — yaml-feature-extractor scans all cloned repos
  3. **Architecture Analysis** — yaml-architecture-analyst maps biome internals
  4. **Spec Writing** — yaml-spec-writer produces implementation specs
  5. **Implementation** — Delegates to existing parser/lint/formatter engineers
  6. **Review** — yaml-code-reviewer validates against spec + conventions
  7. **Summary** — Progress report, updated feature matrix

**Pros:** Complete lifecycle coverage. Phase gates ensure research precedes implementation. Leverages proven feature-dev pattern. Clear separation of concerns.
**Cons:** More upfront work to build. Heavier machinery for early-stage research. May be premature to define all phases before understanding the scope.

### Option C: Plugin — Full Self-Contained Toolkit

**Build a `yaml-dev` plugin following the plugin-dev pattern. Agents + skills + commands + hooks.**

Everything from Option B, plus:

Skills:
- **yaml-feature-comparison** — Structured output templates for feature extraction reports, with progressive disclosure references for each cloned tool
- **biome-language-integration** — Reference knowledge about biome's extension contract (parser → CST → formatter IR → analyzer rules), pulled from JSON/CSS study
- **yaml-spec-methodology** — Rubric for evaluating spec completeness (based on cookbook-audit's scoring pattern)

Hooks:
- **pre-implementation-check** — Blocks implementation tool calls if no spec exists for the target component (hookify pattern)
- **post-edit-validation** — Runs biome-specific checks after edits to YAML crate files

Commands:
- `/yaml-dev:research <topic>` — Feature extraction workflow
- `/yaml-dev:architecture` — Biome internal analysis
- `/yaml-dev:spec <component>` — Spec writing with evaluator-optimizer refinement
- `/yaml-dev:status` — Progress tracking with rubric scoring
- `/yaml-dev:review` — Code review with confidence scoring

**Pros:** Most comprehensive. Self-documenting. Hooks enforce workflow discipline. Skills provide reusable knowledge. Distributable as a unit.
**Cons:** Heaviest upfront investment. Risk of over-engineering before the problem is fully understood. Plugin structure adds complexity that may not pay off for a single-project effort.

### Option D: Incremental — Research-First, Build As Needed

**Start with only what's needed now (feature extraction), add agents/skills as gaps emerge.**

Phase 1 (now):
- **yaml-feature-extractor** agent — Focused solely on scanning cloned repos and producing structured feature reports. Based on research-lead pattern with breadth-first extraction.
- `/yaml-research` command — Simple dispatcher

Phase 2 (after feature extraction complete):
- **yaml-architecture-analyst** agent — Studies biome internals using JSON/CSS as reference
- Decide whether to add spec-writing agent or use doc-coauthoring skill directly

Phase 3 (when implementation begins):
- Enhance existing parser/lint/formatter engineers with YAML-specific knowledge via `references/` directories
- Add review and tracking agents if the project scope warrants them

**Pros:** Lowest risk. Each addition is informed by actual needs. No wasted effort building agents that turn out unnecessary. Fastest time to first useful output.
**Cons:** Less cohesive than a designed-upfront system. May accumulate ad hoc structure. Requires discipline to stop and build tooling when needed rather than pushing through manually.

## Assessment

**Option D (Incremental) is the strongest starting point**, with Option B as the target architecture. Reasons:

1. The immediate need is **feature extraction** from 13 cloned repos — none of the existing agents do this
2. Architecture analysis requires first understanding what features matter — depends on extraction results
3. The feature-dev 7-phase pattern is proven and maps well, but committing to all 7 phases now would be speculative
4. The plugin structure (Option C) adds overhead that's justified for reusable/distributed tooling, not for a single focused project
5. Building incrementally lets each agent's design be informed by what the previous phase actually produced

The research-lead + subagent pattern from the agent patterns notebooks is the highest-value pattern to apply first — it directly solves "extract features from 13 repos in parallel."
