---
description: Write an implementation specification from Phase 1+2 research outputs
argument-hint: <language>
---

# Language Spec Writing

Synthesize feature research and architecture analysis into an actionable implementation specification for $1 support in Biome.

Language: $1

## Gate 1: Plan Capture (BLOCKING)

1. Generate a spec writing plan for $1 covering: which layers to specify, how many tiers, what format options and rules to include based on research.
2. Write the plan to `kb/tasks/$1/phase3-spec-writing-plan.md`.
3. Read the file back to confirm it was written.
4. If the file does not exist or is empty, STOP and report the failure.
5. Only after confirmation, proceed to Gate 2.

## Gate 2: Prerequisite Check (BLOCKING)

Verify Phase 1+2 artifacts exist:
- `references/$1/feature-research-report.md` must exist (Phase 1 output)
- `references/biome/extension-contract.md` must exist (Phase 2 output or pre-existing)
- `references/$1/architecture-notes.md` must exist (Phase 2 output)

If any is missing, report what's needed and STOP. Do not proceed without all three inputs.

## Phase 1: Load Inputs

Read all three prerequisite files in full:
1. `references/$1/feature-research-report.md` — what features to build
2. `references/biome/extension-contract.md` — how Biome integrates languages
3. `references/$1/architecture-notes.md` — current state, gaps, language-specific concerns

Load the spec-methodology skill for the template and tier classification methodology.

## Phase 2: Spec Writing

Launch the `lang-spec-writer` agent via the Task tool with:
- The target language: $1
- All three input file paths
- Instruction to follow the spec template from the spec-methodology skill
- Instruction to include all 8 metadata fields for every lint rule
- Instruction to include a "Defaults That Differ from Biome Globals" section
- Instruction to use Opus model (spec writing requires synthesis quality)

## Phase 3: Review and Finalize

After the spec writer completes:

1. Review the spec against the quality rubric:
   - [ ] Every format option has: name, type, default, research report reference
   - [ ] Every lint rule has: name, category, severity, what it checks, config, edge cases, reference, target file
   - [ ] Every concern from architecture notes is addressed
   - [ ] All file paths match extension contract patterns
   - [ ] Each phase is independently shippable
   - [ ] Implementation order respects layer dependencies
   - [ ] "Defaults That Differ from Biome Globals" section is populated
   - [ ] Testing strategy covers all layers

2. Save the spec to `references/$1/$1-support-spec.md`.

3. Present the rubric results to the user. If any items fail, note what's missing.

## Gate 3: Phase Summary (BLOCKING)

1. Write a summary to `kb/tasks/$1/phase3-spec-writing-summary.md` containing:
   - **Completed work:** Spec scope (N layers, N rules, N format options)
   - **Rubric results:** Which items passed/failed
   - **Artifacts produced:** `references/$1/$1-support-spec.md`
   - **Deferred decisions:** Open questions from the spec
   - **Resumption instructions:** How to continue to Phase 4 (implementation)
2. Report to user: "Phase 3 complete. Spec: references/$1/$1-support-spec.md. Safe to /clear. To resume, read the summary."
