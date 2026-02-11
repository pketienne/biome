# Biome Contribution Specification

**Purpose:** Specification document extracting all instructions from Biome's CONTRIBUTING.md and its linked internal documents, categorized as mandatory or optional, with utility assessment for the YAML implementation workflow.

**Sources analyzed:**
1. Root `CONTRIBUTING.md` (572 lines)
2. `crates/biome_analyze/CONTRIBUTING.md` (1625 lines)
3. `crates/biome_formatter/CONTRIBUTING.md` (337 lines)
4. `crates/biome_parser/CONTRIBUTING.md` (415 lines)
5. `crates/biome_diagnostics/CONTRIBUTING.md` (141 lines)
6. `crates/biome_service/CONTRIBUTING.md` (46 lines)
7. `crates/biome_js_formatter/CONTRIBUTING.md` (63 lines)
8. `.github/workflows/pull_request.yml` (122 lines)
9. `.github/workflows/main.yml` (130 lines)
10. `.github/workflows/pull_request_title_lint.yaml` (59 lines)
11. `.github/PULL_REQUEST_TEMPLATE.md` (31 lines)
12. `Cargo.toml` — workspace lints section (lines 174-258)
13. `clippy.toml` (6 lines)
14. `rustfmt.toml` (3 lines)
15. `.editorconfig` (17 lines)
16. `.coderabbit.yaml` (49 lines)
17. `GOVERNANCE.md` (360 lines)
18. `CODE_OF_CONDUCT.md`

---

## 1. Environment & Tooling Setup

### 1.1 Install Rust stable toolchain
- **Source:** Root CONTRIBUTING.md > Getting Started > Local development
- **Classification:** MANDATORY
- **Notes:** Prerequisite for all compilation. Already satisfied in devcontainer.

### 1.2 Install `just` via OS package manager
- **Source:** Root CONTRIBUTING.md > Install the required tools
- **Classification:** OPTIONAL (but HIGH utility)
- **Assessment:** `just` wraps many multi-step operations (`just ready`, `just gen-formatter`, `just gen-analyzer`, `just test-lintrule`). The lightweight devcontainer does NOT include `just`. **Alternative:** Run the underlying `cargo` commands directly. This works but requires knowing what each `just` recipe expands to. For a second-language implementation, installing `just` in the container is strongly recommended.

### 1.3 Run `just install-tools`
- **Source:** Root CONTRIBUTING.md > Install the required tools
- **Classification:** OPTIONAL (conditional on task scope)
- **Assessment:** Installs `cargo-binstall`, `cargo-insta`, `tombi`, `wasm-bindgen-cli`, `wasm-opt`. Of these:
  - `cargo-insta`: **HIGH utility** — required for snapshot test management (`cargo insta accept/reject/review`)
  - `tombi`: LOW utility for YAML work (formats TOML files)
  - `wasm-bindgen-cli` / `wasm-opt`: NOT needed for YAML (WASM build targets)
  - `cargo-binstall`: LOW utility (convenience installer)
- **Alternative:** Install only `cargo-insta` directly: `cargo install cargo-insta`

### 1.4 Install `pnpm` and run `pnpm install`
- **Source:** Root CONTRIBUTING.md > Install the required tools
- **Classification:** OPTIONAL (only for changesets/Node.js work)
- **Assessment:** Required for `just new-changeset` and Node.js API development. NOT needed during implementation phases. Only needed at PR contribution time.

---

## 2. Development Workflow

### 2.1 Use `cargo biome-cli-dev` for development testing
- **Source:** Root CONTRIBUTING.md > Local development
- **Classification:** MANDATORY (for end-to-end verification)
- **Notes:** Equivalent to running the `biome` CLI in dev mode. Used for `format`, `lint`, `check` commands against test files.

### 2.2 Run `just f` (format) before committing
- **Source:** Root CONTRIBUTING.md > Checks; Analyzer CONTRIBUTING.md > Creating and Implementing the Rule
- **Classification:** MANDATORY (CI enforces)
- **Notes:** Formats Rust and TOML files. **Alternative without `just`:** `cargo fmt` (for Rust) + `tombi fmt Cargo.toml` (for TOML). The Rust formatting is the critical part.

### 2.3 Run `just l` (lint) before committing
- **Source:** Root CONTRIBUTING.md > Checks
- **Classification:** MANDATORY (CI enforces)
- **Notes:** Runs Clippy and other linters. **Alternative without `just`:** `cargo clippy --workspace`

### 2.4 Run appropriate codegen commands
- **Source:** Root CONTRIBUTING.md > Checks
- **Classification:** MANDATORY (CI enforces, code must not be out of sync)
- **Notes:** Which codegen to run depends on what you changed:
  - `just gen-analyzer` — when working on linter rules
  - `just gen-formatter` — when working on formatter (`cargo run -p xtask_codegen -- formatter`)
  - `just gen-bindings` — when working on workspace
  - `just gen-grammar` — when working on parser grammar
  - `just ready` — runs ALL codegen (slow but comprehensive)
- **Alternative without `just`:** Run the underlying `cargo run -p xtask_codegen -- <subcommand>` directly.

### 2.5 Use `--profile debugging` for stack traces
- **Source:** Root CONTRIBUTING.md > Debugging
- **Classification:** OPTIONAL (HIGH utility when debugging panics)
- **Assessment:** The default dev profile strips debug info. Use `cargo t --profile debugging some_test` when you need stacktraces. **No alternative** — this is the only way to get debug symbols in Biome's workspace config.

### 2.6 Create debug binary for reproduction triage
- **Source:** Root CONTRIBUTING.md > Debug binaries
- **Classification:** OPTIONAL (for specific debugging scenarios)
- **Assessment:** `cargo build --bin biome` creates a debug binary. Useful for LSP debugging and reproduction triage. Not routinely needed during initial implementation.

### 2.7 Create production binary with `BIOME_VERSION`
- **Source:** Root CONTRIBUTING.md > Production binaries
- **Classification:** OPTIONAL (release process only)
- **Assessment:** `BIOME_VERSION=0.0.1 cargo build --bin biome --release`. Only relevant for release builds. NOT needed during development.

---

## 3. Testing

### 3.1 Use `cargo test` or `just test` to run tests
- **Source:** Root CONTRIBUTING.md > Testing
- **Classification:** MANDATORY
- **Notes:** `cargo t` from crate directory runs crate-specific tests. `cargo t quick_test` runs a single test. Use `--show-output` with `dbg!` macros.

### 3.2 Use `cargo insta` for snapshot testing
- **Source:** Root CONTRIBUTING.md > Testing; Analyzer CONTRIBUTING.md > Snapshot Tests
- **Classification:** MANDATORY (for analyzer and formatter tests)
- **Notes:** Three commands:
  - `cargo insta accept` — accept all snapshot changes
  - `cargo insta reject` — reject all changes
  - `cargo insta review` — review individually
- **Requires:** `cargo-insta` installed (see 1.3)

### 3.3 Analyzer: Use `quick_test` for rapid rule iteration
- **Source:** Analyzer CONTRIBUTING.md > Quick Test
- **Classification:** OPTIONAL (but VERY HIGH utility)
- **Assessment:** Modify `biome_*_analyze/tests/quick_test.rs`, set `SOURCE` and `rule_filter`, run `cargo t quick_test`. This is the fastest feedback loop for rule development. The YAML analyzer already has this pattern in `lib.rs` tests. **No alternative** that matches speed.

### 3.4 Analyzer: Snapshot tests in `tests/specs/` with correct group/rule pairing
- **Source:** Analyzer CONTRIBUTING.md > Snapshot Tests
- **Classification:** MANDATORY (for rule validation)
- **Notes:** Tests are rigid about `group/ruleName` directory structure. Files prefixed `invalid` contain code reported by the rule; files prefixed `valid` contain unreported code. **If placed in wrong group, no diagnostics appear** — this is a known gotcha.

### 3.5 Analyzer: `.jsonc` test files for script-mode snippets
- **Source:** Analyzer CONTRIBUTING.md > Snapshot Tests > `.jsonc` files
- **Classification:** OPTIONAL (JS-specific)
- **Assessment:** JSON arrays of code snippets interpreted in script environment. Not applicable to YAML rules.

### 3.6 Formatter: `spec_tests.rs` + `language.rs` + `spec_test.rs` test infrastructure
- **Source:** Formatter CONTRIBUTING.md > Testing
- **Classification:** MANDATORY (for formatter validation)
- **Notes:** Three files form the test harness:
  1. `tests/spec_tests.rs` — uses `gen_tests!` macro to auto-generate test functions from fixture files
  2. `tests/language.rs` — implements `TestFormatLanguage`
  3. `tests/spec_test.rs` — `run()` function connecting test files to formatter
- Test fixtures go in `tests/specs/<language>/`. Snapshots generated via `cargo insta`.

### 3.7 Formatter: Use `options.json` for non-default options in tests
- **Source:** Formatter CONTRIBUTING.md > Testing > Create and run tests
- **Classification:** OPTIONAL (only when testing non-default formatter options)
- **Assessment:** Place an `options.json` alongside test fixtures to override default options.

### 3.8 Use `just test-lintrule <ruleName>` for focused rule testing
- **Source:** Analyzer CONTRIBUTING.md > Run the Snapshot Tests
- **Classification:** OPTIONAL (HIGH utility, convenience wrapper)
- **Assessment:** **Alternative without `just`:** `cargo t -p biome_yaml_analyze <ruleName>` achieves similar effect.

---

## 4. Parser Development

### 4.1 All grammar nodes MUST start with language prefix
- **Source:** Parser CONTRIBUTING.md > Conventions
- **Classification:** MANDATORY
- **Notes:** e.g., `YamlBlockMapping`, not `BlockMapping`. Already followed in existing YAML grammar.

### 4.2 Union nodes MUST start with `Any*`
- **Source:** Parser CONTRIBUTING.md > Conventions
- **Classification:** MANDATORY
- **Notes:** e.g., `AnyYamlBlockNode`. Already followed.

### 4.3 Error-enclosing nodes MUST use `Bogus` word
- **Source:** Parser CONTRIBUTING.md > Conventions
- **Classification:** MANDATORY
- **Notes:** e.g., `YamlBogusValue`. Already followed.

### 4.4 Bogus nodes MUST be part of a union variant
- **Source:** Parser CONTRIBUTING.md > Conventions
- **Classification:** MANDATORY
- **Notes:** Every union must include a Bogus variant for error recovery.

### 4.5 List nodes MUST end with `List` postfix
- **Source:** Parser CONTRIBUTING.md > Conventions
- **Classification:** MANDATORY
- **Notes:** e.g., `YamlBlockMapEntryList`. Already followed.

### 4.6 Lists are NEVER optional (mandatory, empty by default)
- **Source:** Parser CONTRIBUTING.md > Conventions
- **Classification:** MANDATORY
- **Notes:** Grammar should declare lists as required fields, not optional.

### 4.7 Parse rules return `ParsedSyntax` and are named `parse_*`
- **Source:** Parser CONTRIBUTING.md > Authoring Parse Rules
- **Classification:** MANDATORY (convention)
- **Notes:** `Absent` if rule can't predict node from next token(s). `Present` if it consumes any tokens. Must NOT progress parser if returning `Absent`.

### 4.8 Lists MUST perform error recovery
- **Source:** Parser CONTRIBUTING.md > Parsing Lists & Error Recovery
- **Classification:** MANDATORY
- **Notes:** Use `ParseSeparatedList` or `ParseNodeList` with recovery token sets. Failure to recover causes infinite loops.

### 4.9 Preservation of valid tree structure over parent invalidation
- **Source:** Parser CONTRIBUTING.md > Parsing Lists & Error Recovery
- **Classification:** MANDATORY (design principle)
- **Notes:** Mark invalid parts as `bogus` rather than invalidating parent nodes. Minimizes information loss.

### 4.10 Run `just gen-grammar` after grammar changes
- **Source:** Parser CONTRIBUTING.md > Run the codegen
- **Classification:** MANDATORY
- **Notes:** Accepts optional language list: `just gen-grammar yaml`. **Alternative:** `cargo run -p xtask_codegen -- grammar yaml`

---

## 5. Formatter Development

### 5.1 Grammar/parser MUST be complete before starting formatter
- **Source:** Formatter CONTRIBUTING.md > Prerequisites
- **Classification:** MANDATORY (dependency chain)
- **Notes:** Formatter codegen depends on generated AST types from grammar. YAML grammar is complete (Layer 1-4 done).

### 5.2 Create formatter crate with `just new-crate`
- **Source:** Formatter CONTRIBUTING.md > Step 1
- **Classification:** OPTIONAL (convenience)
- **Assessment:** `just new-crate biome_yaml_formatter` creates boilerplate. **Alternative:** `cargo new --lib crates/biome_yaml_formatter` + manual Cargo.toml setup. Already done for YAML.

### 5.3 Run `just gen-formatter` to generate boilerplate
- **Source:** Formatter CONTRIBUTING.md > Step 2
- **Classification:** MANDATORY (generates `generated.rs` and module structure)
- **Notes:** **Alternative:** `cargo run -p xtask_codegen -- formatter`. Already done for YAML.

### 5.4 Create required core files manually
- **Source:** Formatter CONTRIBUTING.md > Step 3
- **Classification:** MANDATORY
- **Notes:** Must manually create: `lib.rs`, `context.rs`, `comments.rs`, `cst.rs`, `prelude.rs`, `verbatim.rs`. Codegen only creates `<language>/` directory and `generated.rs`.

### 5.5 Implement `CommentStyle` in `comments.rs`
- **Source:** Formatter CONTRIBUTING.md > Step 4 > CommentStyle
- **Classification:** MANDATORY
- **Notes:** YAML uses `#` line comments only (no block comments). Already implemented.

### 5.6 Implement `FormatContext` in `context.rs`
- **Source:** Formatter CONTRIBUTING.md > Step 4 > FormatContext
- **Classification:** MANDATORY
- **Notes:** `FormatOptions`, `FormatContext`, `CstFormatContext` traits. Already implemented.

### 5.7 Implement `FormatSyntaxNode` in `cst.rs`
- **Source:** Formatter CONTRIBUTING.md > Step 4 > FormatSyntaxNode
- **Classification:** MANDATORY
- **Notes:** Root dispatch via `map_syntax_node!`. Already implemented.

### 5.8 Implement `FormatLanguage` in `lib.rs`
- **Source:** Formatter CONTRIBUTING.md > Step 4 > FormatLanguage
- **Classification:** MANDATORY
- **Notes:** Ties everything together. Exposes `format_node()` public API. Already implemented.

### 5.9 Replace `format_verbatim_node` stubs with real implementations
- **Source:** Formatter CONTRIBUTING.md > Step 5
- **Classification:** MANDATORY (for production-quality formatting)
- **Notes:** Generated formatters initially use `format_verbatim_node`. Must replace with proper IR-based formatting for each node type. Partially done for YAML.

### 5.10 `.ungram` file is the source of truth
- **Source:** Formatter CONTRIBUTING.md > How the codegen works
- **Classification:** MANDATORY (design invariant)
- **Notes:** All generated types derive from the grammar file. Changes to node structure require grammar changes first, then re-codegen.

---

## 6. Analyzer / Lint Rule Development

### 6.1 Three rule types: Syntax, Lint, Assist
- **Source:** Analyzer CONTRIBUTING.md > top
- **Classification:** MANDATORY (conceptual framework)
- **Notes:** Syntax = language spec errors. Lint = static analysis. Assist = refactoring opportunities.

### 6.2 Choose good, cross-language rule names
- **Source:** Analyzer CONTRIBUTING.md > Understanding Biome Linter
- **Classification:** MANDATORY (naming has downstream implications for multi-language options)
- **Notes:** Generic names imply multi-language applicability. Specific names indicate language-specific rules.

### 6.3 Follow naming conventions (`no*`, `use*`, etc.)
- **Source:** Analyzer CONTRIBUTING.md > Naming Conventions for Rules
- **Classification:** MANDATORY
- **Notes:** Comprehensive list:
  - `no<Concept>` — forbid something
  - `use<Concept>` — mandate something
  - `noConstant*`, `noDuplicate*`, `noEmpty*`, `noExcessive*`, `noRedundant*`, `noUnused*`, `noUseless*`, `noInvalid*`, `useValid*`, `noUnknown*`, `noMisleading*`, `noRestricted*`, `noUndeclared*`, `noUnsafe*`, `useConsistent*`, `useShorthand*`

### 6.4 Rule diagnostic pillars: WHAT, WHY, HOW TO FIX
- **Source:** Analyzer CONTRIBUTING.md > What a Rule should say to the User
- **Classification:** MANDATORY
- **Notes:** Three required elements:
  1. Message: explain WHAT the error is
  2. Additional note: explain WHY it's triggered
  3. Code action or note: tell user WHAT TO DO

### 6.5 New rules MUST be in `nursery` group
- **Source:** Analyzer CONTRIBUTING.md > Placement of New Rules
- **Classification:** MANDATORY
- **Notes:** Nursery is exempt from semver. Rules promoted to permanent groups in minor/major releases.

### 6.6 `version` field MUST be `"next"`
- **Source:** Analyzer CONTRIBUTING.md > `declare_lint_rule!` macro
- **Classification:** MANDATORY
- **Notes:** Always use `"next"` — updated to actual version at release time.

### 6.7 Add `source` metadata for rules ported from other ecosystems
- **Source:** Analyzer CONTRIBUTING.md > Biome lint rules inspired by other lint rules
- **Classification:** OPTIONAL (when applicable)
- **Assessment:** Use `RuleSource::Eslint("rule-name").same()` or `.inspired()`. Not applicable to YAML-first rules like `noDuplicateKeys` (which is novel, not ported).

### 6.8 Choose appropriate severity (`error`/`warn`/`info`)
- **Source:** Analyzer CONTRIBUTING.md > Rule severity; Rule group and severity
- **Classification:** MANDATORY
- **Notes:** Group-severity constraints:
  - `correctness`/`security`/`a11y` → `error`
  - `style` → `info` or `warn`
  - `complexity` → `warn` or `info`
  - `suspicious` → `warn` or `error`
  - `performance` → `warn`
  - Actions → `info`

### 6.9 Rule options must live in `biome_rule_options` crate
- **Source:** Analyzer CONTRIBUTING.md > Rule Options
- **Classification:** MANDATORY (when rule has options)
- **Notes:** `just gen-analyzer` creates the option file. Options must implement `Deserializable`, `Merge`, `Serialize`, `Deserialize`, and `JsonSchema`.

### 6.10 Use `Box<[Box<str>]>` over `Vec<String>` for memory efficiency
- **Source:** Analyzer CONTRIBUTING.md > Rule Options
- **Classification:** OPTIONAL (best practice)
- **Assessment:** Saves one word per item (2 words vs 3). Meaningful at scale. Easy to adopt.

### 6.11 Prefer `Box<[Self::State]>` over `Vec<Self::State>` for Signals
- **Source:** Analyzer CONTRIBUTING.md > Multiple Signals
- **Classification:** OPTIONAL (best practice)
- **Assessment:** Same memory optimization as 6.10. Convert with `Vec::into_boxed_slice()`.

### 6.12 Code actions: implement `action` function with `fix_kind`
- **Source:** Analyzer CONTRIBUTING.md > Code Actions
- **Classification:** MANDATORY (when rule provides fixes)
- **Notes:** Must declare `fix_kind: FixKind::Safe` or `FixKind::Unsafe` in macro. Action uses `ctx.action_category()` and `ctx.metadata().applicability()`.

### 6.13 Prefer functional combinators over deep nesting
- **Source:** Analyzer CONTRIBUTING.md > Avoidable Deep Indentation
- **Classification:** OPTIONAL (code quality best practice)
- **Assessment:** Use `.ok()?.and_then()` chains instead of nested `if let`. Reduces indentation, improves readability. HIGH utility.

### 6.14 Avoid unnecessary string allocations
- **Source:** Analyzer CONTRIBUTING.md > Avoidable String Allocations
- **Classification:** OPTIONAL (performance best practice)
- **Assessment:** Use `TokenText` or `&str` comparisons instead of `.to_string()`. Avoids heap allocations. HIGH utility.

### 6.15 Check if variables are global before banning
- **Source:** Analyzer CONTRIBUTING.md > Common Mistakes > Not checking if a variable is global
- **Classification:** MANDATORY (when writing rules that ban variables/functions)
- **Notes:** Must consult semantic model. Not applicable to YAML (no variable semantics).

### 6.16 Rule documentation: first paragraph MUST be single line
- **Source:** Analyzer CONTRIBUTING.md > Documenting the Rule > General Structure
- **Classification:** MANDATORY
- **Notes:** First paragraph used as brief description. Multi-line breaks table of contents.

### 6.17 Rule documentation: MUST have `## Examples` with `### Invalid` first, then `### Valid`
- **Source:** Analyzer CONTRIBUTING.md > Documenting the Rule > General Structure
- **Classification:** MANDATORY
- **Notes:** Invalid before Valid (shows when rule triggers). Code blocks must use `expect_diagnostic` for invalid examples.

### 6.18 Rule documentation: update `language` field appropriately
- **Source:** Analyzer CONTRIBUTING.md > Documenting the Rule > Associated Language(s)
- **Classification:** MANDATORY
- **Notes:** For YAML rules, set `language: "yaml"`.

### 6.19 Rule documentation: code blocks auto-validated by build process
- **Source:** Analyzer CONTRIBUTING.md > Documenting the Rule > Code Blocks
- **Classification:** MANDATORY (CI enforces)
- **Notes:** Valid examples must parse without diagnostics. Invalid examples must emit exactly ONE diagnostic. Use `ignore` property sparingly to skip validation.

### 6.20 Run `just gen-analyzer` after rule changes
- **Source:** Analyzer CONTRIBUTING.md > Code generation
- **Classification:** MANDATORY
- **Notes:** Regenerates registry, lint modules, build.rs. **Alternative:** `cargo run -p xtask_codegen -- analyzer`

### 6.21 Domains: consult maintainers before adding new domains
- **Source:** Analyzer CONTRIBUTING.md > Rule domains
- **Classification:** MANDATORY (governance)
- **Notes:** Domains affect auto-enablement behavior. Not relevant for initial YAML rules.

---

## 7. Contribution Process

### 7.1 Disclose AI assistance in PR
- **Source:** Root CONTRIBUTING.md > AI assistance notice
- **Classification:** MANDATORY
- **Notes:** Must disclose extent of AI usage. Example formats provided.

### 7.2 Follow conventional commit format
- **Source:** Root CONTRIBUTING.md > Commit messages
- **Classification:** MANDATORY (CI checks PR titles)
- **Notes:** Prefixes: `build:`, `chore:`, `ci:`, `docs:`, `feat:`, `fix:`, `perf:`, `refactor:`, `release:`, `revert:`, `test:`. Include scope in parentheses: `feat(biome_yaml_analyze): noDuplicateKeys`.

### 7.3 Bug fixes → `main` branch; new features → `next` branch
- **Source:** Root CONTRIBUTING.md > Creating pull requests
- **Classification:** MANDATORY
- **Notes:** Nursery rules go to `main`. Rule promotions go to `next`. New end-user features go to `next`. Internal features go to `main`.

### 7.4 Create changeset with `just new-changeset`
- **Source:** Root CONTRIBUTING.md > Changelog
- **Classification:** MANDATORY (for user-visible changes)
- **Notes:** Requires `pnpm`. Changeset guidelines:
  - Focus on user-facing changes
  - Be concise (1-3 sentences)
  - Use past tense for actions, present tense for Biome behavior
  - Link issues with `Fixed [#NNNN](...)`
  - Link rules to website
  - End sentences with full stop

### 7.5 Changeset type: `patch` for fixes, `minor` for features, `major` for breaking
- **Source:** Root CONTRIBUTING.md > Choose the correct type of change
- **Classification:** MANDATORY
- **Notes:** `minor`/`major` require PR to target `next` branch.

### 7.6 Commit format: `feat(crate_name): description`
- **Source:** Analyzer CONTRIBUTING.md > Committing your work
- **Classification:** MANDATORY
- **Notes:** Example: `feat(biome_yaml_analyze): noDuplicateKeys`

### 7.7 Documentation PRs to website repo
- **Source:** Root CONTRIBUTING.md > Documentation
- **Classification:** OPTIONAL (deferred until feature PR merges)
- **Assessment:** New formatter options and similar features need website docs. Can be done as follow-up PR.

---

## 8. Release-Related (NOT applicable during development)

### 8.1 Replace `version: "next"` with actual version at release
- **Source:** Root CONTRIBUTING.md > Releasing
- **Classification:** NOT APPLICABLE (release process)

### 8.2 Beta release workflow
- **Source:** Root CONTRIBUTING.md > Releasing > Beta releases
- **Classification:** NOT APPLICABLE (release process)

### 8.3 Regular release workflow
- **Source:** Root CONTRIBUTING.md > Releasing > Regular releases
- **Classification:** NOT APPLICABLE (release process)

---

## 9. Code Style & Lint Policy (from config files)

These are implicit conventions encoded in configuration files — not documented in any CONTRIBUTING.md but enforced by CI.

### 9.1 Workspace Clippy lints (~50 rules)
- **Source:** `Cargo.toml` > `[workspace.lints.clippy]` (lines 174-241)
- **Classification:** MANDATORY (enforced by `cargo clippy`)
- **Notes:** Key rules affecting YAML development:
  - `allow_attributes = "deny"` — cannot use `#[allow(...)]` without `#[expect(...)]` or justification
  - `dbg_macro = "warn"` — `dbg!()` triggers warnings (allowed in tests via `clippy.toml`)
  - `get_unwrap = "warn"` — prefer `.get()` with error handling over indexing
  - `inefficient_to_string = "warn"` — avoid unnecessary `.to_string()` calls
  - `needless_for_each = "warn"` — use `for` loops over `.for_each()`
  - `from_iter_instead_of_collect = "warn"` — use `.collect()` not `FromIterator::from_iter()`
  - `implicit_clone = "warn"` — be explicit about cloning
  - `large_types_passed_by_value = "warn"` — pass large types by reference
  - `rc_buffer = "warn"` / `rc_mutex = "warn"` — avoid `Rc<Vec<T>>` and `Rc<Mutex<T>>` patterns
  - `multiple_crate_versions = "allow"` — duplicate deps are acceptable (exception)

### 9.2 Workspace Rust compiler lints (~12 rules)
- **Source:** `Cargo.toml` > `[workspace.lints.rust]` (lines 243-258)
- **Classification:** MANDATORY (enforced by compiler)
- **Notes:** Key rules:
  - `dead_code = "warn"` — unused code triggers warnings
  - `unused_lifetimes = "warn"` / `unused_import_braces = "warn"` / `unused_macro_rules = "warn"`
  - `trivial_numeric_casts = "warn"` — no-op casts
  - `ambiguous-negative-literals = "warn"`
  - `missing-unsafe-on-extern = "warn"` — extern blocks need `unsafe`

### 9.3 Disallowed methods (clippy.toml)
- **Source:** `clippy.toml`
- **Classification:** MANDATORY (enforced by clippy)
- **Notes:** Three methods are banned project-wide:
  - `str::to_ascii_lowercase` → use `biome_string_case::StrOnlyExtension::to_ascii_lowercase_cow`
  - `std::ffi::OsStr::to_ascii_lowercase` → use `biome_string_case::StrLikeExtension::to_ascii_lowercase_cow`
  - `str::to_lowercase` → use `biome_string_case::StrOnlyExtension::to_lowercase_cow`
  - Reason: avoid heap allocations for case conversion
  - `allow-dbg-in-tests = true` — `dbg!()` is permitted in test code

### 9.4 Rust formatting conventions (rustfmt.toml)
- **Source:** `rustfmt.toml`
- **Classification:** MANDATORY (enforced by CI)
- **Notes:** `edition = "2024"`, `newline_style = "Unix"`, `style_edition = "2024"`. All Rust files formatted with these settings.

### 9.5 Editor conventions (.editorconfig)
- **Source:** `.editorconfig`
- **Classification:** MANDATORY (implicit standard)
- **Notes:**
  - Default: LF line endings, trim trailing whitespace, insert final newline, UTF-8
  - Default indent: tabs, size 2
  - Override for `*.yml`, `*.md`, `*.rs`, `*.json`, `*.jsonc`, `justfile`: spaces
  - Override for `*.rs`: 4-space indent
  - Comment in file: "YAML doesn't support hard tabs"

### 9.6 Build profiles
- **Source:** `Cargo.toml` > `[profile.*]` (lines 259-285)
- **Classification:** MANDATORY (affects debugging capability)
- **Notes:**
  - `[profile.dev]`: `debug = "line-tables-only"` — stripped debug info (why `--profile debugging` is needed)
  - `[profile.debugging]`: `inherits = "dev"`, `debug = true` — full debug symbols
  - `[profile.release-with-debug]`: `inherits = "release"`, `debug = true`

---

## 10. CI/CD Requirements (from workflow files)

These are the actual CI gates. Code that fails these checks cannot be merged.

### 10.1 PR CI jobs (pull_request.yml)
- **Source:** `.github/workflows/pull_request.yml`
- **Classification:** MANDATORY (merge-blocking)
- **Notes:** Five required jobs on every PR to `main` or `next`:
  1. **Lint** — `cargo lint` (clippy) on Windows + Ubuntu ARM
  2. **Check Dependencies** — `cargo +nightly udeps --all-targets` (detects unused deps)
  3. **Test** — `cargo test --workspace --features=js_plugin` on Windows + Ubuntu ARM
  4. **E2E Tests** — builds debug binary, runs `e2e-tests/test-all.sh` on Ubuntu ARM
  5. **Documentation** — `cargo documentation` on Ubuntu ARM
- **Trigger paths:** Only runs when `crates/**`, `fuzz/**`, `xtask/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `rustfmt.toml` change

### 10.2 Main branch CI jobs (main.yml)
- **Source:** `.github/workflows/main.yml`
- **Classification:** MANDATORY (post-merge gate)
- **Notes:** Runs on push to `main`. Adds to PR checks:
  - **Format** — `cargo fmt --all --verbose -- --check` + `tombi format --check`
  - **Test** — expanded to Windows + Ubuntu ARM + **macOS** (3 OS matrix)
  - **Coverage** — Test262 suite on Windows + Ubuntu (continue-on-error)
  - `cargo check --workspace --all-targets --release` — full release check

### 10.3 PR title lint (pull_request_title_lint.yaml)
- **Source:** `.github/workflows/pull_request_title_lint.yaml`
- **Classification:** MANDATORY (merge-blocking)
- **Notes:** Enforced rules beyond what CONTRIBUTING.md states:
  - Subject must NOT start with uppercase: `^[^A-Z].*$`
  - Disallowed scopes: `release`, any `[A-Z_-]+` (all-uppercase scopes)
  - `requireScope: false` — scope is optional
  - Error message: "Please ensure that the subject doesn't start with an uppercase character."

### 10.4 Unused dependency detection
- **Source:** `.github/workflows/pull_request.yml` > `check-dependencies` job
- **Classification:** MANDATORY (merge-blocking)
- **Notes:** `cargo +nightly udeps --all-targets` runs on nightly Rust. New crate dependencies must actually be used. This catches Cargo.toml entries that are declared but unused.

---

## 11. Diagnostics Development

### 11.1 Diagnostic quality principles
- **Source:** `crates/biome_diagnostics/CONTRIBUTING.md` > What is a Diagnostic
- **Classification:** MANDATORY (quality standard)
- **Notes:** Four principles for all diagnostics:
  1. Follow [Technical Principles](https://biomejs.dev/#technical)
  2. Explain WHY something went wrong, not just THAT it went wrong. Add hyperlinks to docs.
  3. Provide a way to fix the issue (log advice, diff advice, or command advice). Add `FIXABLE` tag if actionable.
  4. "Show don't tell" — prefer code frames, diffs, commands over textual explanations

### 11.2 Diagnostic trait properties
- **Source:** `crates/biome_diagnostics/CONTRIBUTING.md` > The `Diagnostic` trait
- **Classification:** MANDATORY (implementation reference)
- **Notes:** Every diagnostic has:
  - `category` — unique string (e.g., `lint/suspicious/noDuplicateKeys`)
  - `severity` — Fatal, Error, Warning, Information, Hint
  - `description` — plain text for contexts without rich markup (editor popovers)
  - `message` — rich markup, displayed at top of diagnostic advices (short and clear)
  - `advices` — rich building blocks: log, list, code frame, diff, backtrace, command
  - `verbose_advices` — additional info behind `--verbose` flag
  - `location` — file path + optional range + optional source content
  - `tags` — FIXABLE, internal error, deprecated/unused code
  - `source` — optional chained diagnostic (e.g., deserialization error behind a request error)

### 11.3 Use `#[derive(Diagnostic)]` macro
- **Source:** `crates/biome_diagnostics/CONTRIBUTING.md` > How to implement Diagnostic
- **Classification:** MANDATORY (standard pattern)
- **Notes:** Use the derive macro with `#[diagnostic]` attribute for static properties and field attributes for dynamic ones:
  ```rust
  #[derive(Debug, Diagnostic)]
  #[diagnostic(severity = Warning, category = "internalError/fs")]
  struct MyDiagnostic {
      #[message]
      #[description]
      #[advice]
      info: MyAdviceType
  }
  ```
- Helper types: `CodeFrameAdvice`, `CommandAdvice`, `DiffAdvice`, `LogAdvice`

### 11.4 Register new diagnostic categories
- **Source:** `crates/biome_diagnostics/CONTRIBUTING.md` > bottom
- **Classification:** MANDATORY (when creating new categories)
- **Notes:** All diagnostic categories must be statically registered in `crates/biome_diagnostics_categories/src/categories.rs`. Already done for `lint/suspicious/noDuplicateKeys`.

---

## 12. Service Integration

### 12.1 Workspace trait architecture
- **Source:** `crates/biome_service/CONTRIBUTING.md`
- **Classification:** MANDATORY (architectural understanding for Stage 4)
- **Notes:** Two implementations of the `Workspace` trait:
  - `WorkspaceServer` — maintains state itself (used in daemon AND daemonless CLI mode)
  - `WorkspaceClient` — connects to daemon, communicates with `WorkspaceServer`
- YAML integration goes into `WorkspaceServer` path via `file_handlers/yaml.rs`

### 12.2 Watcher (daemon mode only)
- **Source:** `crates/biome_service/CONTRIBUTING.md` > Watcher
- **Classification:** OPTIONAL (not needed for YAML implementation)
- **Assessment:** `WorkspaceWatcher` keeps state in sync with filesystem. Only active in daemon mode. Tests in `watcher.tests.rs` and LSP tests. Not required for basic YAML support.

### 12.3 Debugging the service layer
- **Source:** `crates/biome_service/CONTRIBUTING.md` > Debugging
- **Classification:** OPTIONAL (HIGH utility for Stage 4 debugging)
- **Assessment:** Two approaches:
  1. Start daemon: `cargo run --bin=biome -- start`
  2. Run against daemon: `cargo run --bin=biome -- lint --use-server <path>`
  - Logs written to cache folder (path in `crates/biome_fs/src/dir.rs`)

---

## 13. Formatter Rules (from JS formatter CONTRIBUTING)

### 13.1 Use AST tokens, not string literals
- **Source:** `crates/biome_js_formatter/CONTRIBUTING.md` > Rules when formatting AST nodes
- **Classification:** MANDATORY (formatter correctness)
- **Notes:** If a token is mandatory and the AST provides it, use the AST token:
  ```rust
  write!(f, [node.l_paren_token().format()])?;  // YES
  write!(f, [token("(")])?;                       // NO
  ```

### 13.2 Do not attempt to "fix" code in the formatter
- **Source:** `crates/biome_js_formatter/CONTRIBUTING.md` > Rules when formatting AST nodes
- **Classification:** MANDATORY (design principle)
- **Notes:** If a token/node is mandatory but missing, return `None`. The formatter preserves semantics — it does not fix syntax errors.

### 13.3 Use `dbg_write!` for IR debugging
- **Source:** `crates/biome_js_formatter/CONTRIBUTING.md` > Debugging formatter output
- **Classification:** OPTIONAL (HIGH utility for formatter development)
- **Assessment:** `dbg_write!(f, [...])` prints IR elements to console:
  ```
  [src/main.rs:1][0] = StaticToken("hello")
  [src/main.rs:1][1] = Space
  ```
  Essential for understanding what IR the formatter produces. **No alternative** for IR inspection.

---

## 14. PR Template & Review Process

### 14.1 PR template sections
- **Source:** `.github/PULL_REQUEST_TEMPLATE.md`
- **Classification:** MANDATORY (structure for all PRs)
- **Notes:** Three required sections:
  1. **Summary** — motivation, problem solved, link to issues, changeset reference
  2. **Test Plan** — what demonstrates correctness
  3. **Docs** — for rules/actions: inline rustdoc. For other features: PR to website `next` branch

### 14.2 AI assistance disclosure in PR template
- **Source:** `.github/PULL_REQUEST_TEMPLATE.md` (HTML comment at top)
- **Classification:** MANDATORY (reinforces 7.1)
- **Notes:** Template contains a visible reminder linking to `CONTRIBUTING.md#ai-assistance-notice`.

### 14.3 CodeRabbit automated review
- **Source:** `.coderabbit.yaml`
- **Classification:** INFORMATIONAL (not mandatory to follow, but good to understand)
- **Notes:**
  - Profile: "chill" (lenient)
  - Excludes generated files: `**/generated/**`, `**/biome_unicode_table/src/tables.rs`, `**/nursery.rs`, `**/categories.rs`, `**/rules.rs`
  - Uses `CLAUDE.md` and `CONTRIBUTING.md` as knowledge base for guidelines
  - Biome and Clippy tools disabled (Biome uses its own workflows)
  - Auto-review on `main` and `next` branches, not on drafts

---

## 15. Governance & Code Review Philosophy

### 15.1 Liberal code review approach
- **Source:** `GOVERNANCE.md` > Code review
- **Classification:** MANDATORY (process understanding)
- **Notes:** Key principles:
  - "We value quick iteration and low development friction"
  - "Reverting code is easy, so landing code should be just as easy"
  - If you own a particular area, you can merge without review
  - All code must go through PRs and pass status checks
  - If a PR breaks `main`, either revert or quick-fix as separate PR
  - Small changes/bug fixes to your own code can be merged without review
  - Discrete releases (not rolling) — gives safety margin for reversions

### 15.2 Contributor tiers
- **Source:** `GOVERNANCE.md` > Contributor Model
- **Classification:** INFORMATIONAL
- **Notes:** Three tiers: Maintainer → Core Contributor → Lead. Each has different push access, voting rights, and responsibilities. Relevant context for understanding who can approve/merge PRs.

### 15.3 Code of Conduct
- **Source:** `CODE_OF_CONDUCT.md`
- **Classification:** MANDATORY (community standard)
- **Notes:** Contributor Covenant 2.1. Enforcement: correction → warning → temporary ban → permanent ban. Violations reported to `biomejs@googlegroups.com`.

---

## Summary: Gate-Ready Extraction

The following instructions map directly to the 5 gates defined in `agent-evolution-model.md`:

### Gate 1: Environment Readiness (pre-Phase 4)
| Instruction | ID | Mandatory |
|---|---|---|
| Rust stable toolchain installed | 1.1 | Yes |
| `cargo-insta` available | 1.3 | Yes |
| Codegen can run (`xtask_codegen` builds) | 2.4 | Yes |
| `cargo fmt` works | 2.2 | Yes |
| `cargo clippy` works (workspace lints active) | 2.3, 9.1 | Yes |
| `.editorconfig` respected (LF endings, spaces for .rs) | 9.5 | Yes |

### Gate 2: Stage Start (per implementation stage)
| Instruction | ID | Mandatory |
|---|---|---|
| Run appropriate codegen | 2.4 | Yes |
| Grammar complete before formatter | 5.1 | Yes |
| Parser conventions followed | 4.1-4.6 | Yes |
| New diagnostic categories registered | 11.4 | Yes |

### Gate 3: Debug Hygiene (per commit)
| Instruction | ID | Mandatory |
|---|---|---|
| `cargo fmt` passes | 2.2, 9.4 | Yes |
| `cargo clippy` passes (no disallowed methods) | 2.3, 9.3 | Yes |
| `cargo test` passes for affected crate | 3.1 | Yes |
| No `dbg!` macros in non-test code | 9.1, 9.3 | Yes |
| No unused dependencies | 10.4 | Yes |
| Formatter uses AST tokens, not string literals | 13.1 | Yes |
| Formatter does not "fix" code | 13.2 | Yes |

### Gate 4: Code Quality (end of Phase 4)
| Instruction | ID | Mandatory |
|---|---|---|
| Codegen in sync | 2.4 | Yes |
| Snapshot tests accepted | 3.2 | Yes |
| Rule diagnostics have WHAT/WHY/FIX | 6.4, 11.1 | Yes |
| Diagnostics use `#[derive(Diagnostic)]` pattern | 11.3 | Yes |
| Rule documentation valid (single-line first para, examples) | 6.16-6.19 | Yes |
| Naming conventions followed | 6.3 | Yes |
| Rules in `nursery` group | 6.5 | Yes |
| `version: "next"` | 6.6 | Yes |
| Service integration: `WorkspaceServer` path correct | 12.1 | Yes |

### Gate 5: PR Readiness (before contribution)
| Instruction | ID | Mandatory |
|---|---|---|
| AI assistance disclosed | 7.1, 14.2 | Yes |
| Conventional commit format (lowercase subject) | 7.2, 10.3 | Yes |
| Correct target branch | 7.3 | Yes |
| Changeset created | 7.4 | Yes |
| PR has Summary + Test Plan + Docs sections | 14.1 | Yes |
| `just ready` passes (or equivalent) | 2.2-2.4 | Yes |
| `cargo +nightly udeps` passes (no unused deps) | 10.4 | Yes |

---

## Appendix A: Instructions NOT Relevant to YAML

| Instruction | Source | Reason |
|---|---|---|
| `.jsonc` test files for script mode | Analyzer 3.5 | JS-specific |
| Semantic model / `Semantic<>` query | Analyzer 6.15 | YAML has no semantic model |
| `StrictMode` / conditional syntax | Parser 4.9 | YAML has no strict/sloppy modes |
| WASM build tools | Root 1.3 | Not needed for YAML |
| Node.js / pnpm development | Root 1.4 | Only for changesets |
| Rule domains / `RuleDomain` | Analyzer 6.21 | Not applicable to initial YAML rules |
| Multi-file snippet tests (`file=<path>`) | Analyzer 6.19 | YAML rules don't analyze cross-file relationships |
| `RuleSource::Eslint` attribution | Analyzer 6.7 | YAML rules are novel, not ported |
| Workspace watcher / daemon mode | Service 12.2 | Not needed for basic YAML support |
| Test262 coverage suite | CI 10.2 | JS-specific conformance testing |
| `biome_js_type_info` architecture | JS type info CONTRIBUTING | JS-specific type system |
| `biome_aria_metadata` generation | ARIA CONTRIBUTING | Accessibility metadata, not YAML |
| Financial contributions / bounties | Governance 15.2 | Organizational, not technical |

---

## Appendix B: Source File Index

All files contributing to this specification, grouped by category:

**CONTRIBUTING.md files (primary documentation):**
- `CONTRIBUTING.md` — root, general workflow
- `crates/biome_analyze/CONTRIBUTING.md` — analyzer/lint rules (most detailed, 1625 lines)
- `crates/biome_formatter/CONTRIBUTING.md` — formatter crate creation
- `crates/biome_parser/CONTRIBUTING.md` — parser/grammar conventions
- `crates/biome_diagnostics/CONTRIBUTING.md` — diagnostic quality and implementation
- `crates/biome_service/CONTRIBUTING.md` — workspace/service layer
- `crates/biome_js_formatter/CONTRIBUTING.md` — formatter design rules (applies cross-language)

**Configuration files (implicit conventions):**
- `Cargo.toml` `[workspace.lints.*]` — 60+ lint rules enforced by clippy/rustc
- `clippy.toml` — disallowed methods, `dbg!` test policy
- `rustfmt.toml` — Rust edition/style settings
- `.editorconfig` — indentation, line endings, charset

**CI/CD files (hard gates):**
- `.github/workflows/pull_request.yml` — 5 PR jobs
- `.github/workflows/main.yml` — post-merge checks + multi-OS testing
- `.github/workflows/pull_request_title_lint.yaml` — PR title validation rules

**Process files:**
- `.github/PULL_REQUEST_TEMPLATE.md` — required PR sections
- `.coderabbit.yaml` — automated review configuration
- `GOVERNANCE.md` — code review philosophy, contributor tiers
- `CODE_OF_CONDUCT.md` — community standards
