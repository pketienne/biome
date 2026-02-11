# Quality Audit Framework

A universal framework for software quality auditing. Applicable to any codebase regardless of language, domain, or scale.

## How to use this document

This framework defines **what to evaluate** and **how to evaluate it**.

### Category structure

Every category includes at minimum:

- **What it measures** — the quality dimension
- **Why it matters** — the risk of not measuring it
- **Audit procedure** — step-by-step evaluation process

Categories include additional subsections where applicable:

- **Dimensions** — sub-aspects of the category, each with relative severity
- **Thresholds** — quantitative targets and assessment grades
- **Maturity levels** — progression from ad hoc to automated (see [Universal Maturity Model](#universal-maturity-model))
- **Remediation playbook** — common problem patterns with fix strategies and expected impact (see [Remediation Playbook Pattern](#remediation-playbook-pattern))

Domain-specific subsections appear where they add unique value and are noted as such.

### Category architecture

The framework organizes quality concerns into three tiers:

| Tier | What it contains | Role |
|------|-----------------|------|
| **Primary categories** (1-9) | Quality dimensions with unique, measurable signals | What you measure |
| **Standards conformance** (10) | Convention adherence checked against defined schemas | What you validate against |
| **Cross-cutting concerns** (A-B) | Evaluation modes that apply across all categories | How you apply measurements |

Primary categories are mutually exclusive — each measures something no other category measures. Standards conformance uses a single audit mechanism (declarative checks) for all convention types. Cross-cutting concerns are not peer categories — they are ways of applying the primary categories.

### Priority evaluation

Categories do not have fixed priorities. Priority is project-specific and computed from four axes:

| Axis | Determined by | Varies per project? |
|------|--------------|---------------------|
| **Exposure** | How much of the category's risk surface the project touches (deployment model, data sensitivity, user base, team size, regulatory environment) | Yes — assessed per audit |
| **Gap** | How far the current state is from the acceptable threshold (maturity level vs target, metrics vs thresholds) | Yes — measured by audit results |
| **Leverage** | Whether improving this category enables or accelerates improvement in others | No — inherent to the category |
| **Accumulation** | How fast neglect compounds without active effort | No — inherent to the category |

Leverage and accumulation are stable properties of each category and are documented in the [Category Index](#category-index). Exposure and gap are assessed per-project during each audit. Priority is derived from all four: `f(exposure, gap, leverage, accumulation)`.

### Universal maturity model

Most categories follow a four-level maturity progression:

| Level | State | Indicators | Finding severity |
|-------|-------|------------|------------------|
| 0 — None | No capability | No tooling, no process | High |
| 1 — Manual | Capability exists, manually applied | Tools exist, run ad hoc | Medium |
| 2 — Automated | Integrated into workflow | Runs in CI, results stored | Low |
| 3 — Gated | Enforces quality gates | Blocks PRs on regression | Info (target state) |

Categories MAY include a domain-specific maturity model following this skeleton. See [Error Handling](#4-error-handling) and [Performance](#5-performance) for worked examples.

This model subsumes what was traditionally treated as a separate "CI/CD Pipeline" category. Rather than auditing CI/CD as its own concern, each category's maturity level captures whether it is automated and gated. See [CI/CD Enforcement](#b-cicd-enforcement).

### Remediation playbook pattern

Categories MAY include a remediation playbook: a table of common problem patterns with fix techniques and expected impact.

| Column | Purpose |
|--------|---------|
| Pattern | The problem commonly found during audit |
| Technique | How to fix it |
| Expected impact | Quantified improvement estimate |

See [Code Complexity](#2-code-complexity) for the canonical example.

### Finding templates

Audit findings should differentiate by severity. Categories MAY include templates showing what a High, Medium, and Low finding looks like in that domain. See [Test Coverage](#1-test-coverage) for worked examples.

Adapt thresholds and tooling to your project. The categories, procedures, and severity model are universal.

---

## Severity Model

All findings use a 4-level severity scale:

| Severity | Meaning | Action | Example |
|----------|---------|--------|---------|
| **High** | Critical issue, immediate fix required | Block release | Unhandled error in core path, 0% coverage module |
| **Medium** | Significant issue affecting quality | Fix within current cycle | Function complexity >15, missing branch tests |
| **Low** | Minor issue, improvement opportunity | Fix when convenient | Naming inconsistency, missing doc comment |
| **Info** | Recommendation, no current risk | Track for future | Consolidation opportunity, style preference |

### Result status (per module or component)

| Status | Condition | Meaning |
|--------|-----------|---------|
| **Passed** | No findings | All checks passed |
| **Warning** | Has medium or low findings | Issues present, not critical |
| **Failed** | Has high severity findings | Critical issues found |

---

## Category Index

### Primary categories

| # | Category | Leverage | Accumulation |
|---|----------|----------|--------------|
| 1 | [Test Coverage](#1-test-coverage) | High | High |
| 2 | [Code Complexity](#2-code-complexity) | High | Medium |
| 3 | [Code Duplication](#3-code-duplication) | Medium | High |
| 4 | [Error Handling](#4-error-handling) | Medium | Medium |
| 5 | [Performance](#5-performance) | Low | Medium |
| 6 | [Dependencies](#6-dependencies) | Medium | High |
| 7 | [Security](#7-security) | Low | Medium |
| 8 | [Documentation](#8-documentation) | Medium | Medium |
| 9 | [Localization](#9-localization) | Low | Low |

### Standards conformance

| # | Category | Leverage | Accumulation |
|---|----------|----------|--------------|
| 10 | [Project Standards](#10-project-standards) | Medium | Medium |

### Cross-cutting concerns

| ID | Concern | Role |
|----|---------|------|
| A | [Cross-Component Consistency](#a-cross-component-consistency) | Evaluation mode — applies any category's checks comparatively across components |
| B | [CI/CD Enforcement](#b-cicd-enforcement) | Enforcement layer — the maturity model applied across all categories |

---

## 1. Test Coverage

### What it measures

Percentage of code exercised by automated tests. Coverage metrics exist in tiers of increasing rigor; this framework evaluates Tier 1 (execution coverage) as the universal baseline.

### Why it matters

Untested code is unknown code. Coverage gaps correlate with undiscovered defects, particularly in error handling and edge case paths.

### Dimensions

#### Tier 1 — Execution coverage (universal baseline)

Near-universal tool support across all major language ecosystems. Report these always.

| Dimension | What it catches |
|-----------|-----------------|
| Line coverage | Dead code, untested paths |
| Branch coverage | Uncovered conditional branches |
| Function coverage | Entirely untested functions |

#### Higher tiers (adopt when applicable)

| Tier | Metrics | When to adopt |
|------|---------|---------------|
| Tier 2 — Logical | Condition coverage, MC/DC, path coverage | Regulatory/safety requirements (DO-178C, ISO 26262); boolean logic governing authorization or financial decisions |
| Tier 3 — Effectiveness | Mutation coverage | When the question shifts from "is the code executed?" to "are the tests actually catching bugs?" |
| Tier 4 — Semantic | Requirements coverage, state coverage, data flow coverage, error path coverage | When supporting infrastructure exists (requirements traceability, state model definitions) |

Tier 1 metrics are correlated but not redundant: 100% function coverage can coexist with 40% line coverage; 100% line coverage can coexist with 50% branch coverage. Tier 3 (mutation) is orthogonal to all execution-based tiers — a test suite can achieve 100% line/branch/function coverage while asserting nothing meaningful.

### Thresholds

| Metric | Excellent | Good | Acceptable | Needs Improvement | Poor |
|--------|-----------|------|------------|-------------------|------|
| Line coverage | ≥95% | ≥85% | ≥80% | ≥70% | <70% |
| Branch coverage | ≥85% | ≥75% | ≥65% | ≥55% | <55% |
| Function coverage | ≥90% | ≥80% | ≥75% | ≥65% | <65% |

> **CI gate recommendation:** Set CI gates at the "Acceptable" threshold or above. The specific threshold is project-configurable — see [CI/CD Enforcement](#b-cicd-enforcement).

### Test layering model

Tests exist in layers with different value profiles:

| Layer | Type | When to create | What it catches |
|-------|------|----------------|-----------------|
| 1 | Inline smoke tests | During implementation | API surface, basic round-trip |
| 2 | Quick/interactive tests | At module start | Individual behavior, debugging aid |
| 3 | Snapshot/fixture tests | At module start (harness), incremental (fixtures) | Regression, correctness |
| 4 | Integration/E2E tests | After all components wired | Full pipeline, cross-module interaction |
| 5 | Fuzz tests | As early as possible, runs continuously | Unknown unknowns, edge case combinatorics |

### Maturity levels

| Level | State | Indicators |
|-------|-------|------------|
| 0 | No tests | No test files, no test runner configured |
| 1 | Tests exist | Test suite runs manually, no coverage tracking |
| 2 | CI-integrated | Tests run in CI, coverage reports generated |
| 3 | Coverage-gated | PRs blocked when coverage drops below threshold |

### Audit procedure

1. Generate coverage report with line, branch, and function metrics
2. Produce per-module coverage table sorted by coverage (ascending)
3. Identify modules below 80% line coverage — these are **high priority** gaps
4. For each gap, categorize: untestable (stdin, OS-specific), expected (placeholders), or genuinely missing
5. Cross-reference with complexity data — high-complexity + low-coverage = highest risk
6. Document coverage trend if baseline exists (improving, stable, declining)
7. Assess coverage tier: is Tier 1 sufficient for this project, or do regulatory/safety/quality requirements indicate higher tiers?

### Finding templates

- **HIGH:** `Module {name} has {X}% line coverage (target: ≥80%). Contains {Y} functions with 0 test coverage.`
- **MEDIUM:** `Branch coverage is {X}% (target: ≥75%). {N} uncovered branches in error handling paths.`
- **LOW:** `Function {name} has no dedicated test. It is exercised transitively by integration tests.`

---

## 2. Code Complexity

### What it measures

Structural and cognitive complexity of individual functions. This is a per-function property — complexity analysis examines one function's control flow graph or AST in isolation, never cross-referencing other functions.

- **Cyclomatic complexity** — number of independent execution paths (structural complexity)
- **Cognitive complexity** — how hard a function is for a human to understand (nesting depth, control flow breaks)

### Why it matters

High-complexity functions are harder to test, harder to review, and more likely to harbor defects. Cyclomatic complexity directly correlates with the minimum number of test cases needed for full branch coverage.

### Thresholds

| Metric | Low (Good) | Medium | High (Refactor) |
|--------|------------|--------|-----------------|
| Cyclomatic complexity | 1-5 | 6-10 | >10 |
| Cognitive complexity | 1-8 | 9-15 | >15 |
| Function SLOC | 1-30 | 31-60 | >60 |

### Distribution targets

Aggregate thresholds measuring the codebase as a whole:

| Assessment | Threshold |
|------------|-----------|
| Excellent | ≥90% of functions have cyclomatic ≤5 |
| Good | ≥85% of functions have cyclomatic ≤5 |
| Acceptable | ≥80% of functions have cyclomatic ≤5 |
| Needs improvement | <80% of functions have cyclomatic ≤5 |

### Remediation playbook

| Pattern | Technique | Expected reduction |
|---------|-----------|-------------------|
| Nested conditionals | Early returns, guard clauses | -40-60% |
| Long match/switch | Extract to lookup table | -50-80% |
| Mixed concerns in one function | Extract helper functions | -30-50% |
| Repeated validation logic | Extract validation functions | -20-40% |

### Audit procedure

1. Run complexity analysis on all source files
2. Produce function-level table sorted by cyclomatic complexity (descending)
3. Calculate distribution: % low, % medium, % high
4. Identify all functions with cyclomatic >10 — these are refactoring candidates
5. For each high-complexity function, assess: is the complexity inherent (state machine, parser) or accidental (nested conditionals, early-return candidates)?
6. Cross-reference with coverage — high-complexity + low-coverage = highest risk

---

## 3. Code Duplication

### What it measures

Repeated code patterns across the codebase that could be consolidated into shared abstractions. This is a whole-codebase comparative property — duplication detection compares all code locations against each other to find similarity clusters. It is independent of code complexity: trivially simple code can be massively duplicated, and highly complex code can be entirely unique.

### Why it matters

Duplicated code means: bugs must be fixed in multiple places, behavior can diverge silently, and the codebase grows without adding value.

### Dimensions

| Dimension | Description | Severity |
|-----------|-------------|----------|
| Exact clones | Identical code blocks | High |
| Near clones | Same logic, slight variations | Medium |
| Structural | Same algorithm, different types | Low |
| Expected | Config loading, setup patterns | Info |

### Remediation playbook

| Pattern | Technique | Expected impact |
|---------|-----------|-----------------|
| Format/type detection | Shared utility function | Eliminate N-1 copies |
| Boilerplate preambles | Constant or helper | Reduce per-file setup |
| Validation sequences | Extract validator | Single source of truth |
| Error message construction | Localized message function | Consolidates with localization |
| Configuration loading | Shared config loader | Single loading path |

### Audit procedure

1. Run duplication detection tool
2. For each duplication cluster, identify: occurrences, files, lines
3. Classify: extractable (consolidate), expected (acceptable), or structural (different types)
4. For extractable duplications, estimate: lines saved, functions to create
5. Prioritize by: occurrence count × lines per occurrence × severity

---

## 4. Error Handling

### What it measures

How errors are created, propagated, logged, and reported to users.

### Why it matters

Poor error handling leads to: silent failures (errors swallowed), unhelpful error messages (users can't self-diagnose), lost debugging context (developers can't trace issues), and security leaks (internal details exposed).

### Dimensions

| Dimension | What to check | Severity |
|-----------|---------------|----------|
| Error typing | Are errors typed (enum/class) or stringly-typed? | Medium |
| Error propagation | Structured propagation vs crash/abort | High |
| Error context | Are errors annotated with context at each level? | Medium |
| Error logging | Are errors logged before being returned/displayed? | Medium |
| Crash surface | Where can the program crash without recovery? | High |

> **Note:** User-facing error message quality (actionability, localizability) is evaluated under [Localization](#9-localization).

### Maturity levels

| Level | Description | Indicators |
|-------|-------------|------------|
| 0 — Ad hoc | String errors, crashes, bare print statements | High severity |
| 1 — Basic | Error types exist, structured propagation, some logging | Medium |
| 2 — Structured | Typed errors with context, structured logging | Low |
| 3 — Observable | Tracing spans, error categorization, metrics | Info |

### Audit procedure

1. Count error handling patterns: typed errors vs string errors vs crash/abort
2. Identify all unhandled error points (forced unwinding, unrecoverable panics, uncaught exceptions)
3. Check logging coverage: what percentage of error paths emit a log/trace?
4. Map the error propagation chain for 2-3 critical paths end-to-end
5. Identify instrumentation gaps (functions missing tracing/logging)

---

## 5. Performance

### What it measures

Runtime characteristics: latency, throughput, resource usage, and regression over time.

### Why it matters

Performance degradation is cumulative and often invisible until it crosses a user-noticeable threshold. Without measurement, regressions compound silently across releases.

### Dimensions

| Dimension | Metrics |
|-----------|---------|
| Core operations | Latency (p50, p95, p99) |
| Throughput | Items/second |
| Resource usage | Memory, CPU |
| Scale behavior | Performance at 1x, 10x, 100x data |

### Maturity levels

| Level | Description | Indicators |
|-------|-------------|------------|
| 0 — None | No benchmarks | No performance data |
| 1 — Local | Benchmarks exist, run manually | Manual comparison |
| 2 — CI | Benchmarks run in CI, results stored | Historical tracking |
| 3 — Gated | Regressions block PRs | Automated regression detection |

### Audit procedure

1. Identify critical hot paths (most-called operations)
2. Establish baseline benchmarks
3. Categorize: fast CI checks (small data), standard (medium), stress test (large)
4. Document acceptable thresholds per benchmark
5. Evaluate regression detection strategy (CI integration vs manual)

---

## 6. Dependencies

### What it measures

Freshness, security, and policy compliance of project dependencies. Combines dependency vulnerability scanning with dependency health tracking into a single audit.

### Why it matters

Dependencies are the largest external attack surface. Outdated or vulnerable dependencies introduce risk that is entirely outside the project's control to detect without active scanning.

### Dimensions

| Dimension | What it catches | Severity |
|-----------|-----------------|----------|
| Known vulnerabilities | CVEs in dependencies | High (critical/high CVEs) |
| License compliance | Incompatible or disallowed licenses | Medium |
| Freshness | Dependencies behind latest releases | Low |
| Unused dependencies | Imported but not used | Low |
| Dependency depth | Deep transitive trees (fragility risk) | Info |
| Single-maintainer risk | Critical deps with 1 maintainer | Medium |

### Thresholds

| Metric | Target |
|--------|--------|
| Known critical/high vulnerabilities | 0 |
| License violations | 0 |
| Dependencies >2 minor versions behind | Tracked, not gated |

### Maturity levels

| Level | State | Indicators |
|-------|-------|------------|
| 0 | No tracking | Dependencies never audited |
| 1 | Manual review | Periodic manual check |
| 2 | Automated scanning | Scanner runs in CI, alerts generated |
| 3 | Gated | PRs blocked on new vulnerabilities |

### Audit procedure

1. List all direct dependencies with current and latest versions
2. Run vulnerability scanner — flag any critical or high CVEs
3. Check license compatibility against project policy
4. Flag dependencies >2 minor versions behind
5. Identify unused dependencies
6. Assess single-point-of-failure dependencies (critical dep with 1 maintainer)

---

## 7. Security

### What it measures

Code-level vulnerability exposure through coding patterns, input handling, secrets management, and permission models.

> **Note:** Dependency-level security (known CVEs, license compliance) is evaluated under [Dependencies](#6-dependencies).

### Why it matters

Code-level security flaws (injection, secrets exposure, missing access control) are exploitable regardless of dependency health. They require different detection tools and different remediation approaches than dependency vulnerabilities.

### Dimensions

| Dimension | What it catches | Severity |
|-----------|-----------------|----------|
| Secret detection | Hardcoded credentials, API keys | High |
| Input validation | Injection vectors (SQL, command, XSS) | High |
| Permission model | Excessive privileges, missing guards | High |
| Information exposure | Internal details leaked in errors/responses | Medium |

### Thresholds

| Metric | Target |
|--------|--------|
| Hardcoded secrets | 0 |
| Input validation at boundaries | 100% of external inputs |
| SAST findings (critical/high) | 0 |

### Audit procedure

1. Scan for hardcoded secrets/credentials
2. Review all external input points for validation
3. Check for common vulnerability patterns (OWASP top 10)
4. Verify command/process invocations have proper guards
5. Run SAST scanner and triage findings (accounting for false positives)

---

## 8. Documentation

### What it measures

Coverage and quality of documentation at the API, module, and architectural levels.

### Why it matters

Documentation is the primary onboarding tool. Undocumented code forces every new contributor to reverse-engineer intent from implementation, multiplying the cost of every future change.

> **Note:** Required file existence (README, CHANGELOG, LICENSE) is evaluated under [Project Standards](#10-project-standards). This category focuses on documentation *content and quality*.

### Thresholds

| Level | Target | Check method |
|-------|--------|-------------|
| Public API items | ≥70% documented | Doc coverage tool |
| Module-level docs | All public modules documented | Manual review |
| Architecture docs | Exist for complex subsystems | Manual review |

### Maturity levels

| Level | State | Indicators |
|-------|-------|------------|
| 0 | No documentation | No doc comments, no README content |
| 1 | Basic | README exists with purpose and usage |
| 2 | Systematic | API docs generated, doc coverage tracked |
| 3 | Enforced | Doc coverage checked in CI, broken links blocked |

### Audit procedure

1. Generate documentation coverage report
2. Check all public modules for module-level documentation
3. Check for broken documentation links
4. Assess quality: are docs descriptive or just restating the function name?
5. Verify architectural documentation exists for complex subsystems

---

## 9. Localization

### What it measures

Readiness for multi-language and multi-locale support. Measures the gap between hardcoded user-facing strings and those routed through a localization system. Includes the quality of user-facing messages (error messages, CLI output, UI text).

> **Terminology:** This category covers both internationalization (i18n — making software locale-ready) and localization (l10n — implementing specific locale support). "Localization" is used as the category name because it encompasses the full pipeline from readiness to delivery.

### Why it matters

Hardcoded user-facing strings prevent adoption in non-English-speaking markets and make it impossible to provide consistent, high-quality messages across the application. Localizable error messages and CLI output are as important as UI text.

### Dimensions

| Dimension | Check | Severity |
|-----------|-------|----------|
| Localization coverage | % of user-facing strings going through localization | Medium |
| Hardcoded string detection | User-visible strings not routed through localization | Medium |
| Locale completeness | All messages implemented for each supported locale | Low |
| Format string safety | Parameters use named/positional placeholders | High |
| Error message quality | User-facing errors are actionable and localizable | Medium |

### Maturity levels

| Level | State | Indicators |
|-------|-------|------------|
| 0 | All hardcoded | No localization system, all strings inline |
| 1 | Partial | Some strings routed through localization |
| 2 | Systematic | Localization pipeline exists, coverage tracked |
| 3 | Complete | Full locale support, coverage gated in CI |

### Audit procedure

1. Inventory all user-facing output points (messages, errors, CLI output, UI text)
2. Classify each: routed through localization system, or hardcoded
3. Calculate localization coverage percentage
4. For each hardcoded string, assess: user-facing (must fix) or internal (acceptable)
5. Check format strings for named/positional placeholder safety
6. Verify error messages are actionable (user can understand what went wrong and what to do)

---

## 10. Project Standards

### What it measures

Adherence to project conventions: naming rules, file organization, coding patterns, metadata completeness, and required file presence. This category uses a declarative check mechanism — each standard is expressed as a verifiable rule.

### Why it matters

Inconsistent conventions increase cognitive load. When naming, structure, and patterns aren't standardized, contributors spend time guessing conventions instead of reading code. Declarative standards make expectations explicit and automatable.

### Standard domains

| Domain | What it checks | Examples |
|--------|---------------|----------|
| Naming conventions | Case rules, prefixes, hierarchy consistency | Function casing, type casing, CLI command hierarchy |
| File organization | Directory structure, required files, placement | README, LICENSE, CHANGELOG, CI config present |
| Coding patterns | Required patterns present, anti-patterns absent | Uses shared helpers, no hardcoded config values |
| Metadata | Project metadata completeness | Version, description, license declared |
| Configuration | Configuration follows conventions | Namespaced settings, documented defaults |

### Naming convention checks

| Target | Check method |
|--------|-------------|
| Functions/methods | Linter (case convention) |
| Types/classes | Linter (case convention) |
| Constants | Linter (case convention) |
| Files/modules | Pattern check against convention |
| CLI commands/subcommands | Manual review (consistent hierarchy) |
| Error variants | Manual review (subject-first consistency) |
| Test functions | Manual review (descriptive naming) |
| Configuration keys | Manual review (consistent case) |

### Check type taxonomy

Standards are expressed as declarative checks:

| Check type | What it verifies |
|------------|-----------------|
| `file_exists` | Required file is present |
| `directory_exists` | Required directory is present |
| `content_match` | Required pattern found in file(s) |
| `content_not_match` | Anti-pattern absent from file(s) |
| `regex_match` | Complex pattern match |
| `metadata_field` | Metadata field has expected value |
| `semver_version` | Version follows semantic versioning |
| `uses_helper` | Component uses shared utility (not raw reimplementation) |

### Maturity levels

| Level | State | Indicators |
|-------|-------|------------|
| 0 | No standards defined | Ad hoc conventions, inconsistent |
| 1 | Documented | Standards written down, manual review |
| 2 | Linted | Linter rules enforce naming/pattern standards in CI |
| 3 | Fully declarative | All standards expressed as automated checks, violations block PRs |

### Audit procedure

1. Define standards checklist using the check type taxonomy (declarative format)
2. Run language linter with naming rules enabled
3. Verify required files exist: README, LICENSE, CHANGELOG, .gitignore, CI config
4. Check directory structure against project conventions
5. List all public API names and check for naming consistency
6. Verify command hierarchy follows a consistent pattern
7. Check error type naming for consistency
8. For each check: record pass/fail, severity, skip conditions
9. Calculate pass rate by severity level
10. Document intentional deviations with rationale

---

## A. Cross-Component Consistency

### What it is

An evaluation mode, not a standalone quality dimension. Cross-component consistency means: apply any primary category's checks across N components and compare the results. It detects divergence that component-level audits miss.

### Why it matters

Individual components may each pass their own checks but be inconsistent with each other — naming mismatches, dependency asymmetry, pattern divergence. These inconsistencies create confusion and increase maintenance cost.

### Application examples

| Primary category | Cross-component check |
|------------------|-----------------------|
| Code Duplication (§3) | Shared utility usage — components use shared helpers vs reimplemented |
| Project Standards (§10) | Naming consistency — all components follow the same conventions |
| Dependencies (§6) | Dependency consistency — required shared deps declared everywhere |
| Project Standards (§10) | Pattern consistency — same coding patterns across similar components |

### Audit procedure

1. Identify component groups that should be consistent
2. Select which primary category checks to apply comparatively
3. Run checks across all components simultaneously
4. Flag inconsistencies with: which components differ, what the expected pattern is
5. Categorize: intentional divergence (document why) vs accidental (fix)

---

## B. CI/CD Enforcement

### What it is

An enforcement layer, not a standalone quality dimension. CI/CD enforcement answers: which primary categories have automated quality gates? This is the [universal maturity model](#universal-maturity-model) applied across all categories.

### Why it matters

Quality checks that aren't automated degrade over time. CI enforcement converts quality aspirations into mechanical guarantees.

### Assessment method

For each primary category, assess its maturity level:

| Primary category | Level 0 | Level 1 | Level 2 | Level 3 |
|------------------|---------|---------|---------|---------|
| Test Coverage | No tests | Tests run manually | Tests in CI | Coverage-gated |
| Code Complexity | No measurement | Manual review | Analysis in CI | Complexity limits block PRs |
| Code Duplication | No detection | Manual review | Detection in CI | Duplication limits block PRs |
| Error Handling | No checks | Manual review | Pattern linting in CI | Violations block PRs |
| Performance | No benchmarks | Manual benchmarks | Benchmarks in CI | Regressions block PRs |
| Dependencies | No scanning | Manual audit | Scanner in CI | Vulnerabilities block PRs |
| Security | No scanning | Manual review | SAST in CI | Findings block PRs |
| Documentation | No checks | Manual review | Doc build in CI | Broken links block PRs |
| Localization | No checks | Manual review | String extraction in CI | Coverage tracked |
| Project Standards | No checks | Manual review | Lint rules in CI | Violations block PRs |

### CI gate thresholds (recommended)

| Gate | Threshold | Notes |
|------|-----------|-------|
| All tests pass | Required | Non-negotiable |
| Coverage | ≥80% project, ≥80% patch | Configurable per project |
| Lint warnings | 0 | Clean lint required |
| Format violations | 0 | Clean formatting required |
| Known critical/high vulnerabilities | 0 | From Dependencies category |

### Audit procedure

1. Verify CI runs on: push to main, PRs to main
2. For each primary category, assess current maturity level (0-3)
3. Check each CI job: does it run? Does it block on failure?
4. Verify cache strategy (build cache, dependency cache)
5. Measure CI duration — if >10 min, identify optimization opportunities
6. Produce maturity matrix (categories × levels) showing current state and target

---

## Audit Report Structure

A complete quality audit produces a report with:

```
1. Executive Summary
   - Overall assessment (letter grade or pass/warn/fail)
   - Key metrics table (coverage, complexity, findings count by severity)
   - Top 3 risks
   - Top 3 improvements since last audit (if applicable)

2. Category Results (one section per category evaluated)
   - Current metrics
   - Assessment vs thresholds
   - Maturity level (0-3)
   - Findings (itemized, with severity)
   - Trend (improving/stable/declining) if baseline exists

3. Cross-Cutting Findings
   - Cross-component consistency results
   - CI/CD maturity matrix
   - Issues that span multiple categories

4. Recommendations
   - Prioritized by computed priority (exposure × gap × leverage × accumulation)
   - Each with: what to do, estimated effort, expected impact

5. Metrics Comparison (if baseline exists)
   - Before/after table
   - Targets met / not met

6. Methodology
   - Tools used
   - Files analyzed
   - Exclusions and rationale
```

---

## Remediation Plan Template

For each audit finding that requires action:

```
### Finding: {Title}

**Severity:** High | Medium | Low | Info
**Category:** {Category name}
**Location:** {File(s) and line range(s)}

**Current state:** {What exists now}
**Target state:** {What it should look like}
**Strategy:** {How to get from current to target}

**Estimated effort:** Low (minutes) | Medium (hours) | High (days)
**Verification:** {How to confirm the fix}
```

---

## Phased Audit Approach

Not all categories need to be evaluated at once. Phase ordering follows leverage — high-leverage categories are evaluated first because their results inform the assessment of other categories.

| Phase | Categories | Rationale |
|-------|-----------|-----------|
| 1 — Baseline | Test Coverage, Code Complexity, CI/CD maturity matrix | Highest leverage; their results cross-reference into all other categories |
| 2 — Risk reduction | Error Handling, Security, Dependencies | High accumulation or high exposure risk; benefit from Phase 1 baselines |
| 3 — Consistency | Code Duplication, Project Standards, Cross-Component check | Medium leverage; consolidation findings inform Phase 4 |
| 4 — Polish | Documentation, Localization, Performance | Lower leverage; benefit from all prior phases being resolved |
| 5 — Continuous | All categories, trend tracking | Recurring (quarterly) |

---

## Design Principles

Six principles observed across multiple production quality programs:

1. **Audit-first** — establish baselines before remediation
2. **Phased remediation** — prioritize by severity, not by category
3. **Dual-axis risk** — the highest-risk code is simultaneously low-coverage and high-complexity
4. **Complementary refactoring** — consolidating duplication creates natural injection points for localization and error handling improvements
5. **Declarative checklists** — expressing compliance checks as data (not code) makes them more maintainable and auditable
6. **Severity-driven prioritization** — finding severity guides remediation order, not arbitrary category ordering

---

This document is an **attractor** — a term borrowed from dynamical systems theory. An attractor is a set of states toward which a system tends to evolve, regardless of starting conditions. Once a trajectory enters the attractor's basin, it stays — not because it's forced, but because the dynamics naturally pull it there.

A standard says "you must." A guide says "you should." A best practice says "others do." An attractor says "you will, because the alternative costs more effort."

| Property of a gravitational attractor | Property of this document |
|---|---|
| Acts at a distance — influences before contact | Shapes thinking before someone reads every section |
| Creates orbits — stable, productive trajectories | Contributors settle into consistent patterns without rigid enforcement |
| Deviation requires energy — leaving is harder than staying | Once adopted, going against the framework is more work than following it |
| Proportional to mass — the more substantial, the stronger the pull | The more complete and internally consistent it is, the harder it is to ignore |
| Doesn't require active enforcement | No one polices it — the structure itself makes the right thing the easy thing |
