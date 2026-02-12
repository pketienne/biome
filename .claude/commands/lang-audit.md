---
description: Audit the methodology used for a language integration — evaluate agents, commands, gates, and docs
argument-hint: <language>
---

# Methodology Audit

Evaluate the methodology used to integrate $1 into Biome. Rate each component on three dimensions: was it used, did it prevent errors or save time, and would its absence have been noticed.

Language: $1

## Gate 1: Prerequisite Check (BLOCKING)

Verify Phase 5 (review) is complete:
- `kb/tasks/$1/phase5-review-summary.md` must exist

If missing, suggest running `/lang-review $1` first.

## Step 1: Inventory All Components

Read the current state of the toolkit:

**Agents** — Read all files in `.claude/agents/`:
- Which were invoked during $1 integration?
- Which were useful when invoked?
- Which were never invoked?

**Commands** — Read all files in `.claude/commands/`:
- Which were used?
- Did the gate structure enforce plan persistence and prerequisite checks?
- Were gates skipped or bypassed?

**Skills** — Read all files in `.claude/skills/`:
- Which were loaded during the workflow?
- Did they provide information the agent didn't already have?
- Were reference files accessed?

**Reference materials** — Check `references/biome/` and `references/$1/`:
- Which documents were read during implementation?
- Which were genuinely useful vs. ceremony?
- Are they reusable as-is for the next language, or do they need parameterization?

**Plan/summary files** — Check `kb/tasks/$1/`:
- Were plan files created at each phase boundary?
- Were summary files created?
- Did resumption instructions work after `/clear`?

## Step 2: Score Each Component

For each component (agent, command, skill, reference doc, gate), assign three scores:

| Dimension | 0 | 1 | 2 |
|-----------|---|---|---|
| **Used** | Never invoked | Invoked but could have been skipped | Actively used, integral to workflow |
| **Prevented errors** | No errors it could have caught | Caught minor issues | Caught a bug that would have cost 1+ hours |
| **Missed if absent** | Would not be noticed | Mild inconvenience | Would have caused real problems |

**Score range:** 0-6 per component. Components scoring 0-1 are removal candidates. Components scoring 5-6 are core toolkit.

## Step 3: Produce Methodology Scorecard

Present results as a table:

```
| Component | Type | Used | Prevented | Missed | Total | Recommendation |
|-----------|------|------|-----------|--------|-------|---------------|
| lang-feature-extractor | agent | 2 | 1 | 2 | 5 | Keep |
| registration-systems.md | ref | 0 | 0 | 2 | 2 | Convert to hook |
```

**Recommendations per score range:**
- **0-1:** Remove or demote to documentation
- **2-3:** Keep but simplify, or defer to second language for re-evaluation
- **4-6:** Keep as-is, consider hardening (better gates, more automation)

## Step 4: Evaluate Cross-Cutting Concerns

### Orchestration
- Was the phase sequencing correct?
- Were parallelization opportunities exploited?
- Would a single `/lang-dev` orchestrating command be better than per-phase commands?

### Context management
- Was `/clear` used at phase boundaries?
- Did resumption instructions work?
- Did any information loss occur from compaction?

### Testing
- Was the testing gap acceptable?
- What testing should be mandatory before the phase is considered complete?
- Did the test timeline recommendations work?

### Overhead assessment
- How much time was spent on methodology (writing plans, running gates, producing summaries) vs. actual implementation?
- Was the overhead justified by reduced errors and faster work?
- What's the minimum viable methodology for the next language?

## Gate 2: Audit Summary (BLOCKING)

1. Write the scorecard and analysis to `kb/tasks/$1/phase6-methodology-audit.md` containing:
   - **Scorecard table:** All components scored
   - **Removal candidates:** Components scoring 0-1
   - **Core toolkit:** Components scoring 5-6
   - **Orchestration assessment:** Phase ordering, parallelization
   - **Minimum viable methodology:** The smallest set of agents + commands + references + gates for equivalent quality
   - **Recommendations for second language:** Concrete changes to make

2. Update `kb/tasks/agent-evolution-model.md` Phase 6 "Discovered" section.

3. Report to user: "Audit complete. Scorecard: kb/tasks/$1/phase6-methodology-audit.md. Core toolkit: [list components scoring 5-6]. Removal candidates: [list components scoring 0-1]."
