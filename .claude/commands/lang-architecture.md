---
description: Analyze a language's integration state against Biome's 7-layer extension contract
argument-hint: <language>
---

# Language Architecture Analysis

Assess how completely a language is integrated into Biome and produce a gap analysis with dependency-ordered implementation checklist.

Language: $1

## Gate 1: Plan Capture (BLOCKING)

1. Generate an architecture analysis plan for $1 covering: which crates to scan, what layers to assess, what reference implementation to compare against.
2. Write the plan to `kb/tasks/$1/phase2-architecture-analysis-plan.md`.
3. Read the file back to confirm it was written.
4. If the file does not exist or is empty, STOP and report the failure.
5. Only after confirmation, proceed to Gate 2.

## Gate 2: Prerequisite Check (BLOCKING)

Verify Phase 1 artifacts exist:
- `references/$1/feature-research-report.md` must exist (produced by `/lang-research`)

If missing, report what's needed and STOP. Do not proceed without Phase 1 output.

## Phase 1: Load References

1. Read `references/biome/extension-contract.md` — the 7-layer contract.
2. Read `references/$1/feature-research-report.md` — the feature landscape (for context on what the language needs).
3. Load the biome-integration skill for integration checklist awareness.

## Phase 2: Architecture Analysis

Launch the `lang-architecture-analyst` agent via the Task tool with:
- The target language: $1
- Instruction to assess all 7 layers
- Instruction to use JSON as the reference implementation (unless the language has specific needs better served by another reference)
- Instruction to produce the full gap analysis format: Current State Summary → Layer-by-Layer Assessment → Gap Checklist → Dependency Graph → Risk Areas

## Phase 3: Architecture Notes

After the analyst completes:

1. Review the gap analysis output.
2. Create `references/$1/architecture-notes.md` containing:
   - Current integration state (which layers exist, which are complete/partial/missing)
   - Language-specific concerns that differ from JSON (e.g., indentation semantics, embedded languages, type systems)
   - Parser capabilities if parser already exists (what nodes are supported, what's missing)
   - Recommended implementation order with rationale
   - Known complexity hotspots

## Gate 3: Phase Summary (BLOCKING)

1. Write a summary to `kb/tasks/$1/phase2-architecture-analysis-summary.md` containing:
   - **Completed work:** What was assessed, key findings
   - **Artifacts produced:** File paths of architecture-notes.md and any other outputs
   - **Key decisions:** Implementation order, reference implementation choice
   - **Resumption instructions:** How to continue to Phase 3 (spec writing)
2. Report to user: "Phase 2 complete. Artifacts: references/$1/architecture-notes.md. Safe to /clear. To resume, read the summary."
