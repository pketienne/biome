# Task: Rust-Based Trunk Replacement

**Date:** 2026-02-03

**Related:** [Trunk Mise TMPDIR Security Issue](2026-02-03-trunk-mise-tmpdir.md)

## Background

This task emerged from investigating the [trunk + mise trust security issue](2026-02-03-trunk-mise-tmpdir.md). Key findings from that investigation:

1. Trunk hardcodes sandbox directories to `/tmp/trunk-<UID>/`
2. Trunk ignores `TMPDIR` environment variable
3. Trunk CLI is closed source - cannot be forked/fixed
4. Trusting `/tmp` paths weakens mise's security model

Rather than work around trunk's limitations, building a replacement that respects security best practices is a viable alternative.

## Project Name: tRust

**tRust** - a linter orchestrator you can trust.

The name reflects:

- Written in **Rust**
- Born from mise **trust** issues
- Addresses the `/tmp` security anti-pattern
- Trustworthy by design (configurable temp directories, transparent operation)

## Goals

1. Replace trunk for local linter orchestration
2. Respect `TMPDIR` and XDG Base Directory Specification
3. Support the 20 linters currently enabled in symmetra
4. Provide git integration for changed-file detection
5. Parallel execution for performance
6. Simple, transparent configuration

## Current Trunk Usage (Scope)

**20 enabled linters to support:**

| Category | Linters                          |
| -------- | -------------------------------- |
| Ruby     | cookstyle, rubocop               |
| Python   | bandit, black, isort, ruff       |
| Rust     | clippy, rustfmt                  |
| Markdown | dprint, markdownlint             |
| Shell    | shellcheck, shfmt                |
| YAML     | yamllint                         |
| JSON     | biome                            |
| TOML     | taplo                            |
| Security | checkov, osv-scanner, trufflehog |
| Other    | git-diff-check, serdi            |

**Not currently using:** Git hook integration (trunk runs manually)

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                           tRust                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ Config       │    │ File         │    │ Git          │       │
│  │ (TOML)       │───▶│ Classifier   │◀───│ Integration  │       │
│  └──────────────┘    └──────┬───────┘    └──────────────┘       │
│                             │                                    │
│                             ▼                                    │
│                    ┌──────────────────┐                         │
│                    │ Linter Registry  │                         │
│                    │ - command        │                         │
│                    │ - args           │                         │
│                    │ - output_format  │                         │
│                    │ - file_patterns  │                         │
│                    └────────┬─────────┘                         │
│                             │                                    │
│                             ▼                                    │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ Executor     │───▶│ Output       │───▶│ Reporter     │       │
│  │ (parallel)   │    │ Parser       │    │ (terminal)   │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Configuration Design

```toml
# trust.toml

[settings]
tmpdir = "${XDG_CACHE_HOME}/trust"  # Respects TMPDIR env var
parallel = true
cache = true

[[linter]]
name = "rubocop"
command = "rubocop"
args = ["--format", "json", "{files}"]
patterns = ["*.rb"]
output_format = "json"
parser = "rubocop"

[[linter]]
name = "shellcheck"
command = "shellcheck"
args = ["--format", "json", "{files}"]
patterns = ["*.sh", "*.bash"]
output_format = "json"
parser = "shellcheck"

[[linter]]
name = "markdownlint"
command = "markdownlint"
args = ["--json", "{files}"]
patterns = ["*.md"]
output_format = "json"
parser = "markdownlint"
```

## Implementation Outline

### Core Types

```rust
// src/config.rs
#[derive(Debug, Deserialize)]
struct Config {
    settings: Settings,
    #[serde(rename = "linter")]
    linters: Vec<LinterConfig>,
}

#[derive(Debug, Deserialize)]
struct Settings {
    tmpdir: Option<PathBuf>,
    parallel: bool,
    cache: bool,
}

#[derive(Debug, Deserialize)]
struct LinterConfig {
    name: String,
    command: String,
    args: Vec<String>,
    patterns: Vec<String>,
    output_format: OutputFormat,
    parser: String,
}
```

```rust
// src/result.rs
#[derive(Debug)]
struct LintResult {
    file: PathBuf,
    line: usize,
    column: usize,
    severity: Severity,
    message: String,
    linter: String,
    code: Option<String>,
}

#[derive(Debug)]
enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}
```

### Main Flow

```rust
// src/main.rs
async fn run(config: Config) -> Result<Vec<LintResult>> {
    let files = get_changed_files()?;  // or all files
    let grouped = group_files_by_linter(&config.linters, &files);

    let results: Vec<LintResult> = futures::future::join_all(
        grouped.into_iter().map(|(linter, files)| {
            run_linter(linter, files)
        })
    ).await.into_iter().flatten().collect();

    Ok(results)
}
```

## Dependencies

```toml
# Cargo.toml
[package]
name = "trust"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
glob = "0.3"
clap = { version = "4", features = ["derive"] }
colored = "2"
serde_json = "1"
dirs = "5"
```

## Feature Comparison

### What tRust Provides

| Feature          | Description                        |
| ---------------- | ---------------------------------- |
| TMPDIR control   | Configurable, respects environment |
| No mise conflict | No `.mise.toml` in sandbox         |
| Transparency     | Open source, you control the code  |
| Simplicity       | Only features you need             |
| Speed            | Rust + parallel execution          |
| Security         | No `/tmp` anti-pattern             |

### What tRust Won't Have (Initially)

| Feature          | Workaround                          |
| ---------------- | ----------------------------------- |
| Auto-updates     | mise manages linter versions        |
| Linter discovery | Define linters explicitly in config |
| SARIF upload     | Add if needed later                 |
| IDE integration  | LSP server could be added           |
| Daemon mode      | Not needed for manual runs          |

## Implementation Phases

### Phase 1: MVP (Core Functionality)

- [ ] Config file parsing (TOML)
- [ ] File pattern matching (glob)
- [ ] Linter execution (single-threaded first)
- [ ] Basic output parsing (JSON for common linters)
- [ ] Terminal output formatting
- [ ] Support 3-4 linters: rubocop, shellcheck, markdownlint, yamllint

### Phase 2: Git Integration

- [ ] Detect changed files via `git diff`
- [ ] Support `--all` flag for all files
- [ ] Upstream branch detection

### Phase 3: Parallel Execution

- [ ] Tokio-based async execution
- [ ] Configurable parallelism
- [ ] Progress indicator

### Phase 4: Caching

- [ ] File content hashing
- [ ] Cache storage in `$XDG_CACHE_HOME/trust/`
- [ ] Cache invalidation

### Phase 5: Full Linter Support

- [ ] Add remaining 16 linters
- [ ] Linter-specific output parsers
- [ ] Auto-fix support (`--fix` flag)

### Phase 6: Polish

- [ ] Git hook integration
- [ ] Init command (generate config)
- [ ] Shell completions
- [ ] Man page

## Complexity Assessment

| Component           | Complexity | Estimate |
| ------------------- | ---------- | -------- |
| Config parsing      | Low        | 1-2 days |
| File classification | Low        | 1 day    |
| Linter execution    | Low        | 1-2 days |
| Output parsing      | Medium     | 3-5 days |
| Git integration     | Low        | 1 day    |
| Parallel execution  | Medium     | 2-3 days |
| Caching             | Medium     | 2-3 days |
| Terminal output     | Low        | 1-2 days |

**Total MVP estimate:** 2-4 weeks

## Prior Art

Reference implementations to study:

- [lintrunner](https://github.com/pytorch/test-infra/tree/main/tools/lintrunner) - PyTorch's linter runner (Python)
- [mega-linter](https://github.com/oxsecurity/megalinter) - Multi-linter orchestrator
- [pre-commit](https://github.com/pre-commit/pre-commit) - Python, good config design
- [lefthook](https://github.com/evilmartians/lefthook) - Go, fast git hooks manager

## Security Design Principles

1. **Configurable TMPDIR**: Default to `$XDG_CACHE_HOME/trust/`, respect `TMPDIR` env var
2. **No magic config files**: Don't create `.mise.toml` or similar in temp directories
3. **Transparent execution**: Log exact commands being run with `--verbose`
4. **No network by default**: Only linters make network calls, not the orchestrator
5. **Minimal permissions**: Don't require root, don't modify system files

## Open Questions

1. **Output format**: SARIF for IDE integration, or simpler custom format?
2. **Config location**: `.trust.toml` in project root, or `trust.toml` in `.config/`?
3. **Linter versioning**: Should tRust manage linter versions, or delegate to mise?
4. **Remote config**: Support for shared linter configs across projects?

## References

- [Trunk Mise TMPDIR Security Issue](2026-02-03-trunk-mise-tmpdir.md) - The problem that motivated this project
- [mise Documentation](https://mise.jdx.dev/)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [SARIF Specification](https://sarifweb.azurewebsites.net/)
