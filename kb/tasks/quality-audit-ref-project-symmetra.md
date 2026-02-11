# Quality Audit Framework — Project Reference: symmetra

Project-specific information extracted from framework development. References point to the final framework at `quality-audit-framework.md`.

---

## Project profile

- **Name:** symmetra
- **Language:** Ruby
- **Domain:** Gem, Chef cookbook compliance
- **Audit scope:** Pipeline-based audit system with 8 checklist categories, 25+ check types, cross-component validation, severity classification, and RDF export
- **Source material:** 13 audit system source files + 2 quality framework documents

---

## Category contributions

symmetra was the primary source for compliance, cross-component, and file organization patterns. After reorganization, its contributions map to:

| Framework category | What symmetra contributed | Framework section |
|---|---|---|
| [Error Handling](quality-audit-framework.md#4-error-handling) | Error handling patterns in a pipeline/compliance context | §4 |
| [Dependencies](quality-audit-framework.md#6-dependencies) | Dependency health dimensions (vulnerability, license, freshness) | §6 |
| [Security](quality-audit-framework.md#7-security) | Security scanning in CI context | §7 |
| [Documentation](quality-audit-framework.md#8-documentation) | Documentation completeness requirements | §8 |
| [Project Standards](quality-audit-framework.md#10-project-standards) | 8-category standards model, check type taxonomy, declarative check mechanism | §10 |
| [Cross-Component Consistency](quality-audit-framework.md#a-cross-component-consistency) | CROSS-001 through CROSS-017 check patterns, comparative evaluation methodology | §A |

## Methodology contributions

| Contribution | Where it appears in the framework |
|---|---|
| 4-level severity model (High/Medium/Low/Info) | [Severity Model](quality-audit-framework.md#severity-model) — adapted from symmetra's severity classification |
| Result status model (Passed/Warning/Failed) | [Severity Model](quality-audit-framework.md#severity-model) — result status table |
| Check type taxonomy (`file_exists`, `content_match`, `uses_helper`, etc.) | [Project Standards](quality-audit-framework.md#10-project-standards) — check type taxonomy subsection |
| Declarative checklist approach (YAML-driven checks) | [Design Principles](quality-audit-framework.md#design-principles) — principle #5 (declarative checklists) |
| Cross-component comparative validation | [Cross-Component Consistency](quality-audit-framework.md#a-cross-component-consistency) — entire section |
| Pipeline-based aggregation (per-component results → cross-component comparison) | [Cross-Component Consistency](quality-audit-framework.md#a-cross-component-consistency) — audit procedure |

## Architecture contributions

symmetra's design influenced the framework's structural decisions:

- **Standards as data, not code:** symmetra's YAML-driven check definitions demonstrated that expressing compliance checks declaratively (check type + expected value + severity + skip conditions) is more maintainable and auditable than procedural validation code. This became design principle #5 and the check type taxonomy in §10.

- **Cross-component as evaluation mode:** symmetra's CROSS-001 through CROSS-017 checks operated by running the same check across all components and comparing results. This pattern — "apply any check comparatively" — is what made cross-component validation an evaluation mode (§A) rather than a standalone quality dimension in the reorganized framework.

- **Severity classification with result aggregation:** symmetra's severity → result status model (findings aggregated to Passed/Warning/Failed per component) provided the basis for the framework's severity model.

## Audit-specific findings

- The 8-category standards model from symmetra was the empirical basis for merging Naming Conventions, Compliance & Standards, and File Organization into a single Project Standards category — symmetra demonstrated that all three are instances of "check against a schema" with the same audit mechanism
- Cross-component check patterns (CROSS-001 through CROSS-017) showed that consistency checking is orthogonal to what's being checked — the same mechanism works for naming, dependencies, patterns, and anti-patterns
