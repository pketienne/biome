# MD060 Emoji Table Formatting Research

<!-- markdownlint-disable MD013 -->

**Date:** 2026-01-31
**Status:** Research Complete

## Problem Statement

When using dprint to format markdown files and markdownlint to lint them, tables containing emoji characters trigger false-positive MD060 (Table column style) violations. This occurs because markdownlint counts unicode codepoints rather than display width, and some emojis (like ⚠️) consist of multiple codepoints (base character + variation selector).

## Current Stack

| Tool         | Role      | Issue                                    |
| ------------ | --------- | ---------------------------------------- |
| dprint       | Formatter | Formats correctly, no unicode width calc |
| markdownlint | Linter    | MD060 miscounts emoji display width      |

## Root Cause

- Emoji characters have variable byte/codepoint counts
- Example: ⚠️ = U+26A0 (base) + U+FE0F (variation selector) = 2 codepoints
- markdownlint counts codepoints, not rendered display width
- Tables appear aligned visually but fail MD060 validation

## Tools Researched

### Formatters

| Formatter    | Unicode Handling | Notes                                                                                               |
| ------------ | ---------------- | --------------------------------------------------------------------------------------------------- |
| **dprint**   | No wcwidth       | Formats structure, not width-aware                                                                  |
| **Prettier** | Broken           | [Issue #15664](https://github.com/prettier/prettier/issues/15664) - emoji tables misaligned         |
| **mdformat** | Uses wcwidth     | Python-based, [mdformat-gfm](https://github.com/hukkin/mdformat-gfm) has wcwidth ≥0.2.13 dependency |

### Linters

| Linter       | VS Code Extension                                                                                         | Notes                               |
| ------------ | --------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| markdownlint | [vscode-markdownlint](https://marketplace.visualstudio.com/items?itemName=DavidAnson.vscode-markdownlint) | Current - has emoji issues          |
| remark-lint  | [vscode-remark](https://marketplace.visualstudio.com/items?itemName=unifiedjs.vscode-remark)              | Plugin-based, modular rules         |
| textlint     | vscode-textlint                                                                                           | Japanese-origin, better CJK support |

### Supporting Libraries

- **[wcwidth](https://github.com/jquast/wcwidth)** - Python library that correctly measures unicode display width including emoji
- Used by mdformat-gfm for table column alignment

## Solutions Evaluated

### 1. Disable MD060 Globally

**Rejected** - Need MD060 to catch real table formatting issues in other files.

### 2. Inline Disable Comments

**Adopted** - Add `<!-- markdownlint-disable MD060 -->` before tables with emoji.

```markdown
<!-- markdownlint-disable MD060 -->

| Status | Description |
| ------ | ----------- |
| ⚠️      | Warning     |
| ✓      | Success     |

<!-- markdownlint-enable MD060 -->
```

### 3. Replace Emojis with Text

**Not preferred** - Reduces visual clarity of documentation.

### 4. Switch to mdformat

**Future consideration** - Would require:

- Adding mdformat as trunk formatter
- Testing wcwidth handling
- Possible [Issue #16](https://github.com/executablebooks/mdformat-tables/issues/16) edge cases

### 5. Pre-lint Filter

**Impractical** - Temporarily replacing emojis before linting, then reverting.

## Current Workaround

For files with emoji in tables (e.g., `git-as-versioning-strategy.md`):

1. Add inline disable comment before the table
2. Optionally re-enable after the table

## Recommendations

1. **Short-term**: Use inline MD060 disables for emoji tables
2. **Medium-term**: Monitor mdformat-gfm development for better wcwidth support
3. **Long-term**: File issue with markdownlint for proper unicode display width handling

## Related Issues

- [Prettier #15664](https://github.com/prettier/prettier/issues/15664) - Markdown tables with unicode emojis incorrectly formatted
- [mdformat-tables #16](https://github.com/executablebooks/mdformat-tables/issues/16) - Two-width character handling
- [vscode-markdown #151](https://github.com/yzhang-gh/vscode-markdown/issues/151) - Table formatting with combining characters
- [vscode-markdown-table #29](https://github.com/takumisoft68/vscode-markdown-table/issues/29) - Emoji characters break table layout

## Files Affected

- `kb/Projects/tasks/git-as-versioning-strategy.md` - Contains emoji rating tables
