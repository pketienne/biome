# Quality Evaluation Framework — Bespoke Subsection Assessment

**Date:** 2026-02-11
**Document under review:** `kb/tasks/quality-evaluation-framework-generic.md`
**Related:** `kb/tasks/quality-evaluation-framework-consistency-audit.md` (issue #10)

---

## Context

The consistency audit identified 8 bespoke subsections that appear in only one or two categories, contradicting the "each category includes the same 5 subsections" framing. This assessment evaluates each bespoke subsection against all 14 categories to determine whether the information is:

- **Generalizable** — can be promoted to a principle or named optional subsection
- **Low/questionable value** — obvious, redundant, or subsumable into existing structure
- **Genuinely unique** — domain-specific, high value, should stay as-is

---

## 1. Test layering model (§1 Test Coverage)

**Could it generalize?** No. It's a layered progression of test types specific to coverage. No other category has an analogous "ordered layers that build on each other" structure.

**Could other categories use it?** Documentation (§6) has something vaguely analogous (inline → module → API → architecture), but it's already captured in its thresholds table. Performance (§8) has benchmark categories, but those are dimensions, not layers.

**Value:** **High.** The "When to create" and "What it catches" columns are uniquely actionable — they tell the auditor not just *what to measure* but *what to prescribe*. This is the kind of information that separates a useful audit from a metrics dump.

**Verdict:** Genuinely unique, high value. Keep.

---

## 2. Finding templates (§1 Test Coverage)

**Could it generalize?** Yes — to every single category. Every category produces findings at different severity levels.

**Where would it add value?**

| Category | Example template |
|----------|-----------------|
| §2 Complexity | `Function {name} has cyclomatic complexity {X} (threshold: >10).` |
| §4 Error Handling | `Module {name} has {X} unhandled crash points.` |
| §5 DRY | `{N} code blocks duplicated across {files}. {lines} lines consolidatable.` |
| §7 Security | `Found {N} vulnerabilities ({critical} critical, {high} high).` |
| §13 Cross-Component | `{component_a} and {component_b} diverge on pattern {pattern}.` |

**Value:** **Medium.** The templates themselves are somewhat obvious — an auditor would write similar things naturally. But the real value is *forcing severity-level differentiation per category*, which makes the auditor think through what a High vs Medium vs Low finding actually looks like in each domain.

**Verdict:** Generalizable as a *principle* but adding 13 more template sets would bloat the document. Better promoted to a note in "How to use this document" (e.g., "categories MAY include finding templates by severity") with §1 as the worked example. Not worth replicating inline.

---

## 3. Distribution targets (§2 Code Complexity)

**Could it generalize?** Yes — but it's actually a variant of "Thresholds" (aggregate thresholds vs per-item thresholds).

**Where would it add value?**

| Category | Distribution form |
|----------|-------------------|
| §1 Coverage | "≥90% of modules have ≥80% coverage" |
| §3 Naming | "≥95% of public names follow convention" |
| §4 Error Handling | "≥90% of error paths use structured handling" |
| §6 Documentation | Already does this: "≥70% of public API items documented" |

§6 already has aggregate thresholds but doesn't call them "distribution targets." The concept is already present, just unnamed.

**Value:** **Medium.** It answers "how much of the codebase is in good shape?" which is useful, but it's not a separate concept — it's aggregate-level thresholds. The per-item vs aggregate distinction could be noted once.

**Verdict:** Subsumable into "Thresholds." Rather than a standalone subsection, Thresholds tables could note whether they apply per-item or in aggregate. §6 already does this without ceremony.

---

## 4. Refactoring strategies (§2 Code Complexity)

**Could it generalize?** Yes — and it already has. See item 7 below.

**What it actually is:** A "pattern → technique → expected impact" remediation playbook.

**Where would it add value?**

| Category | Remediation playbook content |
|----------|------------------------------|
| §4 Error Handling | string errors → typed errors, crash → structured propagation |
| §7 Security | injection → parameterized queries, hardcoded secret → env var |
| §9 i18n | hardcoded string → i18n wrapper |

**Value:** **High for §2 specifically** — the "expected reduction" column (-40-60%, -50-80%) is uniquely concrete and gives the auditor ammunition for prioritization. No other subsection in the document quantifies expected improvement.

**Verdict:** Generalizable as a pattern ("Remediation playbook"). The *structure* generalizes but the *content* is domain-specific, so you can't write a single table. See item 7 for the duplication issue.

---

## 5. Maturity levels (§4 Error Handling, §8 Performance)

**Could it generalize?** Yes — most strongly of all 8 subsections. It already appears in two categories.

**Where would it add value?**

| Category | Level 0 | Level 1 | Level 2 | Level 3 |
|----------|---------|---------|---------|---------|
| §1 Coverage | No tests | Manual tests exist | CI runs tests | Coverage gated |
| §7 Security | No scanning | Manual review | Automated scanning | CI-gated, blocks PRs |
| §9 i18n | All hardcoded | Some strings i18n'd | Systematic i18n | Full l10n pipeline |
| §11 Dep Health | No tracking | Manual check | Automated alerts | PR blocking |
| §6 Documentation | None | README exists | API docs generated | Auto-checked in CI |
| §10 CI/CD | No CI | CI exists, manual trigger | Automated on PR | Full pipeline + gates |
| §13 Cross-Component | No checks | Manual review | Automated lint rules | CI-gated |

This applies to **at least 9 of 14 categories** (§1, §3, §6, §7, §9, §10, §11, §13, §14). The progression is always the same skeleton: none → manual → automated → gated.

The 0-3 levels also map cleanly to the severity model: Level 0 = High finding, Level 3 = Info. This connection is already implicit in §4's maturity table (which puts severity indicators in the third column) but isn't stated as a principle.

**Value:** **High.** This is the most universally applicable pattern. It answers "where are we on the journey?" which is richer than pass/fail and gives a natural improvement roadmap.

**Verdict:** Strongest generalization candidate. Should be promoted to a top-level pattern in "How to use this document." The skeleton (none → manual → automated → gated) could be stated once, with §4 and §8 as worked examples, and a note that categories MAY include a domain-specific maturity model.

---

## 6. Types of duplication (§5 Code Duplication)

**Could it generalize?** It already has — under different names.

Look at what §4, §7, §9, and §13 call "Dimensions":

| Subsection heading | Category | Content |
|---|---|---|
| Dimensions | §4 Error Handling | Subtypes (typing, propagation, context...) with severity |
| Dimensions | §7 Security | Subtypes (vulnerabilities, secrets, validation...) |
| Dimensions | §9 i18n | Subtypes (coverage, detection, completeness...) with severity |
| Dimensions | §13 Cross-Component | Subtypes (naming, utility, dependency...) with severity |
| **Types of duplication** | §5 DRY | Subtypes (exact, near, structural, expected) with severity |

§5's "Types of duplication" is structurally identical to the "Dimensions" tables in §4, §7, §9, §13 — it lists subtypes with their relative severity. It's just named differently.

**Value:** **Low as a unique pattern.** It's a naming inconsistency, not a novel idea.

**Verdict:** Rename to "Dimensions" for consistency, or note in the consistency audit that this is the same concept. Not a real bespoke subsection.

---

## 7. Common consolidation targets (§5 Code Duplication)

**Could it generalize?** It already has — it's the same pattern as §2's Refactoring strategies.

| §2 Refactoring strategies | §5 Common consolidation targets |
|---|---|
| Pattern → Technique → Expected reduction | Pattern → Consolidation strategy |

Both are "here's the common problem, here's how to fix it." §2's version is strictly better because it includes the "expected reduction" quantification.

**Value:** **Medium.** Useful content, but structurally redundant with §2. Also, the consolidation targets are fairly obvious (boilerplate → helper, validation → extract validator). An experienced developer wouldn't need this table.

**Verdict:** This is the same generalizable pattern as §2 (remediation playbook). §5's version is the weaker instance — it lacks the quantified impact column that makes §2's version actionable. If the pattern is promoted to a named optional subsection ("Remediation playbook"), §5's instance should adopt §2's column structure.

---

## 8. Check type taxonomy (§12 Compliance & Standards)

**Could it generalize?** Partially — but only to categories that are inherently about pattern matching.

**Where would it apply?**
- §14 File Organization: `file_exists`, `directory_exists` directly apply
- §13 Cross-Component: `content_match`, `uses_helper` directly apply
- §6 Documentation: `file_exists` (README), `content_match` (required sections)

But §1, §2, §4, §7, §8, §9 all have their own domain-specific tools (coverage tools, complexity analyzers, vulnerability scanners). Declarative check types aren't the right abstraction for those domains.

**Value:** **High for §12's domain.** The `file_exists`/`content_match`/`uses_helper` taxonomy is a powerful abstraction for compliance automation specifically because compliance checks are inherently declarative. This doesn't transfer to categories with richer analysis tools.

**Verdict:** Genuinely unique to the compliance domain. Keep in §12. Cross-reference from §13 and §14 where the check types would serve as the automation mechanism for those audits too.

---

## Summary

| Subsection | Pattern type | Verdict |
|---|---|---|
| Test layering model (§1) | Genuinely unique | **Keep** — no analog elsewhere, high value |
| Finding templates (§1) | Generalizable to all 14 | **Promote to principle** — too much bloat to replicate; state once, use §1 as example |
| Distribution targets (§2) | Subsumable into Thresholds | **Fold in** — it's aggregate thresholds; note per-item vs aggregate distinction |
| Refactoring strategies (§2) | Generalizable "remediation playbook" | **Name the pattern** — two instances exist (§2, §5); promote as optional subsection |
| Maturity levels (§4, §8) | Most generalizable; applies to ~9/14 | **Promote to top-level pattern** — none→manual→automated→gated is universal |
| Types of duplication (§5) | Naming inconsistency | **Rename to "Dimensions"** — same concept as §4, §7, §9, §13 |
| Common consolidation targets (§5) | Duplicate of §2 pattern | **Merge** — same as Refactoring strategies but weaker (lacks impact column) |
| Check type taxonomy (§12) | Domain-specific unique | **Keep** — powerful for compliance, doesn't transfer to tool-based categories |

---

## Recommended Actions

Three actions emerge from this assessment:

### 1. Promote maturity levels to a top-level optional pattern

It's the strongest generalization: applies to 9+ categories, maps to the severity model, and provides a roadmap not just a score. The skeleton (none → manual → automated → gated) should be stated once in "How to use this document," with §4 and §8 as worked examples.

### 2. Name the remediation playbook pattern

§2 and §5 independently invented it. Acknowledge it as a named optional subsection, adopt §2's column structure (pattern → technique → expected impact) as the canonical form.

### 3. Rename §5 "Types of duplication" to "Dimensions"

It's the same concept that 4 other categories already use, just inconsistently named.

The remaining three (test layering model, finding templates, check type taxonomy) are correctly placed: test layering and check type taxonomy are genuinely domain-specific, and finding templates is better as a stated principle than a replicated subsection.
