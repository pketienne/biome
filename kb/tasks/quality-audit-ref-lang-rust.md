# Quality Audit Framework — Language Reference: Rust

Language-specific tooling, idioms, and patterns for applying the framework to Rust codebases. References point to the final framework at `quality-audit-framework.md`.

---

## Test Coverage — [§1](quality-audit-framework.md#1-test-coverage)

### Tier 1 tooling (execution coverage)

| Metric | Tool | Notes |
|---|---|---|
| Line coverage | `cargo llvm-cov`, `tarpaulin` | `llvm-cov` uses LLVM instrumentation; `tarpaulin` is cargo-native |
| Branch coverage | `cargo llvm-cov --branch` | Requires explicit flag |
| Function coverage | Same tools, function-level reports | Reported alongside line coverage |

### Tier 3 tooling (mutation coverage)

| Tool | Notes |
|---|---|
| `cargo-mutants` | Mutation testing for Rust; injects mutations and checks if tests catch them |

---

## Code Complexity — [§2](quality-audit-framework.md#2-code-complexity)

### Tooling

| Tool | Metrics | Notes |
|---|---|---|
| `rust-code-analysis-cli` (Mozilla) | Cyclomatic, cognitive, SLOC | Multi-metric analysis |
| `cargo clippy -W clippy::cognitive_complexity` | Cognitive complexity warnings | Integrated into standard linting workflow |

---

## Code Duplication — [§3](quality-audit-framework.md#3-code-duplication)

### Language-specific patterns

| Pattern | Rust example | Consolidation strategy |
|---|---|---|
| Error message construction | `format!("Cannot find {}", x)` repeated across modules | Localized message function or error enum with Display impl |
| Hardcoded string output | `bail!("English text")`, `println!("English text")` | Route through localization system |

---

## Error Handling — [§4](quality-audit-framework.md#4-error-handling)

### Idioms and patterns

| Pattern | Rust form | Severity | Framework dimension |
|---|---|---|---|
| Crash/abort (unrecoverable) | `.unwrap()`, `.expect()` without context | High | Crash surface |
| Structured propagation | `?` operator with typed errors | Target state | Error propagation |
| String errors | `String` or `&str` as error type | Medium | Error typing |
| Typed errors | `enum` with variants per error case | Target state | Error typing |
| Context annotation | `.context("while doing X")?` (anyhow/eyre) | Target state | Error context |
| Tracing integration | `#[instrument]`, `tracing::error!()` | Target state | Error logging |

### Maturity progression (Rust-specific)

| Level | Indicators |
|---|---|
| 0 — Ad hoc | `.unwrap()` everywhere, `println!` for errors, string errors |
| 1 — Basic | Error enums exist, `?` propagation, some `tracing` |
| 2 — Structured | `thiserror`/`anyhow` with context, structured logging via `tracing` |
| 3 — Observable | `tracing` spans on all public functions, error categorization, metrics export |

---

## Dependencies — [§6](quality-audit-framework.md#6-dependencies)

### Tooling

| Dimension | Tool | Notes |
|---|---|---|
| Vulnerability scanning | `cargo audit` | Checks against RustSec advisory database |
| License/policy compliance | `cargo deny` | Checks licenses, sources, disallowed crates |
| Freshness | `cargo outdated` | Lists deps behind latest version |
| Unused dependencies | `cargo udeps` | Requires nightly; detects unused deps |
| Dependency tree inspection | `cargo tree` | Visualizes transitive dependency graph |

---

## Security — [§7](quality-audit-framework.md#7-security)

### Tooling

| Dimension | Tool | Notes |
|---|---|---|
| Dependency vulnerabilities | `cargo audit` | Also listed under Dependencies — same tool serves both categories |
| Dependency policy | `cargo deny` | License and source restrictions |
| Secret detection | `gitleaks`, `trufflehog` | Language-agnostic |

---

## Documentation — [§8](quality-audit-framework.md#8-documentation)

### Tooling and conventions

| Aspect | Rust form |
|---|---|
| Doc coverage | `rustdoc` statistics |
| Module-level docs | `//!` doc comments at top of module |
| Item-level docs | `///` doc comments on public items |
| Doc tests | Code blocks in doc comments are compiled and run during `cargo test` |

---

## Localization — [§9](quality-audit-framework.md#9-localization)

### Detection patterns

| Pattern | What to search for | Severity |
|---|---|---|
| Hardcoded user-facing strings | `bail!("English")`, `println!("English")`, `eprintln!("English")` | Medium |
| Hardcoded format strings | `format!("Cannot find {}", x)` without i18n wrapper | Medium |

---

## Project Standards — [§10](quality-audit-framework.md#10-project-standards)

### Naming conventions

| Target | Convention |
|---|---|
| Functions/methods | `snake_case` |
| Types/structs/enums | `PascalCase` |
| Constants | `SCREAMING_SNAKE_CASE` |
| Files/modules | `snake_case.rs` |
| Crates | `snake_case` (with hyphens in Cargo.toml, underscores in code) |

### Linting

| Tool | What it checks |
|---|---|
| `cargo clippy` | Naming conventions, idiomatic patterns, complexity warnings, common mistakes |
| `rustfmt` | Formatting (not naming, but part of Project Standards §10 checks) |
