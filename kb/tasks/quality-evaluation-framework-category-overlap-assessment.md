# Quality Evaluation Framework — Category Overlap & MECE Assessment

**Date:** 2026-02-11
**Document under review:** `kb/tasks/quality-evaluation-framework-generic.md`
**Related:** `kb/tasks/quality-evaluation-framework-consistency-audit.md` (issue #6)

---

## Full Overlap Map

The consistency audit identified 2 overlapping pairs. There are actually 10.

### Overlap 1: Security (§7) ↔ Dependency Health (§11)

| §7 Security dimension | §11 Dependency Health dimension |
|---|---|
| Dependency vulnerabilities — Known CVEs | Vulnerability status — Any known CVEs? |
| Dependency policy — Disallowed licenses | License compliance — All licenses compatible? |

2 of 5 §7 dimensions and 2 of 5 §11 dimensions are the same check under different names. An auditor running both procedures would run the same vulnerability scanner twice.

### Overlap 2: Compliance (§12) ↔ File Organization (§14)

| §12 Compliance standard type | §14 File Organization dimension |
|---|---|
| File organization — Directory structure, file placement | Source organization — Logical grouping |
| Presence verification — README, CHANGELOG, config files | Standard files present — README, LICENSE, CHANGELOG, CI config |

§14's audit procedure is a strict subset of what §12's check type taxonomy can express (`file_exists`, `directory_exists`).

### Overlap 3: Compliance (§12) ↔ Naming (§3)

§3 measures naming convention adherence. §12's "Code patterns" standard type checks "required patterns present, anti-patterns absent." Naming conventions ARE coding standards — §3 is a specialized instance of §12 focused on one kind of standard (names).

### Overlap 4: Compliance (§12) ↔ Cross-Component (§13)

| §12 check type | §13 dimension |
|---|---|
| `uses_helper` — Component uses shared utility | Shared utility usage — Not reimplemented |
| `content_match` — Required pattern found | Pattern consistency — Same patterns used |
| `content_not_match` — Anti-pattern absent | Anti-pattern detection — No known anti-patterns |

§13's dimensions are implementable via §12's check type taxonomy. The only thing §13 adds is the *comparative* framing: "check the same thing across N components and compare."

### Overlap 5: Cross-Component (§13) ↔ Naming (§3)

§13 dimension: "Naming consistency — Components follow same naming convention."
§3: the entire category.

Naming consistency across components is the intersection of these two categories. An auditor doing §3 thoroughly would catch what §13's naming dimension catches.

### Overlap 6: Cross-Component (§13) ↔ DRY (§5)

§13 dimension: "Shared utility usage — Components use shared helpers (not reimplemented)."
§5: consolidation of duplicated code.

Code reimplemented instead of using a shared utility is simultaneously a DRY violation and a cross-component inconsistency.

### Overlap 7: Error Handling (§4) ↔ i18n (§9)

§4 dimension: "User-facing messages — Are error messages actionable and localizable?"
§9 audit step 1: "Inventory all user-facing output points (messages, errors, logs)."

Localizable error messages are claimed by both categories. An auditor doing §4 checks whether messages are localizable; an auditor doing §9 inventories all user-facing strings including error messages.

### Overlap 8: Documentation (§6) ↔ Compliance (§12) ↔ File Organization (§14)

Triple overlap on file existence:

| §6 | §12 | §14 |
|---|---|---|
| README exists with required sections | Presence verification: README, CHANGELOG | Standard files: README, LICENSE, CHANGELOG |

Three categories independently check whether README exists.

### Overlap 9: CI/CD (§10) ↔ everything

§10's "Required CI jobs" table explicitly references other categories:

| §10 CI job | Category it enforces |
|---|---|
| Test | §1 Test Coverage |
| Coverage | §1 Test Coverage |
| Lint | §3 Naming + static analysis |
| Format | (formatting, not a named category) |
| Security | §7 Security / §11 Dependency Health |
| Docs | §6 Documentation |

§10's "Pipeline thresholds" re-state targets from §1 (coverage ≥85%) and §7 (0 critical/high vulnerabilities). CI/CD doesn't measure quality — it automates and gates the measurement defined by other categories.

### Overlap 10: Security (§7) code patterns ↔ Error Handling (§4)

§4 "Why it matters": "security leaks (internal details exposed)."
§7 "Input validation": injection vectors, permission model.

Error handling that exposes internal details is both an error handling and a security concern.

---

## The Structural Problem

The 14 categories aren't 14 peer-level quality dimensions. They mix three fundamentally different things:

### A. Quality dimensions (what you measure)

These measure genuinely distinct properties of the code:

| Category | What it measures | Measurement type |
|---|---|---|
| §1 Test Coverage | Code exercised by tests | Percentage |
| §2 Code Complexity | Structural complexity of functions | Per-function score |
| §4 Error Handling | Error management patterns | Pattern classification |
| §5 Code Duplication | Repeated code patterns | Clone clusters |
| §8 Performance | Runtime characteristics | Latency, throughput, resource usage |

These five are mostly orthogonal — knowing one tells you little about the others.

### B. Standards conformance (what you validate against)

These all answer "does X match standard Y?" and share the same audit shape (define standard → check → report violations):

| Category | What standard it checks against |
|---|---|
| §3 Naming Conventions | Naming rules (case, prefixes, hierarchy) |
| §6 Documentation | Documentation completeness schema |
| §7 Security (code-level) | Secure coding patterns (OWASP, input validation) |
| §9 i18n | Localization readiness schema |
| §11 Dependency Health | Freshness/vulnerability/license thresholds |
| §14 File Organization | Project structure conventions |

These are all instances of "check against a schema." They differ in *what* standard they check against, but the audit mechanism is the same: define expected state → compare to actual state → report gaps.

### C. Meta-categories (how you apply A and B)

These aren't quality dimensions — they're evaluation modes or enforcement layers:

| Category | What it actually is |
|---|---|
| §10 CI/CD Pipeline | Enforcement mechanism — automates and gates checks from A and B |
| §12 Compliance & Standards | Meta-framework — can express any check from B (and some from A) |
| §13 Cross-Component Validation | Evaluation mode — "apply checks from A and B across N components and compare" |

§12 is literally the generalization of all group B categories. §13 is a way of running any check comparatively. §10 is where checks get automated.

---

## Assessment: Demonstrability, Mutual Exclusivity, Actionability

For tech projects specifically, evaluated on the three criteria:

### Demonstrability

Can a tool produce a concrete, unambiguous output?

| Category | Tool | Output type | Demonstrability |
|---|---|---|---|
| §1 Test Coverage | Coverage analyzer | % per module, uncovered lines | **Very High** — exact numbers, no judgment needed |
| §2 Code Complexity | Complexity analyzer | Score per function | **Very High** — exact numbers |
| §5 Code Duplication | Clone detector | Clone clusters with locations and line counts | **High** — exact counts |
| §11 Dependency Health | Vuln scanner, outdated checker, license checker | CVE list, version diffs, license list | **High** — concrete lists |
| §7 Security (deps) | Same as §11 | Same as §11 | **High** — but duplicate of §11 |
| §3 Naming | Linter | Violation list | **High** for case rules; **Low** for semantic quality |
| §6 Documentation | Doc coverage tool | % documented items | **High** for coverage; **Low** for quality |
| §14 File Organization | File existence check | Present/absent list | **High** — binary checks |
| §10 CI/CD | Pipeline config inspection | Job list, gate status | **High** — structural check |
| §4 Error Handling | Pattern search + manual | Pattern tallies | **Medium** — requires classifying patterns |
| §7 Security (code) | SAST scanner | Finding list | **Medium** — high false positive rates |
| §9 i18n | String extraction + classification | String inventory | **Medium** — requires classifying strings as user-facing |
| §8 Performance | Benchmarks | Numbers (if benchmarks exist) | **Medium** — bootstrapping problem: needs benchmarks first |
| §13 Cross-Component | Comparative analysis | Divergence list | **Low-Medium** — requires defining "should be consistent" first |
| §12 Compliance | Declarative checks | Pass/fail per check | **High** when checks are defined; **Low** before definition |

### Mutual Exclusivity

Does this category measure something no other category measures?

| Category | Unique signal? | What overlaps with? |
|---|---|---|
| §1 Test Coverage | **Yes** — no other category measures test exercise | — |
| §2 Code Complexity | **Yes** — no other category measures structural complexity | — |
| §5 Code Duplication | **Mostly** — clone detection is unique; "shared utility usage" overlaps §13 | §13 |
| §4 Error Handling | **Mostly** — error pattern analysis is unique; localizable messages overlaps §9 | §9, §7 |
| §8 Performance | **Yes** — no other category measures runtime characteristics | — |
| §3 Naming | **No** — is a special case of §12, naming consistency overlaps §13 | §12, §13 |
| §6 Documentation | **Mostly** — doc quality/coverage is unique; file existence overlaps §12, §14 | §12, §14 |
| §7 Security | **Partially** — code-level security is unique; dep security duplicates §11 | §11, §4 |
| §9 i18n | **Mostly** — localization readiness is unique; error message overlap with §4 | §4 |
| §10 CI/CD | **No** — enforces other categories' checks, doesn't measure its own dimension | §1, §3, §6, §7/§11 |
| §11 Dep Health | **Partially** — freshness/unused are unique; vuln/license duplicates §7 | §7 |
| §12 Compliance | **No** — meta-category that subsumes §3, §14, parts of §6, §13 | §3, §6, §13, §14 |
| §13 Cross-Component | **No** — evaluation mode (comparative), not a quality dimension | §3, §5, §12 |
| §14 File Organization | **No** — strict subset of §12 | §12, §6 |

### Actionability

Does the audit procedure produce findings that directly tell you what to fix?

| Category | Audit output → action clarity | Schema-validatable? |
|---|---|---|
| §1 Test Coverage | Module X has Y% coverage → write tests for module X | Yes (threshold) |
| §2 Code Complexity | Function X has score Y → refactor function X | Yes (threshold) |
| §5 Code Duplication | Lines A-B in file X duplicate lines C-D in file Y → extract | Yes (threshold) |
| §11 Dep Health | Dep X has CVE-Y → update dep X | Yes (CVE database) |
| §3 Naming | Name X violates rule Y → rename X | Yes (linter rules) |
| §6 Documentation | Item X undocumented → document X | Yes (coverage threshold) |
| §14 File Organization | File X missing → create X | Yes (file checklist) |
| §10 CI/CD | Job X missing → add job X | Yes (job checklist) |
| §4 Error Handling | Function X uses crash pattern → convert to structured | Partially (pattern heuristics) |
| §7 Security (code) | Pattern X at location Y → fix pattern | Partially (SAST rules, false positive triage) |
| §9 i18n | String X not routed through i18n → wrap X | Partially (string classification) |
| §8 Performance | Metric X regressed by Y% → investigate X | Only if benchmarks exist |
| §13 Cross-Component | Components A and B differ on pattern X → align | Requires defining "correct" first |
| §12 Compliance | Check X failed → remediate per check definition | Yes, but you must define checks first |

---

## Proposed Reorganization

Based on the three criteria, the 14 categories separate into a tighter set of **9 primary categories** plus **2 meta-concerns** that apply across all of them.

### Primary categories (mutually exclusive, each with unique signal)

| # | Category | Source | Rationale |
|---|---|---|---|
| 1 | **Test Coverage** | §1 unchanged | Orthogonal, very high demonstrability, clear action |
| 2 | **Code Complexity** | §2 unchanged | Orthogonal, very high demonstrability, clear action |
| 3 | **Code Duplication** | §5 unchanged | Mostly orthogonal, high demonstrability |
| 4 | **Error Handling** | §4, minus localizable-messages (→ §8) | Unique signal on error patterns, medium demonstrability |
| 5 | **Performance** | §8 unchanged | Orthogonal, demonstrable when benchmarks exist |
| 6 | **Dependencies** | §7 dep dimensions + §11 merged | Eliminates the biggest overlap; one audit, one report |
| 7 | **Security** | §7 code-level dimensions only | Injection, secrets, permissions, input validation — distinct from dep scanning |
| 8 | **Documentation** | §6, absorbs doc-related checks from §12/§14 | Unique signal on doc quality/completeness |
| 9 | **Localization** | §9, absorbs §4's localizable-messages dimension | Unique signal on i18n readiness; cleaner boundary with Error Handling |

### Standards conformance (merged from §3, §12, §14)

| # | Category | Source | Rationale |
|---|---|---|---|
| 10 | **Project Standards** | §3 + §12 + §14 merged | All three answer "does X match convention Y?" — naming, file structure, coding patterns, metadata are all standards. §12's check type taxonomy is the audit mechanism; §3 and §14 are specific standard definitions that plug into it. |

This is where:
- §3 Naming becomes a *standard definition* (naming rules) evaluated by the compliance mechanism
- §14 File Organization becomes a *standard definition* (structure rules) evaluated by the compliance mechanism
- §12 becomes the *audit framework* for evaluating all standards, with its check type taxonomy as the engine

### Cross-cutting concerns (not peer categories — evaluation modes)

| # | Concern | Source | Role |
|---|---|---|---|
| A | **Cross-Component Consistency** | §13 | Evaluation mode: "run any primary category's checks across N components and compare." Not a quality dimension — it's a way of applying categories 1-10 comparatively. |
| B | **CI/CD Enforcement** | §10 | Enforcement layer: "which of categories 1-10 are automated and gated?" Not a quality dimension — it's the maturity level of automation for all other categories. |

Note: CI/CD maps directly to the maturity levels pattern identified in the subsection assessment. A category at maturity level 2+ means it has CI enforcement. §10 is really "what maturity level is each category's automation at?"

---

## What changes

### Eliminated overlaps

| Current overlap | Resolution |
|---|---|
| §7 ↔ §11 (dep vulns + licenses) | Merged into "Dependencies" |
| §12 ↔ §14 (file existence) | Merged into "Project Standards" |
| §12 ↔ §3 (naming = standards) | Merged into "Project Standards" |
| §12 ↔ §13 (pattern checks) | §13 reclassified as cross-cutting evaluation mode |
| §13 ↔ §3 (naming consistency) | §3 absorbed into "Project Standards"; §13 becomes evaluation mode |
| §13 ↔ §5 (shared utility) | DRY stays primary; §13 becomes evaluation mode |
| §4 ↔ §9 (localizable messages) | Localizable messages goes to "Localization" |
| §6 ↔ §12 ↔ §14 (README exists) | All file existence under "Project Standards"; doc quality stays in "Documentation" |
| §10 ↔ everything (enforcement) | §10 reclassified as cross-cutting enforcement concern, not a peer category |

### What the audit gains

1. **No duplicate work.** An auditor won't run a vulnerability scanner in §7 and again in §11.
2. **Clear ownership.** "Where does checking README existence go?" has one answer (Project Standards), not three.
3. **Cleaner action items.** Findings map to exactly one category, so remediation ownership is unambiguous.
4. **CI/CD becomes a maturity assessment.** Instead of auditing CI/CD as its own category, each primary category gets a maturity level (none → manual → automated → gated). This integrates naturally with the maturity levels pattern from the subsection assessment.

### What the audit loses

1. **Category count drops from 14 to 10+2.** Some stakeholders may prefer the longer list for thoroughness signaling.
2. **"Project Standards" is broad.** It covers naming, file structure, coding patterns, and metadata. Could feel like a grab bag. Mitigated by using §12's check type taxonomy as the organizing structure — each standard is a defined check type, not a vague aspiration.
3. **CI/CD loses visibility as a top-level concern.** Mitigated by the maturity level assessment being built into every category — CI/CD actually gets MORE coverage, distributed across all categories, rather than being a single checkbox.

---

## Decision Matrix

For each original category, where it goes:

| Original | → Destination | Why |
|---|---|---|
| §1 Test Coverage | **Test Coverage** (primary) | Unique, orthogonal |
| §2 Code Complexity | **Code Complexity** (primary) | Unique, orthogonal |
| §3 Naming Conventions | **Project Standards** (absorbed) | Naming rules are standards; not a standalone dimension |
| §4 Error Handling | **Error Handling** (primary, narrowed) | Unique core; localizable-messages → Localization |
| §5 Code Duplication | **Code Duplication** (primary) | Unique, mostly orthogonal |
| §6 Documentation | **Documentation** (primary, tightened) | Doc quality unique; file existence → Project Standards |
| §7 Security | **Security** (code-level, primary) + **Dependencies** (dep dimensions merged) | Split resolves §7/§11 overlap |
| §8 Performance | **Performance** (primary) | Unique, orthogonal |
| §9 i18n | **Localization** (primary, expanded) | Absorbs §4's localizable-messages; unique i18n signal |
| §10 CI/CD Pipeline | **CI/CD Enforcement** (cross-cutting) | Enforcement layer, not a quality dimension |
| §11 Dependency Health | **Dependencies** (merged with §7 dep dimensions) | Eliminates biggest overlap |
| §12 Compliance & Standards | **Project Standards** (audit mechanism) | Check type taxonomy becomes engine for all standards |
| §13 Cross-Component | **Cross-Component Consistency** (cross-cutting) | Evaluation mode, not a dimension |
| §14 File Organization | **Project Standards** (absorbed) | File structure rules are standards |
