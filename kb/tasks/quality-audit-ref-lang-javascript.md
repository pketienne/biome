# Quality Audit Framework — Language Reference: JavaScript

Language-specific tooling, idioms, and patterns for applying the framework to JavaScript/TypeScript codebases. References point to the final framework at `quality-audit-framework.md`.

---

## Test Coverage — [§1](quality-audit-framework.md#1-test-coverage)

### Tier 1 tooling (execution coverage)

| Metric | Tool | Notes |
|---|---|---|
| Line coverage | `istanbul`, `c8` | `c8` uses V8's built-in coverage; `istanbul`/`nyc` instruments code |
| Branch coverage | Same tools | Reported by default in most configurations |
| Function coverage | Same tools | Reported alongside line coverage |

### Tier 3 tooling (mutation coverage)

| Tool | Notes |
|---|---|
| Stryker | Mutation testing for JS/TS (also supports .NET); injects mutations and checks if tests catch them |

---

## Code Complexity — [§2](quality-audit-framework.md#2-code-complexity)

### Tooling

| Tool | Metrics | Notes |
|---|---|---|
| `eslint-plugin-complexity` | Cyclomatic complexity | Integrated into ESLint workflow; configurable threshold |

---

## Code Duplication — [§3](quality-audit-framework.md#3-code-duplication)

### Tooling

| Tool | Detection type | Notes |
|---|---|---|
| `jscpd` | Token-based clone detection | Multi-language support; configurable min-lines threshold |

---

## Dependencies — [§6](quality-audit-framework.md#6-dependencies)

### Tooling

| Dimension | Tool | Notes |
|---|---|---|
| Vulnerability scanning | `npm audit` | Built into npm; checks against npm advisory database |
| Freshness | `npm outdated` | Lists packages behind latest version |
| Unused dependencies | `depcheck` | Detects unused and missing dependencies |

---

## Security — [§7](quality-audit-framework.md#7-security)

### Tooling

| Dimension | Tool | Notes |
|---|---|---|
| Dependency vulnerabilities | `npm audit` | Also listed under Dependencies — same tool serves both categories |
| Secret detection | `gitleaks`, `trufflehog` | Language-agnostic |

---

## Documentation — [§8](quality-audit-framework.md#8-documentation)

### Tooling and conventions

| Aspect | JavaScript form |
|---|---|
| Doc generation | JSDoc |
| Doc format | JSDoc tags (`@param`, `@returns`, `@example`) |

---

## Project Standards — [§10](quality-audit-framework.md#10-project-standards)

### Naming conventions

| Target | Convention |
|---|---|
| Functions/variables | `camelCase` |
| Classes | `PascalCase` |
| Constants | `SCREAMING_SNAKE_CASE` or `camelCase` (varies by project) |
| Files | `camelCase.js`, `kebab-case.js`, or `PascalCase.js` (varies by framework convention) |

### Linting

| Tool | What it checks |
|---|---|
| ESLint | Naming conventions, complexity, style, common mistakes |
| Biome | Linting, formatting, import sorting (Rust-based, fast) |
