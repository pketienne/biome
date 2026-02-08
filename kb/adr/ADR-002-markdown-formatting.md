# ADR-002: Markdown Formatting Strategy

**Status:** Proposed
**Date:** 2026-01-25
**Deciders:** pke
**Affects:** mnemosyne, potentially all projects using shared Trunk config

## Context

The mnemosyne project uses Biome for linting and formatting TypeScript, JavaScript, JSON, CSS, and GraphQL. Trunk is being adopted for unified code quality management across the workspace (see [ADR-001](ADR-001-trunk-configuration.md)).

Biome does not support markdown. A strategy is needed for markdown formatting that:

- Integrates with Trunk's pre-commit workflow
- Handles embedded code blocks (JS/TS/JSON) within markdown
- Avoids pulling in heavyweight dependencies for a narrow use case
- Maintains consistency with Biome's formatting where applicable

## Options Considered

### Option 1: Prettier

The standard choice for markdown formatting.

**Markdown-specific options:**

| Option                       | Values                        | Default    |
| ---------------------------- | ----------------------------- | ---------- |
| `proseWrap`                  | `always`, `never`, `preserve` | `preserve` |
| `printWidth`                 | integer                       | `80`       |
| `tabWidth`                   | integer                       | `2`        |
| `useTabs`                    | boolean                       | `false`    |
| `embeddedLanguageFormatting` | `auto`, `off`                 | `auto`     |

**Pros:**

- Well-supported in Trunk (auto-enabled)
- Handles embedded code block formatting automatically
- Battle-tested, widely adopted

**Cons:**

- Large dependency (~2MB+) for limited markdown options
- Cannot configure bullet style, emphasis markers, rule characters
- Would duplicate functionality already handled by Biome for JS/TS/JSON
- "A lot of engine for not a lot of car"

### Option 2: markdownlint only

Linting without formatting.

**Pros:**

- Already in shared Trunk plugin
- Catches style issues

**Cons:**

- Does not format/fix, only reports
- Would need manual fixes or separate formatter

### Option 3: remark-stringify (Custom Formatter)

Build a minimal markdown formatter using the remark ecosystem.

**Architecture:**

```
markdown → remark-parse → mdast tree → [plugins] → remark-stringify → formatted markdown
```

**remark-stringify formatting options:**

| Option           | Values                | Default |
| ---------------- | --------------------- | ------- |
| `bullet`         | `*`, `-`, `+`         | `*`     |
| `listItemIndent` | `one`, `tab`, `mixed` | `one`   |
| `quote`          | `"`, `'`              | `"`     |
| `rule`           | `*`, `-`, `_`         | `*`     |
| `emphasis`       | `*`, `_`              | `*`     |
| `strong`         | `*`, `_`              | `*`     |
| `fence`          | `` ` ``, `~`          | `` ` `` |

**Pros:**

- More formatting options than Prettier for markdown
- Minimal footprint (~50-80 lines for CLI wrapper)
- Can delegate embedded code formatting to Biome via custom plugin

**Cons:**

- Requires writing and maintaining custom tooling
- No existing Trunk linter definition (would need custom plugin)

### Option 4: remark-stringify + Biome for Code Blocks

Extend Option 3 with a custom remark plugin that formats embedded code blocks using Biome.

**Implementation approach:**

```javascript
import { visit } from "unist-util-visit";
import { execSync } from "child_process";

const BIOME_LANGS = ["js", "ts", "jsx", "tsx", "json", "css"];

function remarkBiomeFormat() {
  return (tree) => {
    visit(tree, "code", (node) => {
      if (node.lang && BIOME_LANGS.includes(node.lang)) {
        try {
          node.value = execSync(`biome format --stdin-file-path=file.${node.lang}`, {
            input: node.value,
            encoding: "utf-8",
          }).trim();
        } catch {
          // Keep original if Biome fails
        }
      }
    });
  };
}
```

**mdast code block structure:**

```javascript
{
  type: 'code',
  lang: 'typescript',
  meta: null,
  value: 'const x = 1;'
}
```

**Pros:**

- Unified formatting: Biome for all JS/TS/JSON/CSS, even inside markdown
- Full control over markdown formatting options
- Lightweight (~80-120 lines total)
- No Prettier dependency

**Cons:**

- Custom tooling to maintain
- Requires Trunk plugin definition
- Process spawning overhead for Biome calls (could optimize with Biome JS API)

### Option 5: remark-stringify + Multi-Formatter Dispatch

Extend Option 4 to support multiple formatters, selecting the appropriate tool based on code block language.

**Architecture:**

```text
markdown → remark-parse → mdast tree → [remark-format-code-blocks] → remark-stringify → formatted markdown
                                                  ↓
                                         formatter dispatch
                                        ┌─────────┴─────────┐
                                        ↓         ↓         ↓
                                      biome    shfmt    rustfmt ...
```

**Formatter mapping:**

| Language         | Formatter              | Notes                       |
| ---------------- | ---------------------- | --------------------------- |
| js, ts, jsx, tsx | biome                  | Primary JS/TS formatter     |
| json, jsonc      | biome                  | JSON with optional comments |
| css              | biome                  | CSS formatting              |
| graphql          | biome                  | GraphQL support             |
| sh, bash, zsh    | shfmt                  | Shell script formatting     |
| go               | gofmt                  | Go standard formatter       |
| rust             | rustfmt                | Rust standard formatter     |
| python, py       | ruff format / black    | Python formatting           |
| yaml, yml        | yamlfmt                | YAML formatting             |
| toml             | taplo                  | TOML formatting             |
| sql              | sql-formatter          | SQL formatting              |
| html             | biome / htmlbeautifier | HTML formatting             |
| xml              | xmllint --format       | XML formatting              |

**Implementation approach:**

```javascript
import { visit } from "unist-util-visit";
import { execSync } from "child_process";

const FORMATTERS = {
  js: { cmd: "biome format --stdin-file-path=file.js", stdin: true },
  ts: { cmd: "biome format --stdin-file-path=file.ts", stdin: true },
  tsx: { cmd: "biome format --stdin-file-path=file.tsx", stdin: true },
  jsx: { cmd: "biome format --stdin-file-path=file.jsx", stdin: true },
  json: { cmd: "biome format --stdin-file-path=file.json", stdin: true },
  css: { cmd: "biome format --stdin-file-path=file.css", stdin: true },
  graphql: {
    cmd: "biome format --stdin-file-path=file.graphql",
    stdin: true,
  },
  sh: { cmd: "shfmt", stdin: true },
  bash: { cmd: "shfmt", stdin: true },
  go: { cmd: "gofmt", stdin: true },
  rust: { cmd: "rustfmt", stdin: true },
  python: { cmd: "ruff format -", stdin: true },
  py: { cmd: "ruff format -", stdin: true },
  yaml: { cmd: "yamlfmt -", stdin: true },
  yml: { cmd: "yamlfmt -", stdin: true },
  toml: { cmd: "taplo fmt -", stdin: true },
  sql: { cmd: "sql-formatter", stdin: true },
};

function remarkFormatCodeBlocks(options = {}) {
  const formatters = { ...FORMATTERS, ...options.formatters };

  return (tree) => {
    visit(tree, "code", (node) => {
      const formatter = formatters[node.lang];
      if (!formatter) return; // No formatter configured, leave as-is

      try {
        node.value = execSync(formatter.cmd, {
          input: node.value,
          encoding: "utf-8",
          timeout: 5000,
        }).trim();
      } catch {
        // Keep original if formatter fails or not installed
      }
    });
  };
}
```

**Configuration file (`.md-fmt.json`):**

```json
{
  "formatters": {
    "js": {
      "cmd": "biome format --stdin-file-path=file.js",
      "stdin": true
    },
    "python": { "cmd": "black -", "stdin": true }
  },
  "disabled": ["sql"],
  "remarkStringify": {
    "bullet": "-",
    "emphasis": "_",
    "listItemIndent": "one"
  }
}
```

**Pros:**

- Formats all code blocks, not just Biome-supported languages
- Uses existing project formatters (consistent with project style)
- Graceful degradation: missing formatters are silently skipped
- Configurable per-project via config file
- Can leverage Trunk's formatter knowledge for consistency

**Cons:**

- More complex than Option 4 (~150-200 lines)
- Requires multiple formatters to be installed for full coverage
- Process spawning overhead for each code block (mitigated by parallel execution)
- Configuration management for formatter commands

### Option 6: Wait for Biome Markdown Support

Biome has an open issue ([#3718](https://github.com/biomejs/biome/issues/3718)) for markdown support.

**Current status (as of 2026-01-25):**

- Parser work started but stalled (PR #5292 closed, contributor dropped out)
- Grammar code generation merged
- No formatter or linter rules implemented
- Depends on #3334 for embedded code block formatting
- Timeline: months away at minimum

**Pros:**

- Would provide unified tooling
- No custom code to maintain

**Cons:**

- Not available now
- Uncertain timeline
- May not meet all formatting needs when released

## Existing Ecosystem Gaps

### Remark Plugins for Embedded Code Formatting

| Plugin                | Status     | Capability                                    |
| --------------------- | ---------- | --------------------------------------------- |
| unified-prettier      | Active     | Calls Prettier (uses its embedded formatting) |
| remark-prettier       | Deprecated | Replaced by unified-prettier                  |
| prettier-plugin-embed | Active     | Extends Prettier for SQL, XML, PHP, etc.      |

**Gap:** No existing remark plugin supports arbitrary formatters (like Biome) for embedded code blocks. A custom plugin would be required.

## Decision

**Proposed: Option 5 — remark-stringify + Multi-Formatter Dispatch**

Rationale:

1. Avoids Prettier dependency entirely
2. Provides more markdown formatting control than Prettier offers
3. Formats all code blocks using appropriate project formatters, not just Biome-supported languages
4. Consistent formatting: code blocks match project style (same formatters used everywhere)
5. Graceful degradation: missing formatters are silently skipped
6. Can be simplified to Option 4 (Biome-only) if multi-formatter complexity isn't needed
7. Can be deprecated when Biome adds markdown support

### Implementation Plan

1. Create `@pke/md-fmt` package or script in workspace
2. CLI interface: `md-fmt [files...]` or stdin/stdout
3. Configuration via `.md-fmt.json` for formatter overrides and remark-stringify options
4. Default formatter mapping for common languages (biome, shfmt, gofmt, rustfmt, ruff, etc.)
5. Trunk plugin definition for integration
6. Replace with Biome when markdown support ships

### Trunk Integration

Minimal `.trunk/trunk.yaml` addition:

```yaml
lint:
    definitions:
        - name: md-fmt
          files: [markdown]
          commands:
              - name: format
                output: rewrite
                run: md-fmt ${target}
                success_codes: [0]
    enabled:
        - md-fmt
```

## Consequences

### Positive

- All code blocks formatted, not just Biome-supported languages
- Uses existing project formatters for consistency
- More markdown formatting options than Prettier
- Focused tooling with clear scope
- Clear migration path to Biome when ready

### Neutral

- Requires ~150-200 lines of custom code
- New Trunk plugin definition needed
- Configuration file for formatter customization

### Negative

- Maintenance burden until Biome adds markdown support
- Process spawning for each code block (mitigated by parallelization)
- Requires formatters to be installed for full coverage
- Not as battle-tested as Prettier

## References

- [Biome Markdown Issue #3718](https://github.com/biomejs/biome/issues/3718)
- [remark-stringify options](https://github.com/remarkjs/remark/tree/main/packages/remark-stringify)
- [Prettier markdown options](https://prettier.io/docs/options)
- [unified ecosystem](https://unifiedjs.com/)
- [mdast specification](https://github.com/syntax-tree/mdast)

## Appendix: Prettier vs remark-stringify Feature Comparison

| Feature               | Prettier   | remark-stringify |
| --------------------- | ---------- | ---------------- |
| Prose wrapping        | Yes        | Yes              |
| Print width           | Yes        | Yes              |
| Tab width             | Yes        | Yes              |
| Bullet style          | No         | Yes              |
| List indent style     | No         | Yes              |
| Emphasis marker       | No         | Yes              |
| Strong marker         | No         | Yes              |
| Horizontal rule style | No         | Yes              |
| Quote style           | No         | Yes              |
| Fence marker          | No         | Yes              |
| Format embedded code  | Yes (auto) | Via plugin       |
