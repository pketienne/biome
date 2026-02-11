# Quality Audit Framework — Language Reference: Ruby

Language-specific tooling, idioms, and patterns for applying the framework to Ruby codebases. References point to the final framework at `quality-audit-framework.md`.

---

## Test Coverage — [§1](quality-audit-framework.md#1-test-coverage)

### Tier 1 tooling (execution coverage)

| Metric | Tool | Notes |
|---|---|---|
| Line coverage | `simplecov` | Standard Ruby coverage gem |
| Branch coverage | `simplecov` with branch mode | Requires explicit configuration |
| Function/method coverage | `simplecov` | Reported alongside line coverage |

---

## Code Complexity — [§2](quality-audit-framework.md#2-code-complexity)

### Tooling

| Tool | Metrics | Notes |
|---|---|---|
| `flog` | Method complexity scores | Higher score = more complex; scores >20 indicate refactoring candidates |
| `rubocop-metrics` | Method length, ABC size | Integrated into RuboCop linting workflow |

### Thresholds (Ruby-specific)

| Metric | Low (Good) | Medium | High (Refactor) |
|---|---|---|---|
| Method complexity (flog) | <10 | 10-20 | >20 |

---

## Error Handling — [§4](quality-audit-framework.md#4-error-handling)

### Idioms and patterns

| Pattern | Ruby form | Severity | Framework dimension |
|---|---|---|---|
| Bare rescue (swallowed errors) | `rescue => e` with no re-raise or logging | High | Error propagation |
| Uncontextualized raise | `raise "something went wrong"` (string error) | Medium | Error typing |
| Typed errors | Custom exception classes inheriting from `StandardError` | Target state | Error typing |

---

## Dependencies — [§6](quality-audit-framework.md#6-dependencies)

### Tooling

| Dimension | Tool | Notes |
|---|---|---|
| Vulnerability scanning | `bundler-audit` | Checks against Ruby advisory database |
| License compliance | `license_finder` | Audits gem licenses |
| Freshness | `bundle outdated` | Lists gems behind latest version |
| Dependency tree inspection | `bundle viz` | Visualizes gem dependency graph |

---

## Security — [§7](quality-audit-framework.md#7-security)

### Tooling

| Dimension | Tool | Notes |
|---|---|---|
| Dependency vulnerabilities | `bundler-audit` | Also listed under Dependencies — same tool serves both categories |
| Secret detection | `gitleaks`, `trufflehog` | Language-agnostic |

---

## Documentation — [§8](quality-audit-framework.md#8-documentation)

### Tooling and conventions

| Aspect | Ruby form |
|---|---|
| Doc coverage | YARD statistics |
| Doc generation | YARD (`yardoc`) |
| Doc format | YARD tags (`@param`, `@return`, `@example`) |

---

## Project Standards — [§10](quality-audit-framework.md#10-project-standards)

### Naming conventions

| Target | Convention |
|---|---|
| Methods | `snake_case` |
| Classes/Modules | `PascalCase` |
| Constants | `SCREAMING_SNAKE_CASE` |
| Files | `snake_case.rb` or `kebab-case.rb` (varies by project) |

### Linting

| Tool | What it checks |
|---|---|
| `rubocop` | Naming conventions, style, complexity metrics, common mistakes |
