# Quality Evaluation Framework — Internal Consistency Audit

**Date:** 2026-02-11
**Document under review:** `kb/tasks/quality-evaluation-framework-generic.md`

---

## 1. Structural inconsistency across categories

The "How to use" section promises 5 subsections per category. Actual coverage:

| Subsection | Present in | Missing from |
|---|---|---|
| What it measures | All 14 | — |
| Why it matters | 1, 2, 3, 4, 5, 13 | **6, 7, 8, 9, 10, 11, 12, 14** (8 of 14) |
| How to measure | 1 | **2-14** (13 of 14) |
| Thresholds | 1, 2, 6, 7, 10 | **3, 4, 5, 8, 9, 11, 12, 13, 14** (9 of 14) |
| Audit procedure | All 14 | — |

Only **Test Coverage (§1)** has all 5 promised subsections. This is the most significant consistency problem.

---

## 2. Invalid severity value

Line 524 — Cross-Component Validation dimensions table:

> `Shared utility usage | ... | Medium-High`

**"Medium-High" is not a valid severity** in the 4-level model (High, Medium, Low, Info) defined in the Severity Model section.

---

## 3. Remediation template omits Info

Line 613 — `**Severity:** High | Medium | Low` omits Info. Not necessarily wrong (Info findings may not need remediation), but the omission is unstated.

---

## 4. Threshold contradiction: complexity vs severity example

- Severity Model (line 47): `Function complexity >20` is the **Medium** severity example
- Complexity Thresholds (line 133): `>10` is already **High (Refactor)**

If >10 triggers refactoring, >20 should be higher severity than Medium. The two scales (complexity rating vs finding severity) use different meanings of "High" without explanation.

---

## 5. Coverage target mismatch

- Test Coverage thresholds (line 83): ≥80% line coverage = **Acceptable**
- CI/CD Pipeline thresholds (line 422): Project coverage target = **≥85%**
- Test Coverage audit step 3 (line 103): Below 80% = **high priority gap**

The CI/CD gate is stricter than the coverage assessment table. A project at 82% is "Acceptable" per §1 but fails CI per §10.

---

## 6. Category overlap (unacknowledged)

Two pairs of categories have overlapping scope:

**Security (§7) ↔ Dependency Health (§11):**
- §7 dimensions: "Dependency vulnerabilities", "Dependency policy"
- §11 dimensions: "Vulnerability status", "License compliance"

Same checks, different names.

**Compliance (§12) ↔ File Organization (§14):**
- §12 standard types: "File organization", "Presence verification" (README, CHANGELOG, config)
- §14 dimensions: "Standard files present" (README, LICENSE, CHANGELOG, CI config)

Same checks.

---

## 7. Phase-priority misalignment

| Phase | Categories in it | Their priorities |
|---|---|---|
| 3 — Consistency | Naming, DRY, Compliance, Cross-Component | Medium, Medium, Medium, Medium |
| 4 — Polish | Documentation, i18n, Performance, File Org | **Medium**, **Medium**, Low, Low |

Documentation and i18n are both Medium priority but placed in the last substantive phase. No rationale distinguishes why these Medium categories are deferred vs the Phase 3 Medium categories.

---

## 8. Residual language-specific idioms

Line 232 — Error Handling audit step 2:

> `Identify all unhandled error points (unwrap, bare raise, uncaught exceptions)`

"unwrap" is a Rust idiom, "bare raise" is Ruby. Should use generic phrasing.

---

## 9. Terminology inconsistency: i18n vs localization

The document mixes:
- "Internationalization" (category name, line 375)
- "i18n" (phased audit, line 636; design principles, line 648)
- "Localization coverage" (dimension name, line 385)
- "localizable" (error handling, line 217)
- "Localized message function" (DRY, line 274)

Internationalization and localization are distinct concepts (i18n = making it possible, l10n = doing it for a specific locale). The document uses them interchangeably.

---

## 10. Unique subsections without pattern

Several categories have bespoke subsections found nowhere else:

| Subsection | Category | Appears elsewhere? |
|---|---|---|
| Test layering model | §1 | No |
| Finding templates | §1 | No |
| Distribution targets | §2 | No |
| Refactoring strategies | §2 | No |
| Maturity levels | §4, §8 | Only these two |
| Types of duplication | §5 | No |
| Common consolidation targets | §5 | No |
| Check type taxonomy | §12 | No |

This isn't wrong — categories genuinely differ — but it contradicts the "each category includes [same 5 things]" framing.

---

## Summary

| Issue | Severity | Fix effort |
|---|---|---|
| 8 categories missing "Why it matters" | Medium | Low — add 1-2 sentences each |
| 13 categories missing "How to measure" | Medium | Medium — decide: drop from promise or add |
| 9 categories missing "Thresholds" | Medium | Medium — some have alternatives (maturity levels, dimensions) |
| "Medium-High" invalid severity | High | Low — change to Medium or High |
| Complexity >20 = Medium severity example | Medium | Low — change example to >15 or pick different example |
| Coverage 80% vs 85% CI gate | Low | Low — add "(configurable)" to one or align |
| Security ↔ Dep Health overlap | Low | Low — add cross-reference note |
| Compliance ↔ File Org overlap | Low | Low — add cross-reference note or merge |
| "unwrap, bare raise" language-specific | Medium | Low — rephrase generically |
| i18n/localization terminology | Low | Low — pick one or define both |

The document's content is sound. The main structural issue is the "How to use" section promising a uniform 5-subsection format that only 1 of 14 categories actually follows. Either relax the promise or fill the gaps.
