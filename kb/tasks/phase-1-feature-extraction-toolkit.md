# Phase 1: Feature Extraction Toolkit

## Context

This is Phase 1 of the incremental evolution model (Option E → C) for adding language support to Biome. The immediate need is extracting features from 13 cloned YAML tool repositories. No existing agents support research or feature extraction. YAML is the first language; naming is language-agnostic so agents/commands can be reused for shell, toml, ruby, rust, etc.

## Files to Create

### 1. `.claude/agents/lang-feature-extractor.md`

New research agent. Based on feature-dev's `code-explorer` + cookbook's `research-lead` pattern.

- **name:** `lang-feature-extractor`
- **color:** `cyan` (existing agents use purple/green/red)
- **model:** `sonnet`
- **tools:** `Glob, Grep, LS, Read, WebFetch, WebSearch, TodoWrite, Task` — read-only plus Task for spawning parallel subagents per repo
- **description:** 3 example blocks covering linter comparison, formatter comparison, and validator comparison
- **system prompt (~2200 chars):** Role as research analyst. 5-step process: Inventory → Triage → Extract → Synthesize → Report. Output format: tool summaries, feature matrix, consensus features, unique features, observations. Uses Task subagents for 4+ repos.

**Design rationale:**

- **Tools include `Task`** so the agent can spawn parallel subagents per repo when extracting from 4+ repos simultaneously. The research-lead + subagent pattern from the agent patterns notebooks was identified as the highest-value pattern to apply first (see `kb/tasks/agent-leverage-options.md`).
- **Color is `cyan`** because existing agents use purple (cst-parser-engineer), green (biome-lint-engineer), and red (ir-formatter-engineer). The agent-development skill docs suggest cyan maps to analysis/review tasks. Yellow is taken by `code-explorer` in the reference patterns.
- **Model is `sonnet`** matching the `code-explorer` reference pattern. This agent does breadth-first research, not deep reasoning requiring opus.
- **Read-only tools** follow least-privilege. No Write, Edit, or Bash — this is a research agent.

### 2. `.claude/commands/lang-research.md`

New slash command for `/lang-research <language> [focus-area]`.

- **argument-hint:** `<language> [focus-area]`
- **5-phase workflow:**
  1. **Setup** — Load `references/$1/tools.md`, create todo list. If the tool inventory doesn't exist, stop and inform the user.
  2. **Clarifying Questions** — Scope confirmation (scan all repos or a subset?), depth preference (broad survey or deep extraction?), priority signals (any known feature priorities for Biome?). Wait for answers before proceeding.
  3. **Feature Extraction** — Launch lang-feature-extractor agent(s) via Task. For 1-3 repos, single agent. For 4+ repos, multiple agents in parallel grouped by tool type (all linters together, all formatters together, etc.).
  4. **Synthesis** — Combine all agent outputs into a unified report: executive summary, feature matrices by category, consensus features (in 2+ tools), unique features (in 1 tool only), architectural observations, recommended next steps.
  5. **Completion** — Present report, offer drill-down into specific areas, suggest saving to `references/$1/`.

**Design rationale:**

- **5 phases rather than 7** because this covers only research (phases 1-4 of feature-dev's lifecycle), not implementation or review.
- **Clarifying Questions phase is explicit** because the feature-dev command pattern marks it as critical ("DO NOT SKIP").
- **`$1` and `$2` positional args** follow the command-development docs' pattern. `$1` for language, `$2` for optional focus area. This makes the command language-agnostic — `/lang-research yaml` reads `references/yaml/tools.md`, `/lang-research shell` would read `references/shell/tools.md`.
- **`@references/$1/tools.md`** uses the `@filepath` syntax to include file contents directly into the command context. The `$1` parameterization is exactly the pattern called out in the evolution model.

### 3. `references/yaml/tools.md`

Tool inventory for the 13 cloned YAML repos, grouped by type. Each entry includes: path, type, language, specific feature file locations, and notes.

**Linters (3):**

| Repo | Language | Path | Feature Locations | Notes |
|------|----------|------|-------------------|-------|
| yamllint | Python | ~/Clones/adrienverge/yamllint/ | `yamllint/rules/` (25 rule files), `yamllint/conf/`, `yamllint/cli.py` | Most mature YAML linter. Self-contained rule files with enable/disable, severity, and per-rule options. |
| yaml-lint-rs | Rust | ~/Clones/hiromaily/yaml-lint-rs/ | `core/src/rules/` (11 rule files), `core/src/linter.rs`, `core/src/config.rs` | Rust port of yamllint. Subset of rules. Useful for Rust-based linting patterns. |
| yamllint-rs | Rust | ~/Clones/AvnerCohen/yamllint-rs/ | `src/rules/` (25 rule files), `src/rules/factory.rs`, `src/rules/registry.rs`, `src/config.rs` | More complete Rust port. Factory/registry patterns for rule management. |

**Formatters (3):**

| Repo | Language | Path | Feature Locations | Notes |
|------|----------|------|-------------------|-------|
| yamlfmt | Go | ~/Clones/google/yamlfmt/ | `engine.go`, `formatter.go`, `feature.go`, `formatters/`, `internal/features/`, `cmd/yamlfmt/` | Google's YAML formatter. Modular engine/formatter architecture. Feature flag system. |
| prettier | JavaScript | ~/Clones/prettier/prettier/ | `src/language-yaml/` (parser-yaml.js, printer-yaml.js, print/, options.js) | Multi-language formatter. IR-based formatting (doc builders). |
| yamlfix | Python | ~/Clones/lyz-code/yamlfix/ | `src/yamlfix/` (config.py, adapters.py, services.py, model.py) | Opinionated formatter/fixer. Configuration-driven. Comment-preserving. |

**Parsers (3):**

| Repo | Language | Path | Feature Locations | Notes |
|------|----------|------|-------------------|-------|
| yaml-rust2 | Rust | ~/Clones/Ethiraric/yaml-rust2/ | `src/` (parser.rs, scanner.rs, emitter.rs, yaml.rs) | Maintained fork of yaml-rust. Event-based parser. Key scanner/parser architecture reference. |
| saphyr | Rust | ~/Clones/saphyr-rs/saphyr/ | `saphyr/src/` (loader.rs, emitter.rs, annotated.rs), `parser/src/` (parser.rs, scanner.rs) | Modern Rust YAML library. Two-crate workspace. Annotated node support. Active development. |
| serde-yaml | Rust | ~/Clones/dtolnay/serde-yaml/ | `src/` (de.rs, ser.rs, loader.rs, mapping.rs, value/, error.rs) | UNMAINTAINED (archived). Serde integration. Useful for error reporting patterns only. |

**Validators (3):**

| Repo | Language | Path | Feature Locations | Notes |
|------|----------|------|-------------------|-------|
| kubeconform | Go | ~/Clones/yannh/kubeconform/ | `pkg/validator/`, `pkg/registry/`, `pkg/resource/`, `pkg/cache/`, `pkg/output/` | Kubernetes YAML validator. Schema registry pattern (local + remote). |
| yaml-validator | Rust | ~/Clones/MathiasPius/yaml-validator/ | `yaml-validator/src/` (lib.rs, modifiers/, types/, errors/, breadcrumb.rs) | Schema-based validator. Modifier pattern. Breadcrumb-based error path reporting. |
| action-validator | Rust | ~/Clones/mpalmer/action-validator/ | `src/` (lib.rs, schemas.rs, schemastore/, config.rs, validation_error.rs) | GitHub Actions validator. Embedded schema approach. |

**Language Server (1):**

| Repo | Language | Path | Feature Locations | Notes |
|------|----------|------|-------------------|-------|
| vscode-yaml | TypeScript | ~/Clones/redhat-developer/vscode-yaml/ | `src/` (extension.ts, schema-extension-api.ts, json-schema-cache.ts) | Red Hat's YAML language server. JSON Schema validation integration. Schema extension API. |

## Directories to Create

- `.claude/commands/` (does not exist yet)
- `references/yaml/` (does not exist yet)

## Verification

After creating the files:
1. Run `/lang-research yaml` to verify the command loads and reads `references/yaml/tools.md`
2. Confirm the agent triggers when asked to "extract features from YAML linter repos"
3. Verify the tools.md paths are correct by spot-checking a few repo paths
