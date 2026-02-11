# Quality Evaluation Framework

A universal framework for software quality auditing and best practices. Applicable to any codebase regardless of language, domain, or scale.

## How to use this document

This framework defines **what to evaluate** and **how to evaluate it**. Each category includes:

- **What it measures** — the quality dimension
- **Why it matters** — the risk of not measuring it
- **How to measure** — concrete techniques
- **Thresholds** — targets and assessment grades
- **Audit procedure** — step-by-step evaluation process

Adapt thresholds and tooling to your project. The categories, procedures, and severity model are universal.

---

## Category Index

| # | Category | Priority |
|---|----------|----------|
| 1 | [Test Coverage](#1-test-coverage) | High |
| 2 | [Code Complexity](#2-code-complexity) | High |
| 3 | [Naming Conventions](#3-naming-conventions) | Medium |
| 4 | [Error Handling](#4-error-handling) | High |
| 5 | [Code Duplication (DRY)](#5-code-duplication-dry) | Medium |
| 6 | [Documentation](#6-documentation) | Medium |
| 7 | [Security](#7-security) | High |
| 8 | [Performance](#8-performance) | Low |
| 9 | [Internationalization](#9-internationalization) | Medium |
| 10 | [CI/CD Pipeline](#10-cicd-pipeline) | High |
| 11 | [Dependency Health](#11-dependency-health) | Medium |
| 12 | [Compliance & Standards](#12-compliance--standards) | Medium |
| 13 | [Cross-Component Validation](#13-cross-component-validation) | Medium |
| 14 | [File Organization](#14-file-organization) | Low |

---

## Severity Model

All findings use a 4-level severity scale:

| Severity | Meaning | Action | Example |
|----------|---------|--------|---------|
| **High** | Critical issue, immediate fix required | Block release | Unhandled error in core path, 0% coverage module |
| **Medium** | Significant issue affecting quality | Fix within current cycle | Function complexity >20, missing branch tests |
| **Low** | Minor issue, improvement opportunity | Fix when convenient | Naming inconsistency, missing doc comment |
| **Info** | Recommendation, no current risk | Track for future | Consolidation opportunity, style preference |

### Result Status (per module or component)

| Status | Condition | Meaning |
|--------|-----------|---------|
| **Passed** | No findings | All checks passed |
| **Warning** | Has medium or low findings | Issues present, not critical |
| **Failed** | Has high severity findings | Critical issues found |

---

## 1. Test Coverage

### What it measures

Percentage of code exercised by automated tests. Three dimensions: line coverage, branch coverage, and function coverage.

### Why it matters

Untested code is unknown code. Coverage gaps correlate with undiscovered defects, particularly in error handling and edge case paths.

### How to measure

| Dimension | What it catches |
|-----------|-----------------|
| Line coverage | Dead code, untested paths |
| Branch coverage | Uncovered conditional branches |
| Function coverage | Entirely untested functions |

### Thresholds

| Metric | Excellent | Good | Acceptable | Needs Improvement | Poor |
|--------|-----------|------|------------|-------------------|------|
| Line coverage | ≥95% | ≥85% | ≥80% | ≥70% | <70% |
| Branch coverage | ≥85% | ≥75% | ≥65% | ≥55% | <55% |
| Function coverage | ≥90% | ≥80% | ≥75% | ≥65% | <65% |

### Test layering model

Tests exist in layers with different value profiles:

| Layer | Type | When to create | What it catches |
|-------|------|----------------|-----------------|
| 1 | Inline smoke tests | During implementation | API surface, basic round-trip |
| 2 | Quick/interactive tests | At module start | Individual behavior, debugging aid |
| 3 | Snapshot/fixture tests | At module start (harness), incremental (fixtures) | Regression, correctness |
| 4 | Integration/E2E tests | After all components wired | Full pipeline, cross-module interaction |
| 5 | Fuzz tests | As early as possible, runs continuously | Unknown unknowns, edge case combinatorics |

### Audit procedure

1. Generate coverage report with line, branch, and function metrics
2. Produce per-module coverage table sorted by coverage (ascending)
3. Identify modules below 80% line coverage — these are **high priority** gaps
4. For each gap, categorize: untestable (stdin, OS-specific), expected (placeholders), or genuinely missing
5. Cross-reference with complexity data — high-complexity + low-coverage = highest risk
6. Document coverage trend if baseline exists (improving, stable, declining)

### Finding templates

- **HIGH:** `Module {name} has {X}% line coverage (target: ≥80%). Contains {Y} functions with 0 test coverage.`
- **MEDIUM:** `Branch coverage is {X}% (target: ≥75%). {N} uncovered branches in error handling paths.`
- **LOW:** `Function {name} has no dedicated test. It is exercised transitively by integration tests.`

---

## 2. Code Complexity

### What it measures

Two dimensions of function complexity:

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

| Assessment | Threshold |
|------------|-----------|
| Excellent | ≥90% of functions have cyclomatic ≤5 |
| Good | ≥85% of functions have cyclomatic ≤5 |
| Acceptable | ≥80% of functions have cyclomatic ≤5 |
| Needs improvement | <80% of functions have cyclomatic ≤5 |

### Audit procedure

1. Run complexity analysis on all source files
2. Produce function-level table sorted by cyclomatic complexity (descending)
3. Calculate distribution: % low, % medium, % high
4. Identify all functions with cyclomatic >10 — these are refactoring candidates
5. For each high-complexity function, assess: is the complexity inherent (state machine, parser) or accidental (nested conditionals, early-return candidates)?
6. Cross-reference with coverage — high-complexity + low-coverage = highest risk

### Refactoring strategies

| Pattern | Technique | Expected reduction |
|---------|-----------|-------------------|
| Nested conditionals | Early returns, guard clauses | -40-60% |
| Long match/switch | Extract to lookup table | -50-80% |
| Mixed concerns in one function | Extract helper functions | -30-50% |
| Repeated validation logic | Extract validation functions | -20-40% |

---

## 3. Naming Conventions

### What it measures

Consistency of naming across the codebase: variables, functions, types, modules, files, and commands.

### Why it matters

Inconsistent naming increases cognitive load. When conventions aren't followed, contributors spend time guessing names instead of reading code.

### Categories to evaluate

| Category | Check method |
|----------|-------------|
| Functions/methods | Linter (case convention) |
| Types/classes | Linter (case convention) |
| Constants | Linter (case convention) |
| Files/modules | Pattern check against convention |
| CLI commands/subcommands | Manual review (consistent hierarchy) |
| Error variants | Manual review (subject-first consistency) |
| Test functions | Manual review (descriptive naming) |
| Configuration keys | Manual review (consistent case) |

### Audit procedure

1. Run language linter with naming rules enabled
2. List all public API names and check for consistency
3. Verify command hierarchy follows a consistent pattern
4. Check error type naming for subject-first consistency
5. Document any intentional deviations with rationale

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
| User-facing messages | Are error messages actionable and localizable? | Medium |
| Crash surface | Where can the program crash without recovery? | High |

### Maturity levels

| Level | Description | Indicators |
|-------|-------------|------------|
| 0 — Ad hoc | String errors, crashes, bare print statements | High severity |
| 1 — Basic | Error types exist, structured propagation, some logging | Medium |
| 2 — Structured | Typed errors with context, structured logging, i18n | Low |
| 3 — Observable | Tracing spans, error categorization, metrics | Info |

### Audit procedure

1. Count error handling patterns: typed errors vs string errors vs crash/abort
2. Identify all unhandled error points (unwrap, bare raise, uncaught exceptions)
3. Check logging coverage: what percentage of error paths emit a log/trace?
4. Verify user-facing error messages are localizable (not hardcoded)
5. Map the error propagation chain for 2-3 critical paths end-to-end
6. Identify instrumentation gaps (functions missing tracing/logging)

---

## 5. Code Duplication (DRY)

### What it measures

Repeated code patterns that could be consolidated into shared abstractions.

### Why it matters

Duplicated code means: bugs must be fixed in multiple places, behavior can diverge silently, and the codebase grows without adding value.

### Types of duplication

| Type | Description | Severity |
|------|-------------|----------|
| Exact clones | Identical code blocks | High |
| Near clones | Same logic, slight variations | Medium |
| Structural | Same algorithm, different types | Low |
| Expected | Config loading, setup patterns | Info |

### Audit procedure

1. Run duplication detection tool
2. For each duplication cluster, identify: occurrences, files, lines
3. Classify: extractable (consolidate), expected (acceptable), or structural (different types)
4. For extractable duplications, estimate: lines saved, functions to create
5. Prioritize by: occurrence count x lines per occurrence x severity

### Common consolidation targets

| Pattern | Consolidation strategy |
|---------|----------------------|
| Format/type detection | Shared utility function |
| Boilerplate preambles | Constant or helper |
| Validation sequences | Extract validator |
| Error message construction | Localized message function |
| Configuration loading | Shared config loader |

---

## 6. Documentation

### What it measures

Coverage and quality of documentation at multiple levels: API docs, module docs, README, architectural docs.

### Thresholds

| Level | Target | Check method |
|-------|--------|-------------|
| Public API items | ≥70% documented | Doc coverage tool |
| Module-level docs | All public modules documented | Manual review |
| README | Exists with: purpose, install, usage, contributing | File check |
| CHANGELOG | Exists, follows keepachangelog format | File check |
| Architecture docs | Exist for complex subsystems | Manual review |

### Audit procedure

1. Generate documentation coverage report
2. Check all public modules for module-level documentation
3. Verify README completeness (sections: purpose, install, usage, contributing)
4. Check for broken documentation links
5. Assess quality: are docs descriptive or just restating the function name?

---

## 7. Security

### What it measures

Vulnerability exposure through dependencies, code patterns, and configuration.

### Dimensions

| Dimension | What it catches |
|-----------|-----------------|
| Dependency vulnerabilities | Known CVEs in dependencies |
| Dependency policy | Disallowed licenses, untrusted sources |
| Secret detection | Hardcoded credentials, API keys |
| Input validation | Injection vectors (SQL, command, XSS) |
| Permission model | Excessive privileges, missing guards |

### Thresholds

| Metric | Target |
|--------|--------|
| Known critical vulnerabilities | 0 |
| Known high vulnerabilities | 0 |
| Hardcoded secrets | 0 |
| Input validation at boundaries | 100% of external inputs |

### Audit procedure

1. Run dependency vulnerability scanner
2. Run dependency policy checker (license compliance)
3. Scan for hardcoded secrets/credentials
4. Review all external input points for validation
5. Check for common vulnerability patterns (OWASP top 10)
6. Verify command/process invocations have proper guards

---

## 8. Performance

### What it measures

Runtime characteristics: latency, throughput, resource usage, and regression over time.

### Benchmark categories

| Category | Metrics |
|----------|---------|
| Core operations | Latency (p50, p95, p99) |
| Throughput | Items/second |
| Resource usage | Memory, CPU |
| Scale behavior | Performance at 1x, 10x, 100x data |

### Audit procedure

1. Identify critical hot paths (most-called operations)
2. Establish baseline benchmarks
3. Categorize: fast CI checks (small data), standard (medium), stress test (large)
4. Document acceptable thresholds per benchmark
5. Evaluate regression detection strategy (CI integration vs manual)

### Maturity levels

| Level | Description |
|-------|-------------|
| 0 — None | No benchmarks |
| 1 — Local | Benchmarks exist, run manually |
| 2 — CI | Benchmarks run in CI, results stored |
| 3 — Gated | Regressions block PRs |

---

## 9. Internationalization

### What it measures

Readiness for multi-language/locale support. Measures the gap between hardcoded user-facing strings and those routed through a localization system.

### Dimensions

| Dimension | Check | Severity |
|-----------|-------|----------|
| Localization coverage | % of user-facing strings going through i18n | Medium |
| Hardcoded string detection | User-visible strings not routed through i18n | Medium |
| Locale completeness | All messages implemented for each supported locale | Low |
| Format string safety | Parameters use named/positional placeholders | High |

### Audit procedure

1. Inventory all user-facing output points (messages, errors, logs)
2. Classify each: routed through i18n, or hardcoded
3. Calculate i18n coverage percentage
4. For each hardcoded string, assess: user-facing (must fix) or internal (acceptable)
5. Check for consolidation opportunities — can i18n gaps be fixed alongside DRY refactoring?

---

## 10. CI/CD Pipeline

### What it measures

Automated quality enforcement through continuous integration and deployment.

### Required CI jobs

| Job | Purpose | Blocking? |
|-----|---------|-----------|
| Test | Run full test suite | Yes |
| Coverage | Generate + upload coverage report | Yes (if below threshold) |
| Lint | Static analysis | Yes |
| Format | Code formatting check | Yes |
| Security | Dependency audit | Continue-on-error initially |
| Docs | Documentation build | Yes (broken links) |

### Pipeline thresholds

| Metric | Target |
|--------|--------|
| All tests pass | Required |
| Project coverage | ≥85% (configurable) |
| Patch coverage | ≥80% (new code) |
| Lint warnings | 0 |
| Format violations | 0 |
| Known vulnerabilities | 0 critical/high |

### Audit procedure

1. Verify CI runs on: push to main, PRs to main
2. Check each job: does it run? Does it block on failure?
3. Verify coverage reporting integration
4. Verify cache strategy (build cache, dependency cache)
5. Measure CI duration — if >10 min, identify optimization opportunities
6. Check for missing jobs: security scanning, doc building, benchmark comparison

---

## 11. Dependency Health

### What it measures

Freshness, security, and policy compliance of project dependencies.

### Dimensions

| Dimension | Check |
|-----------|-------|
| Outdated dependencies | How many deps are behind latest? |
| Vulnerability status | Any known CVEs? |
| License compliance | All licenses compatible? |
| Unused dependencies | Any deps imported but not used? |
| Dependency depth | How deep is the transitive tree? |

### Audit procedure

1. List all direct dependencies with current and latest versions
2. Flag any dependency >2 minor versions behind
3. Run vulnerability scanner
4. Check license compatibility
5. Identify unused dependencies
6. Assess: are there single-point-of-failure dependencies (critical dep with 1 maintainer)?

---

## 12. Compliance & Standards

### What it measures

Adherence to project-specific, domain-specific, or organizational coding standards beyond language conventions.

### Standard types

| Category | What it checks | Example checks |
|----------|---------------|----------------|
| File organization | Directory structure, file placement | Required files exist, correct locations |
| Metadata | Project metadata completeness | Version, description, license declared |
| Resource patterns | Idiomatic resource usage | Guards on side-effects, action declarations |
| Code patterns | Required patterns present, anti-patterns absent | Uses shared helpers, no hardcoded values |
| Presence verification | Required files/directories exist | README, CHANGELOG, config files |
| Configuration standards | Configuration follows conventions | Namespaced settings, documented defaults |

### Check type taxonomy

Compliance checks can be expressed as declarative rules:

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

### Audit procedure

1. Define checklist of required standards (declarative format preferred)
2. For each check: define type, severity, expected value, skip conditions
3. Run automated checks against codebase
4. Aggregate results per component
5. Calculate pass rate by severity level
6. Generate findings with remediation guidance

---

## 13. Cross-Component Validation

### What it measures

Consistency across components/modules that should follow the same patterns but are maintained independently.

### Why it matters

Individual components may each pass their own checks but be inconsistent with each other — naming mismatches, dependency asymmetry, pattern divergence.

### Dimensions

| Dimension | What it checks | Severity |
|-----------|---------------|----------|
| Naming consistency | Components follow same naming convention | Medium |
| Shared utility usage | Components use shared helpers (not reimplemented) | Medium-High |
| Dependency consistency | Required shared dependencies declared everywhere | High |
| Pattern consistency | Same patterns used across similar components | Medium |
| Anti-pattern detection | No component uses known anti-patterns | High |
| Configuration alignment | Settings/options consistent across components | Low |

### Audit procedure

1. Identify component groups that should be consistent
2. Define cross-component checks: naming, dependency, pattern, anti-pattern
3. Run checks across all components simultaneously
4. Flag inconsistencies with: which components differ, what the expected pattern is
5. Categorize: intentional divergence (document why) vs accidental (fix)

---

## 14. File Organization

### What it measures

Whether the project's directory structure follows conventions and is navigable.

### Dimensions

| Dimension | Check |
|-----------|-------|
| Standard files present | README, LICENSE, CHANGELOG, CI config |
| Source organization | Logical grouping (by feature, by layer, by type) |
| Test co-location | Tests near source, or in parallel test tree |
| Config co-location | Configuration files at project root or dedicated directory |
| Generated files | Clearly separated from hand-written code |
| Documentation | In dedicated directory or inline |

### Audit procedure

1. List all top-level files and directories
2. Verify standard files: README, LICENSE, CHANGELOG, .gitignore, CI config
3. Assess source organization: is it consistent? Could a new contributor find things?
4. Check for orphaned files (not imported/referenced)
5. Verify generated code is clearly marked/separated

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
   - Findings (itemized, with severity)
   - Trend (improving/stable/declining) if baseline exists

3. Cross-Cutting Findings
   - Issues that span multiple categories
   - Systemic patterns

4. Recommendations
   - High priority (severity: high, effort: varies)
   - Medium priority
   - Low priority
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

**Severity:** High | Medium | Low
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

Not all categories need to be evaluated at once:

| Phase | Categories | When |
|-------|-----------|------|
| 1 — Baseline | Coverage, Complexity, CI/CD | First audit |
| 2 — Risk reduction | Error Handling, Security, Dependency Health | After baseline |
| 3 — Consistency | Naming, DRY, Compliance, Cross-Component | After risk reduction |
| 4 — Polish | Documentation, i18n, Performance, File Organization | After consistency |
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
