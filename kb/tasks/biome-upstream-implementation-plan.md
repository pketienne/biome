# Plan: Local Biome Fork with Markdown Support

## Context

Biome doesn't support markdown yet. Upstream development is in progress but slow — parser at ~70%, formatter boilerplate in an unmerged PR, linter nonexistent, no service integration. We can't wait for the community.

**Goal:** Fork biome, add markdown linting and formatting ourselves, and use the custom build locally. Move fast, implement what we need, skip what we don't. Upstream contribution is a future concern — keep the fork rebasing-friendly but don't let that slow us down.

## Step 0: Fork and Assess

- Fork `https://github.com/biomejs/biome` to a personal GitHub repo (enables easy rebasing from upstream later)
- Clone into `/home/pke/Projects/biome-markdown/biome`
- Cherry-pick unmerged community PRs that help us:
  - PR #8908 (parser structure improvements) — if not already on main
  - PR #8962 (formatter boilerplate) — saves us scaffolding work
- Assess what the parser actually handles today (run conformance tests)
- Identify which markdown constructs we use most and need support for

## Step 1: Service Integration (wire the pipeline)

Do this first — it's the skeleton everything else hangs on. Without it, nothing is testable end-to-end.

- Create `crates/biome_service/src/file_handlers/markdown.rs`
- Register `.md` and `.mdx` file extensions in `DocumentFileSource`
- Add `markdown` field to `Features` struct
- Implement `ExtensionHandler` with parser capability (formatter + analyzer stubs)
- Add minimal `markdown` section to `biome.json` configuration schema
- **Milestone:** `biome check README.md` runs without crashing (even if it does nothing useful yet)

## Step 2: Linter — First Batch of Rules

Create `biome_markdown_analyze` crate and implement high-value rules that work with the parser as-is.

### Crate scaffold
- `Cargo.toml` with dependencies on `biome_analyze`, `biome_markdown_syntax`, `biome_console`, `biome_diagnostics`
- `src/lib.rs` — `analyze()` entry point, rule metadata registry
- `src/lint/mod.rs` — declare lint groups (correctness, style, suspicious)
- `src/suppression_action.rs` — markdown comment suppression (`<!-- biome-ignore -->`)

### Starter rules (pick 5 that give immediate value)
1. `noMissingLanguage` (MD040) — fenced code blocks without language tag
2. `noInvalidHeadingLevel` (MD001) — heading level jumps (h1 → h3)
3. `noDuplicateHeadings` (MD024) — same heading text repeated
4. `noEmptyLinks` (MD042) — `[text]()` with empty href
5. `noReversedLinks` (MD011) — `(text)[url]` instead of `[text](url)`

### Wire into service
- Update `markdown.rs` file handler to call analyzer
- **Milestone:** `biome lint README.md` produces real diagnostics

## Step 3: Formatter — Basic Markdown Formatting

If PR #8962 boilerplate is available, cherry-pick it. Otherwise scaffold from `biome_html_formatter`.

### Priority format rules (what matters most for daily use)
1. Headings — ATX style normalization, consistent spacing
2. Lists — indentation, marker style (`-` vs `*`)
3. Code blocks — fence style, language tag normalization
4. Blank lines — consistent spacing between blocks
5. Trailing whitespace — removal
6. Thematic breaks — consistent style

### Skip for now
- Paragraph reflowing/prose wrapping (hard, opinionated)
- Table column alignment (requires GFM parser support)
- Embedded code block formatting (nice-to-have, not urgent)

### Wire into service
- Update file handler with formatter capability
- Add formatter config options to `biome.json` markdown section
- **Milestone:** `biome format README.md` produces formatted output

## Step 4: Parser Improvements (as needed)

Only fix parser gaps that block rules we're implementing. Don't chase 100% spec compliance.

Likely needs:
- Fix emphasis/strong if it affects linter rules
- Improve link parsing if `noEmptyLinks`/`noReversedLinks` need it
- GFM table parsing if we want table rules (defer unless needed)

## Step 5: Build and Deploy Locally

- Build optimized binary: `cargo build --release`
- Install to PATH or symlink
- Configure `biome.json` in projects with markdown settings enabled
- Integrate with editor (VS Code extension pointed at custom binary)
- Set up as pre-commit hook or Trunk linter

## Future: Upstream Contribution

When ready to contribute back:
- Isolate clean commits per feature
- Ensure conformance with biome's code style and contribution guidelines
- Open PRs for individual features (linter crate, service integration, individual rules)
- Rebase fork on latest upstream periodically

## Verification

- `cargo build` — full workspace compiles
- `cargo test` — existing tests still pass
- `cargo test -p biome_markdown_analyze` — new lint rule tests pass
- `biome check some-file.md` — produces diagnostics
- `biome format some-file.md` — produces formatted output
- `biome format --write some-file.md` — formats in place
