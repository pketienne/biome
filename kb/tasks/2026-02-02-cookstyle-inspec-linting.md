# Task: Cookstyle Integration for InSpec DSL Linting

**Date:** 2026-02-02

## Problem

When running `trunk check` on InSpec compliance files, RuboCop was failing because it doesn't understand the InSpec DSL (Domain Specific Language). The errors occurred because:

1. Trunk copies files to `/tmp/trunk-*` directories for linting
2. mise doesn't trust `.mise.toml` files in `/tmp` (security feature - `/tmp` is world-writable)
3. Even if mise trusted the files, standard RuboCop doesn't understand InSpec-specific constructs like `control`, `describe`, `only_if`, etc.

Initial workaround was to ignore InSpec files from RuboCop entirely, but this meant no linting at all for compliance code.

## Solution

Integrated **Cookstyle** - a RuboCop-based linter specifically designed for Chef and InSpec code. Cookstyle understands the Chef/InSpec DSL and provides appropriate linting rules.

## Implementation

### 1. trunk-pke Plugin Updates (v1.5.0 → v1.5.1)

**Added cookstyle linter definition** in `plugin.yaml`:

```yaml
- name: cookstyle
  description: Chef/InSpec style linter (RuboCop-based)
  files: [ruby]
  suggest_if: never
  commands:
    - name: lint
      output: sarif
      run: /opt/cinc-workstation/bin/cookstyle --format json ${target}
      success_codes: [0, 1]
      batch: true
      parser:
        runtime: python
        run: python3 ${plugin}/configs/cookstyle_to_sarif.py
    - name: fix
      output: rewrite
      run: /opt/cinc-workstation/bin/cookstyle --auto-correct ${target}
      success_codes: [0, 1]
      in_place: true
      batch: true
      formatter: true
  direct_configs: [.rubocop.yml]
  issue_url_format: https://docs.chef.io/workstation/cookstyle/
  known_good_version: 7.32.8
  version_command:
    parse_regex: "Cookstyle ${semver}"
    run: /opt/cinc-workstation/bin/cookstyle --version
```

**Key configuration details:**

The `parser` block (lines 36-38 in the YAML above) is critical:

```yaml
parser:
  runtime: python
  run: python3 ${plugin}/configs/cookstyle_to_sarif.py
```

- `${plugin}` - Trunk variable that resolves to the plugin's installation directory (e.g., `~/.cache/trunk/plugins/https---github-com-pketienne-trunk-pke/v1.5.1-xxx/`)
- Using `${plugin}` ensures the parser is found regardless of which project is being linted
- If we used `${cwd}` instead, each project would need its own copy of the parser script

**Data flow:**

1. Trunk runs cookstyle with `--format json` on the target file(s)
2. Cookstyle's JSON output is piped to stdin of the parser script
3. The parser reads JSON, converts to SARIF format, writes to stdout
4. Trunk reads the SARIF output and displays issues in its UI

**Created SARIF parser** at `configs/cookstyle_to_sarif.py`:

Cookstyle outputs JSON in the same format as RuboCop, so the parser converts this to SARIF format for Trunk integration. The parser maps RuboCop/Cookstyle severity levels to SARIF levels:

| Cookstyle Severity         | SARIF Level |
| -------------------------- | ----------- |
| convention, refactor, info | note        |
| warning                    | warning     |
| error, fatal               | error       |

**Bug fix (v1.5.1):** Changed `--autocorrect` to `--auto-correct` (cookstyle uses hyphen).

### 2. Symmetra trunk.yaml Configuration

```yaml
lint:
  enabled:
    - cookstyle  # Use cookstyle for Chef/InSpec files

  ignore:
    - linters: [ALL]
      paths:
        - .chefctl/**
    # Rubocop doesn't understand InSpec DSL - use cookstyle for compliance files
    - linters: [rubocop]
      paths:
        - cookbooks/*/compliance/**
        - scripts/**
    # Cookstyle is specifically for Chef/InSpec - don't run on other Ruby files
    - linters: [cookstyle]
      paths:
        - scripts/**
```

**Rationale:**

- RuboCop: Ignores compliance files (doesn't understand InSpec DSL)
- Cookstyle: Handles compliance files (understands Chef/InSpec DSL)
- Scripts: Use RuboCop only (standard Ruby, not Chef/InSpec)

### 3. Auto-fixed Compliance Files

Ran `trunk fmt` on all 39 compliance control files. Primary fix was removing extra blank lines (cookstyle's `Layout/EmptyLines` rule).

## Files Changed

**trunk-pke repository:**

- `plugin.yaml` - Added cookstyle linter definition
- `configs/cookstyle_to_sarif.py` - New SARIF parser

**symmetra repository:**

- `.trunk/trunk.yaml` - Enable cookstyle, configure ignore rules
- `cookbooks/*/compliance/profiles/default/controls/default.rb` - 37 files auto-formatted

## Commits

**trunk-pke:**

- `6ffe694` Add cookstyle linter for Chef/InSpec DSL support (v1.5.0)
- `99a48e6` Fix cookstyle auto-correct flag (v1.5.1)

## Testing

Verified the integration works:

```bash
# Check cookstyle version
/opt/cinc-workstation/bin/cookstyle --version
# Output: Cookstyle 7.32.8

# Test JSON output
/opt/cinc-workstation/bin/cookstyle --format json <file> | python3 configs/cookstyle_to_sarif.py

# Test trunk integration
trunk check cookbooks/bash/compliance/profiles/default/controls/default.rb
# Output: ✔ No issues (after auto-fix)
```

## Key Learnings

1. **Cookstyle vs RuboCop:** Cookstyle is RuboCop with Chef-specific cops. It understands InSpec DSL constructs that would confuse standard RuboCop.

2. **Trunk plugin paths:** Use `${plugin}` to reference files within the plugin directory (e.g., `${plugin}/configs/cookstyle_to_sarif.py`).

3. **Cookstyle flags:** Uses `--auto-correct` (with hyphen), not `--autocorrect` like some RuboCop versions.

4. **Linter coexistence:** Configure ignore rules so RuboCop and Cookstyle each handle appropriate file types without overlap.

## References

- [Cookstyle Documentation](https://docs.chef.io/workstation/cookstyle/)
- [Trunk Plugin Development](https://docs.trunk.io/plugins)
- [InSpec DSL Reference](https://docs.chef.io/inspec/dsl_inspec/)
