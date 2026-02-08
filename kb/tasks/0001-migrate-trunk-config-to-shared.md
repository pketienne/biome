# Task: Migrate Local Trunk Configuration to Shared Plugin

**Status:** Planned
**Created:** 2026-02-01
**Target:** trunk-pke v1.1.0

---

## Summary

Migrate mnemosyne's local Trunk/Biome configuration to the shared trunk-pke plugin, consolidating settings and enabling reuse across projects.

---

## Decisions Made

| Category               | Decision                                 |
| ---------------------- | ---------------------------------------- |
| Biome file types       | Add jsonc, svelte, vue to shared         |
| dprint file types      | Markdown only (biome handles JS/TS/JSON) |
| lineWidth              | Change to 80                             |
| lineEnding             | Keep "lf"                                |
| quoteStyle             | Keep "single"                            |
| tailwindDirectives     | Keep true                                |
| cssModules             | Keep true                                |
| Linter rules           | recommended: true + complexity rules     |
| Next.js domain         | Keep                                     |
| React domain           | Keep                                     |
| VCS integration        | Keep                                     |
| ignoreUnknown          | Change to false + explicit ignores       |
| files.includes         | Remove (rely on VCS + explicit ignores)  |
| organizeImports        | Keep                                     |
| Tests                  | Both pre-commit and pre-push             |
| Typecheck              | Both pre-commit and pre-push             |
| pnpm audit             | Pre-push only                            |
| Cargo/boops actions    | Keep                                     |
| trunk-announce/upgrade | Keep                                     |

---

## Phase 1: Update trunk-pke Plugin

### 1.1 Update `plugin.yaml`

**File:** `~/Projects/trunk-pke/plugin.yaml`

**Changes:**

- [ ] Add vue file type definition
- [ ] Add jsonc file type definition
- [ ] Add svelte, vue, jsonc to biome files list
- [ ] Change dprint to markdown only
- [ ] Add pnpm-typecheck-pre-commit action
- [ ] Add pnpm-typecheck-pre-push action
- [ ] Add pnpm-test-pre-push action
- [ ] Add pnpm-audit-pre-push action
- [ ] Update actions.enabled list

**New plugin.yaml content:**

```yaml
version: 0.1
required_trunk_version: ">=1.22.8"

actions:
  definitions:
    - id: boops-export-pre-commit
      display_name: Export Boops Store
      run: "[ -f Cargo.toml ] && cargo run --quiet -- export || true"
      triggers:
        - git_hooks: [pre-commit]

    - id: cargo-audit-pre-push
      display_name: Cargo Security Audit
      run: "[ -f Cargo.toml ] && mise exec -- cargo audit || true"
      triggers:
        - git_hooks: [pre-push]

    - id: cargo-deny-pre-push
      display_name: Cargo Dependency Check
      run: "[ -f Cargo.toml ] && mise exec -- cargo deny check --config .trunk/configs/deny.toml || true"
      triggers:
        - git_hooks: [pre-push]

    - id: cargo-test-pre-commit
      display_name: Cargo Tests
      run: "[ -f Cargo.toml ] && cargo test || true"
      triggers:
        - git_hooks: [pre-commit]

    - id: pnpm-test-pre-commit
      display_name: Pnpm Tests (Pre-commit)
      run: "[ -f package.json ] && mise exec -- pnpm test || true"
      triggers:
        - git_hooks: [pre-commit]

    - id: pnpm-test-pre-push
      display_name: Pnpm Tests (Pre-push)
      run: "[ -f package.json ] && mise exec -- pnpm test || true"
      triggers:
        - git_hooks: [pre-push]

    - id: pnpm-typecheck-pre-commit
      display_name: Pnpm Typecheck (Pre-commit)
      run: "[ -f package.json ] && mise exec -- pnpm typecheck || true"
      triggers:
        - git_hooks: [pre-commit]

    - id: pnpm-typecheck-pre-push
      display_name: Pnpm Typecheck (Pre-push)
      run: "[ -f package.json ] && mise exec -- pnpm typecheck || true"
      triggers:
        - git_hooks: [pre-push]

    - id: pnpm-audit-pre-push
      display_name: Pnpm Security Audit
      run: "[ -f package.json ] && mise exec -- pnpm audit --audit-level=high || true"
      triggers:
        - git_hooks: [pre-push]

  enabled:
    - boops-export-pre-commit
    - cargo-audit-pre-push
    - cargo-deny-pre-push
    - cargo-test-pre-commit
    - pnpm-audit-pre-push
    - pnpm-test-pre-commit
    - pnpm-test-pre-push
    - pnpm-typecheck-pre-commit
    - pnpm-typecheck-pre-push
    - trunk-announce
    - trunk-check-pre-commit
    - trunk-check-pre-push
    - trunk-fmt-pre-commit
    - trunk-upgrade-available

lint:
  definitions:
    - name: biome
      files:
        - javascript
        - typescript
        - json
        - jsonc
        - css
        - graphql
        - astro
        - svelte
        - vue

    - name: dprint
      tools: [dprint]
      files: [markdown]
      commands:
        - name: format
          run: dprint fmt ${target}
          output: rewrite
          success_codes: [0]
          formatter: true
          in_place: true
      direct_configs:
        - .trunk/configs/dprint.json
        - dprint.json
      affects_cache:
        - dprint.json
        - .trunk/configs/dprint.json
      suggest_if: config_present

    - name: rustfmt
      direct_configs:
        - .trunk/configs/rustfmt.toml
        - rustfmt.toml

    - name: serdi
      description: RDF syntax validator and canonical formatter
      files: [turtle]
      known_good_version: 0.32.6
      suggest_if: files_present
      commands:
        - name: format
          formatter: true
          output: rewrite
          run: serdi -o turtle ${target}
          success_codes: [0]
        - name: lint
          output: pass_fail
          read_output_from: stderr
          run: serdi -o turtle ${target}
          success_codes: [0, 1]

    - name: markdownlint
      direct_configs:
        - .trunk/configs/.markdownlint.json
        - .markdownlint.json

    - name: taplo
      direct_configs:
        - .trunk/configs/.taplo.toml
        - .taplo.toml
        - taplo.toml

  disabled:
    - prettier

  enabled:
    - bandit@1.9.3
    - biome@2.3.11
    - black@26.1.0
    - checkov@3.2.499
    - clippy@1.93.0
    - dprint@0.51.1
    - git-diff-check
    - isort@7.0.0
    - markdownlint@0.47.0
    - osv-scanner@2.3.2
    - ruff@0.14.14
    - rustfmt@1.93.0
    - serdi
    - shellcheck@0.11.0
    - shfmt@3.6.0
    - taplo@0.10.0
    - trufflehog@3.92.5
    - yamllint@1.38.0

  exported_configs:
    - configs:
        - configs/.editorconfig
        - configs/.markdownlint.json
        - configs/.taplo.toml
        - configs/biome.json
        - configs/deny.toml
        - configs/dprint.json
        - configs/rustfmt.toml

  files:
    - name: turtle
      comments:
        - hash
      extensions:
        - ttl
    - name: vue
      comments:
        - html-tag
        - slashes-block
      extensions:
        - vue
    - name: jsonc
      extensions:
        - jsonc

  ignore:
    - linters: [ALL]
      paths:
        - "**/.git/**"
        - "**/node_modules/**"
        - .git/**
        - .next/**
        - .open-next/**
        - .sst/**
        - .trunk/**
        - .turbo/**
        - build/**
        - dist/**
        - node_modules/**
        - target/**
        - vendor/**

runtimes:
  enabled:
    - go@1.21.0
    - node@22.16.0
    - python@3.10.8

tools:
  definitions:
    - name: dprint
      runtime: node
      package: dprint
      shims: [dprint]
      known_good_version: 0.51.1

  enabled:
    - dprint@0.51.1
```

---

### 1.2 Update `configs/biome.json`

**File:** `~/Projects/trunk-pke/configs/biome.json`

**Changes:**

- [ ] Change lineWidth: 100 → 80
- [ ] Change ignoreUnknown: true → false
- [ ] Remove files.includes
- [ ] Add files.ignore with explicit patterns
- [ ] Add complexity rules to linter

**New biome.json content:**

```json
{
  "$schema": "https://biomejs.dev/schemas/2.3.11/schema.json",
  "vcs": {
    "enabled": true,
    "clientKind": "git",
    "useIgnoreFile": true
  },
  "files": {
    "ignoreUnknown": false,
    "ignore": [
      "*.png", "*.jpg", "*.jpeg", "*.gif", "*.svg", "*.ico", "*.webp",
      "*.woff", "*.woff2", "*.ttf", "*.eot", "*.otf",
      "*.pdf", "*.zip", "*.tar", "*.gz", "*.tgz",
      "*.lock", "pnpm-lock.yaml", "package-lock.json", "yarn.lock",
      "*.md",
      "*.sql", "*.prisma",
      "*.env", "*.env.*",
      "*.csv", "*.xml"
    ]
  },
  "formatter": {
    "enabled": true,
    "indentStyle": "tab",
    "lineEnding": "lf",
    "lineWidth": 80
  },
  "json": {
    "formatter": {
      "enabled": true,
      "indentStyle": "tab",
      "lineEnding": "lf",
      "trailingCommas": "none"
    }
  },
  "css": {
    "parser": {
      "cssModules": true,
      "tailwindDirectives": true
    }
  },
  "javascript": {
    "formatter": {
      "quoteStyle": "single"
    }
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true,
      "suspicious": {
        "noUnknownAtRules": "off"
      },
      "complexity": {
        "noExcessiveCognitiveComplexity": {
          "level": "warn",
          "options": { "maxAllowedComplexity": 15 }
        },
        "noExcessiveNestedTestSuites": "warn"
      }
    },
    "domains": {
      "next": "recommended",
      "react": "recommended"
    }
  },
  "assist": {
    "actions": {
      "source": {
        "organizeImports": "on"
      }
    }
  }
}
```

---

### 1.3 Update `configs/dprint.json`

**File:** `~/Projects/trunk-pke/configs/dprint.json`

**Changes:**

- [ ] Remove biome plugin
- [ ] Update lineWidth to 80
- [ ] Keep markdown plugin only

**New dprint.json content:**

```json
{
  "$schema": "https://dprint.dev/schemas/v0.json",
  "lineWidth": 80,
  "newLineKind": "lf",
  "markdown": {
    "textWrap": "maintain",
    "emphasisKind": "underscores",
    "strongKind": "asterisks"
  },
  "excludes": ["**/node_modules", "**/.git", "**/dist", "**/build", "**/target"],
  "plugins": [
    "https://plugins.dprint.dev/markdown-0.17.8.wasm"
  ]
}
```

---

## Phase 2: Update mnemosyne Local Config

### 2.1 Simplify `.trunk/trunk.yaml`

**File:** `~/Projects/mnemosyne/.trunk/trunk.yaml`

**Changes:**

- [ ] Remove lint.files definitions
- [ ] Remove lint.definitions
- [ ] Remove actions.definitions
- [ ] Update plugin ref to v1.1.0

**New trunk.yaml content:**

```yaml
# This file controls the behavior of Trunk: https://docs.trunk.io/cli
# To learn more about the format of this file, see https://docs.trunk.io/reference/trunk-yaml
version: 0.1
cli:
  version: 1.25.0

# Trunk provides extensibility via plugins. (https://docs.trunk.io/plugins)
plugins:
  sources:
    - id: trunk
      ref: v1.7.4
      uri: https://github.com/trunk-io/plugins
    - id: trunk-pke
      ref: v1.1.0
      uri: https://github.com/pketienne/trunk-pke
```

---

### 2.2 Delete local `biome.json`

**File:** `~/Projects/mnemosyne/biome.json`

- [ ] Delete this file (shared config will be used)

---

## Phase 3: Version Bump & Test

### 3.1 Bump trunk-pke version

- [ ] Update version references to v1.1.0
- [ ] Commit changes
- [ ] Create git tag v1.1.0
- [ ] Push to remote

### 3.2 Update mnemosyne to use new version

- [ ] Update trunk.yaml ref to v1.1.0
- [ ] Run `trunk check` to verify config loads

### 3.3 Run verification tests

- [ ] `trunk check --all` passes
- [ ] `trunk fmt --all` works
- [ ] Test pre-commit hooks (make a test commit)
- [ ] Test pre-push hooks (make a test push)
- [ ] Verify biome handles: js, ts, jsx, tsx, json, jsonc, css, graphql, astro, svelte, vue
- [ ] Verify dprint handles: markdown only

---

## Execution Order

1. Update trunk-pke/configs/biome.json
2. Update trunk-pke/configs/dprint.json
3. Update trunk-pke/plugin.yaml
4. Test trunk-pke locally
5. Commit, tag v1.1.0, push trunk-pke
6. Update mnemosyne/.trunk/trunk.yaml (ref → v1.1.0)
7. Delete mnemosyne/biome.json
8. Test mnemosyne with new shared config
9. Commit mnemosyne changes

---

## Rollback Plan

If issues are encountered:

1. Revert mnemosyne trunk.yaml to ref: v1.0.3
2. Restore mnemosyne/biome.json from git
3. Investigate and fix trunk-pke issues
4. Re-attempt migration

---

## Notes

- Cargo/boops actions auto-skip if no Cargo.toml (harmless for JS-only projects)
- pnpm actions auto-skip if no package.json (harmless for Rust-only projects)
- SST projects may have typecheck failures due to generated Resource types - consider running `sst dev` first or adjusting typecheck action
