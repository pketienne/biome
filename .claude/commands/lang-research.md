---
description: Research language tooling ecosystem by extracting features from cloned reference repos
argument-hint: <language> [focus-area]
---

# Language Tooling Research

Conduct a systematic research workflow to extract and compare features from cloned reference tool repositories for a target language. Use the lang-feature-extractor agent for extraction and produce a comprehensive feature report.

Language: $1
Focus area (optional): $2

## Gate 1: Plan Capture (BLOCKING)

1. Generate a research plan for $1 covering: which repos to scan, how to group them, what extraction depth to use, what spec versions to reference.
2. Write the plan to `kb/tasks/$1/phase1-feature-extraction-plan.md`.
3. Read the file back to confirm it was written.
4. If the file does not exist or is empty, STOP and report the failure.
5. Only after confirmation, proceed to Phase 1.

## Phase 1: Setup

1. Create a todo list tracking all phases of this research workflow.
2. Load the tool inventory for the target language from @references/$1/tools.md to understand which repos are available, their types, and where features are located.
3. If the tool inventory file does not exist, inform the user that references/$1/tools.md must be created first, listing cloned repos with their type, language, and feature locations. Stop here.
4. If $2 is provided, filter the tool inventory to repos relevant to that focus area (e.g., "linting" filters to linter repos, "formatting" filters to formatter repos, "parsing" filters to parser repos, "validation" filters to validator repos). If no filter matches, use all repos.
5. Load the `feature-comparison` skill for report template and spec classification guidance.

## Phase 2: Clarifying Questions

Before launching extraction, review the tool inventory and ask the user:

1. **Scope confirmation** — "I found N repos of type X, Y, Z. Should I scan all of them, or focus on a subset?"
2. **Depth preference** — "Should I do a broad survey (feature names and categories) or a deep extraction (including configuration options, default values, and error messages)?"
3. **Priority signals** — "Are there specific features or capabilities you already know you want in Biome? This helps me prioritize the comparison."

Wait for answers before proceeding.

## Phase 3: Feature Extraction

Launch the lang-feature-extractor agent via the Task tool. Based on the number of repos and user preferences:

- **For 1-3 repos:** Launch a single lang-feature-extractor agent covering all repos.
- **For 4+ repos:** Launch multiple lang-feature-extractor agents in parallel, grouped by tool type (all linters together, all formatters together, etc.).

Provide each agent with:
- The list of repos to scan (paths from tools.md)
- The feature locations within each repo (from tools.md)
- The extraction depth (broad or deep, per user preference)
- The language name for context
- Instruction to classify each feature's spec basis: spec-mandated, spec-divergence (between spec versions), spec-ambiguity, or tool-opinion. For YAML, reference the YAML 1.1 (2005) and YAML 1.2 (2009/2021) specifications.

Update the todo list as each agent completes.

## Phase 4: Synthesis

After all extraction agents complete:

1. Read all agent outputs carefully.
2. Combine findings into a unified report structured as:

### Executive Summary
- Total features found across all tools
- Number of consensus features (in 2+ tools)
- Number of unique features (in 1 tool only)
- Key patterns observed

### Feature Matrix by Category
One matrix per tool type (linting, formatting, parsing, validation). Features as rows, tools as columns.

### Consensus Features (Biome Candidates)
Features present in multiple tools, ranked by prevalence. These are the strongest candidates for Biome implementation.

### Unique and Notable Features
Features found in only one tool that may still warrant Biome implementation. Include rationale.

### Spec Grounding
For each major feature area, classify whether tools are enforcing spec-mandated behavior, addressing spec-version differences (e.g., YAML 1.1 vs 1.2 boolean/octal/merge-key semantics), filling spec gaps, or imposing tool-specific opinions. Identify where the spec versions diverge and how each tool handles it.

### Architectural Observations
Cross-tool patterns in implementation approach, configuration design, error handling, or extensibility that should inform Biome's design.

### Recommended Next Steps
Prioritized list of what to investigate or implement next.

## Phase 5: Completion

1. Save the report to `references/$1/feature-research-report.md`.
2. Present the synthesis report to the user.
3. Ask if they want to drill deeper into any specific area, tool, or feature category.
4. Mark all todos complete.

## Gate 2: Phase Summary (BLOCKING)

1. Write a summary to `kb/tasks/$1/phase1-feature-extraction-summary.md` containing:
   - **Completed work:** N tools scanned, N features extracted, N consensus features identified
   - **Artifacts produced:** `references/$1/feature-research-report.md`
   - **Key findings:** Top 3-5 observations
   - **Resumption instructions:** How to continue to Phase 2 (architecture analysis)
2. Report to user: "Phase 1 complete. Report: references/$1/feature-research-report.md. Safe to /clear. To resume, read the summary."
