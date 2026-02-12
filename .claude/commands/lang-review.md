---
description: Post-implementation review checkpoint — capture discoveries and update methodology
argument-hint: <language>
---

# Language Review Checkpoint

Post-implementation checkpoint for $1 support in Biome. Captures what was learned, what the spec missed, what hooks would have prevented, and updates the methodology.

Language: $1

This is a checkpoint, not a full phase. It takes 15-20 minutes and produces observations, not code.

## Gate 1: Prerequisite Check (BLOCKING)

Verify Phase 4 is complete:
- `kb/tasks/$1/phase4-implementation-summary.md` must exist
- At minimum, `cargo build -p biome_$1_formatter` OR `cargo build -p biome_$1_analyze` succeeds

If Phase 4 summary doesn't exist, ask the user whether implementation is complete enough for review.

## Step 1: Read Phase History

Read these files to understand the full arc:
1. `kb/tasks/$1/phase4-implementation-summary.md` — what was built, deferred, discovered
2. `references/$1/$1-support-spec.md` — what was planned
3. `kb/tasks/$1/phase4-implementation-plan.md` — the original implementation plan

## Step 2: Checklist Review

Answer each question with specific findings:

### What broke?
- Which stages had unexpected failures?
- What error messages or symptoms appeared?
- How long did debugging take?
- What was the root cause?

### What was harder than expected?
- Which parts of the spec underestimated complexity?
- Where did the reference implementation not apply cleanly?
- What language-specific concerns weren't anticipated?

### What did the spec miss?
- Features that should have been in the spec but weren't
- Sections that should exist (e.g., "Defaults That Differ from Biome Globals" was discovered post-YAML)
- Information that would have saved time if included upfront

### What would save time for the next language?
- Hook candidates (patterns of mistakes that could be caught automatically)
- Improved gate checks
- Better reference material
- Tool requirements that should be verified earlier

### What worked well?
- Which parts of the process were efficient?
- Which artifacts were genuinely useful?
- Which agents/commands saved time?

## Step 3: Update Methodology

1. Update `kb/tasks/agent-evolution-model.md`:
   - Add findings to the relevant phase "Discovered" sections
   - Update the "Growth path" diagram if new components crystallized
   - Add any new process issues
   - Update crystallization heuristics if patterns emerged

2. If any skill was used a second time (crystallization trigger), note it as validated.

3. If any mistake occurred once (hook crystallization trigger), define the hook.

## Gate 2: Phase Summary (BLOCKING)

1. Write a summary to `kb/tasks/$1/phase5-review-summary.md` containing:
   - **Key findings:** 3-7 most important observations
   - **Spec gaps:** What the spec should have included
   - **Hook candidates:** Patterns that should be automated
   - **Methodology updates:** What changed in agent-evolution-model.md
   - **Recommendation for next language:** What to do differently

2. Report to user: "Review complete. Summary: kb/tasks/$1/phase5-review-summary.md. Key findings: [list top 3]."
