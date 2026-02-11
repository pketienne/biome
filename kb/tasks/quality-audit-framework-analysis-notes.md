# Quality Audit Framework — Analysis Notes

Working notes from framework development. These capture analysis that informed design decisions in `quality-audit-framework.md` but aren't part of the framework itself.

---

## Priority is a computed result, not a framework property

The original framework assigned fixed priority labels (High/Medium/Low) to each category. These were artifacts from real-world audit reports that got accidentally baked into the generic framework as if they were universal truths. Priorities belong in audit reports, not in the attractor.

### Priority lineage from the original framework

| New category | Priority | Inherited from | Original priority |
|---|---|---|---|
| Test Coverage | High | §1 Test Coverage | High |
| Code Complexity | High | §2 Code Complexity | High |
| Code Duplication | Medium | §5 Code Duplication | Medium |
| Error Handling | High | §4 Error Handling | High |
| Performance | Low | §8 Performance | Low |
| Dependencies | **High** | §7 Security (High) + §11 Dep Health (Medium) | Took the higher |
| Security | High | §7 Security | High |
| Documentation | Medium | §6 Documentation | Medium |
| Localization | Medium | §9 Internationalization | Medium |
| Project Standards | **Medium** | §3 Naming (Medium) + §12 Compliance (Medium) + §14 File Org (Low) | Took the majority |
| Cross-Component Consistency | **—** | §13 Cross-Component Validation | Medium — but dropped during reclassification |
| CI/CD Enforcement | **—** | §10 CI/CD Pipeline | High — but dropped during reclassification |

The cross-cutting concerns lost their priority assignments when reclassified from peer categories to evaluation modes. That was a silent decision with no stated rationale. The implicit reasoning: if something isn't a quality dimension but a way of applying quality dimensions, it doesn't compete for priority alongside them — it's always relevant when any primary category is being evaluated. But that reasoning cuts both ways. CI/CD Enforcement was High in the original, and reclassifying it as cross-cutting could be read as either promoting it (now woven into every category's maturity level) or demoting it (lost its top-level priority signal).

### Four axes for computing priority

Priority for a category is a function of four evaluation axes:

**1. Exposure** — How much of this category's risk surface does the project actually touch?

Entirely project-contextual. A payment processor has high Security exposure; an internal CLI tool has low. A single-developer project has low Documentation exposure; a library with 40 contributors has high. Determined by: deployment model, data sensitivity, user base size, team size/turnover, regulatory environment.

**2. Gap** — How far is the current state from the acceptable threshold?

What the audit measures. A category at maturity level 0 has a larger gap than one at level 2. Directly derivable from audit results — no judgment needed beyond what the maturity model and thresholds already define.

**3. Leverage** — Does improving this category enable or accelerate improvement in others?

Mostly inherent to the category and relatively stable across projects:

| Category | Leverage | Why |
|---|---|---|
| Test Coverage | High | Gates CI enforcement, enables safe refactoring, validates error handling |
| Code Complexity | High | Simpler code is easier to test, review, document, handle errors in |
| Code Duplication | Medium | Consolidation creates injection points for localization and error handling |
| Error Handling | Medium | Structured errors enable logging, debugging, user experience |
| Dependencies | Medium | Healthy deps reduce security surface, enable tooling updates |
| Documentation | Medium | Enables onboarding, which enables everything else |
| Project Standards | Medium | Consistent conventions reduce cognitive load for all other work |
| Performance | Low | Improvements don't generally unlock other quality dimensions |
| Security | Low | Fixes don't generally unlock other quality dimensions |
| Localization | Low | Readiness doesn't unlock other dimensions |

**4. Accumulation** — How fast does neglect compound?

Also mostly inherent to the category:

| Category | Accumulation | Why |
|---|---|---|
| Test Coverage | High | Every untested feature expands the untested surface; coverage ratio declines as code grows even without intent |
| Dependencies | High | Vulnerabilities discovered externally on a continuous basis; freshness degrades with calendar time regardless of project activity |
| Code Duplication | High | Each copy creates another divergence point; grows combinatorially as features reuse the copied pattern |
| Code Complexity | Medium | Grows function by function; doesn't self-accelerate |
| Error Handling | Medium | Debt grows linearly with new error paths |
| Performance | Medium | Regressions compound per release, but each is individually bounded |
| Documentation | Medium | Debt grows linearly with codebase size |
| Project Standards | Medium | Convention drift grows with team size and time |
| Security | Medium | New code may introduce new vectors, but existing ones don't multiply on their own |
| Localization | Low | Relatively stable — doesn't worsen without active changes |

### Key structural insight

Leverage and accumulation are properties of the categories themselves — they can be documented in the framework as stable reference data. Exposure and gap are properties of the project — they must be assessed per-audit. A project only needs to evaluate two axes to derive priorities, because the other two are already known.

Priority then becomes: `f(exposure, gap, leverage, accumulation)` — not a fixed label, but a computed result that changes as the project evolves and as audit results update the gap.

### Open questions

1. Should cross-cutting concerns have priorities? If they're always applicable, priority may not be the right axis — maybe "when to apply" (every audit vs periodic) is more useful.
2. What scoring model maps four axes to a single priority? Simple options: weighted sum, max-of-four, threshold-based (any axis High → priority High).
3. Should leverage and accumulation be added to each category's section in the framework, or kept as a separate reference table?

---

## Test coverage dimensions are Tier 1 of a richer taxonomy

The framework defines three dimensions of test coverage: line, branch, and function. These are the universally tooled tier of a much larger landscape. They're defensible as a pragmatic default but not defensible as "the three dimensions of test coverage."

### Tier 1 — Execution coverage (near-universal tool support)

What the framework currently covers. Every major language ecosystem has tooling that reports them out of the box.

| Metric | What it measures | Tool support |
|---|---|---|
| **Line/statement** | % of executable lines executed | gcov, lcov, JaCoCo, coverage.py, istanbul/c8, go test -cover, tarpaulin, SimpleCov, dotCover |
| **Branch** | % of conditional branches taken (both sides of every if/else) | Same tools, sometimes requires explicit flag |
| **Function/method** | % of functions called at least once | Same tools, generally reported alongside line |

These three are correlated but not redundant: 100% function coverage can coexist with 40% line coverage (every function entered, most code skipped). 100% line coverage can coexist with 50% branch coverage (only happy-path branches taken). They form a useful trio because each catches something the others miss.

### Tier 2 — Logical coverage (domain-specific, commercial/specialized tooling)

| Metric | What it measures | When it matters | Tool support |
|---|---|---|---|
| **Condition coverage** | Each boolean sub-expression evaluated to both true and false independently | `if (a && b)` — branch coverage needs the whole expression true/false; condition coverage needs `a` and `b` independently flipped | VectorCAST, LDRA, Parasoft; not in standard open-source tools |
| **MC/DC** (Modified Condition/Decision Coverage) | Each condition shown to independently affect the decision outcome | Required by DO-178C Level A (avionics), ISO 26262 ASIL D (automotive) | VectorCAST, LDRA, Rapita — specialized commercial tools |
| **Path coverage** | % of unique execution paths through a function | Theoretically complete but exponential (2^n for n branches) — rarely practical at scale | Academic tools, some commercial static analyzers |

The gap between branch and condition coverage is real and under-appreciated. Consider:

```
if (is_admin || (is_owner && !is_locked))
```

Branch coverage: 2 test cases (whole expression true, whole expression false). Condition coverage: needs each of `is_admin`, `is_owner`, `is_locked` independently true and false. MC/DC: needs each condition proven to independently flip the outcome. These are progressively stricter — and the stricter versions catch real bugs in authorization logic that branch coverage misses entirely.

### Tier 3 — Effectiveness coverage (expensive, high-signal)

| Metric | What it measures | When it matters | Tool support |
|---|---|---|---|
| **Mutation coverage** | % of deliberately injected bugs (mutants) killed by the test suite | Always — but cost-prohibitive for large codebases run frequently | PIT (Java), Stryker (JS/TS/.NET), mutmut/cosmic-ray (Python), cargo-mutants (Rust) |

This is the most important metric the framework omits. Mutation coverage measures test **quality**, not test **execution**. A test suite can achieve 100% line, 100% branch, 100% function coverage while asserting nothing meaningful — every line is executed but no test would fail if you changed a `<` to `<=` or deleted a conditional. Mutation coverage catches this: it changes the code and checks whether tests break. If they don't, the "surviving mutant" reveals a test gap that no execution-based metric would find.

The reason it's not in Tier 1: running a mutation suite on a large codebase can take orders of magnitude longer than the test suite itself (every mutant requires a test run). It's typically sampled or run on changed code only.

### Tier 4 — Semantic coverage (requires additional infrastructure)

| Metric | What it measures | When it matters | Infrastructure required |
|---|---|---|---|
| **Requirements coverage** | % of specified requirements traced to at least one test | Regulated environments (medical devices, aerospace, finance) | Requirements management system with traceability |
| **State coverage** | % of states and state transitions exercised | Protocol implementations, UI flows, state machines | State model definition |
| **Data flow coverage** | Definition-use pairs of variables exercised | Catches use-before-define, define-never-use | Specialized static analysis (mostly academic tools) |
| **Error path coverage** | % of error/exception paths specifically exercised | Always relevant, rarely measured as a distinct metric | No standard tooling separates this from line coverage |
| **API surface coverage** | % of public API endpoints/methods exercised | Libraries, services | Test harness that tracks API entry points |

### Assessment

| Question | Answer |
|---|---|
| Are line/branch/function **wrong**? | No — they're the right starting point |
| Are they **sufficient**? | For many projects, yes. For safety-critical, security-critical, or high-reliability systems, no |
| Are they **universal**? | Tool support is near-universal. The metrics themselves are the lowest tier of a richer taxonomy |
| What's the biggest gap? | Mutation coverage — it's the only metric that measures test effectiveness rather than test execution, and its tooling is now available for most major languages |
| What's the second biggest gap? | Condition/MC/DC — matters when boolean logic governs authorization, safety, or financial decisions |

### Implications for the framework

The current framing — "Three dimensions: line coverage, branch coverage, and function coverage" — presents Tier 1 as exhaustive. A more honest framing would be:

- **Tier 1 (line, branch, function)** is the universal baseline — report these always
- **Tier 3 (mutation)** is the quality check on Tier 1 — adopt when the question shifts from "is the code executed?" to "are the tests actually catching bugs?"
- **Tier 2 (condition, MC/DC)** is domain-gated — adopt when regulatory or safety requirements mandate it
- **Tier 4 (requirements, state, data flow, error path)** is infrastructure-gated — adopt when the supporting systems exist

This would also make Test Coverage the first category to benefit from the maturity model more richly — maturity isn't just "are tests in CI?" but also "what tier of coverage metrics is the project tracking?"
