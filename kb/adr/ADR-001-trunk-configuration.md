# ADR-001: Trunk Configuration Consolidation

**Status:** Accepted
**Date:** 2026-01-25
**Deciders:** pke
**Affects:** All projects in ~/Projects workspace

## Context

The workspace has two trunk-related directories:

- `.trunk/` — Trunk configuration for the Projects workspace itself
- `trunk/` — A Trunk plugin intended for sharing configuration across projects

Initially, `.trunk/trunk.yaml` contained its own linter configuration directly, duplicating what was defined in `trunk/plugin.yaml`. This created a maintenance burden where configuration changes needed to be made in multiple places.

Additionally, `trunk/plugin.yaml` had an incorrect `exported_configs` schema that prevented it from being used as a plugin.

## Options Considered

### Option 1: Keep Separate Configurations

Maintain independent configurations in each location.

**Pros:**

- No dependencies between directories
- Each can evolve independently

**Cons:**

- Duplicate configuration to maintain
- Drift between configurations over time
- Workspace doesn't validate its own shared plugin

### Option 2: Consolidate via Plugin Reference

Have `.trunk/trunk.yaml` reference `trunk/` as a local plugin, same as other projects would.

**Pros:**

- Single source of truth in `trunk/plugin.yaml`
- Workspace "eats its own dog food"
- Changes only need to happen in one place
- Validates that the shared plugin works

**Cons:**

- Workspace trunk config depends on `trunk/` directory existing

## Decision

**Use Option 2: Consolidate via Plugin Reference**

The workspace's `.trunk/trunk.yaml` now references the shared plugin:

```yaml
version: 0.1
cli:
    version: 1.22.8
plugins:
    sources:
        - id: trunk
          ref: v1.6.5
          uri: https://github.com/trunk-io/plugins
        - id: pke-config
          local: /home/pke/Projects/trunk
```

### Changes Made

1. **`.trunk/trunk.yaml`** — Replaced inline config with plugin reference
2. **`trunk/plugin.yaml`** — Fixed `exported_configs` schema (was list of strings, needed nested `configs:` key)
3. **`trunk/templates/trunk.yaml.template`** — Fixed path from `trunk-config` to `trunk`
4. **`trunk/README.md`** — Fixed path references

### Correct exported_configs Format

```yaml
lint:
    exported_configs:
        - configs:
              - configs/.markdownlint.json
              - configs/.editorconfig
```

## Consequences

### Positive

- Single source of truth for trunk configuration
- Workspace validates its own shared plugin
- New projects can copy the same plugin reference pattern
- Configuration changes propagate automatically

### Neutral

- Workspace depends on `trunk/` directory (acceptable, it's part of the workspace)

### Negative

- None identified

## Usage Pattern for New Projects

Projects wanting to use the shared configuration add to their `.trunk/trunk.yaml`:

```yaml
version: 0.1
cli:
    version: 1.22.8
plugins:
    sources:
        - id: trunk
          ref: v1.6.5
          uri: https://github.com/trunk-io/plugins
        - id: pke-config
          local: /home/pke/Projects/trunk

# Optional: project-specific overrides
lint:
    enabled:
        - biome@2.3.11 # Add project-specific linters
    disabled:
        - markdownlint # Disable if using biome for markdown (when supported)
```

## Shared Plugin Contents

The `trunk/plugin.yaml` provides:

| Category         | Items                                                               |
| ---------------- | ------------------------------------------------------------------- |
| Linters          | git-diff-check, markdownlint@0.42.0, shellcheck@0.10.0, shfmt@3.6.0 |
| Ignore patterns  | node_modules, .git, vendor, dist, build, .trunk                     |
| Exported configs | .markdownlint.json, .editorconfig                                   |
| Actions          | trunk-check-pre-commit, trunk-fmt-pre-commit                        |

## References

- [Trunk Plugin Documentation](https://docs.trunk.io/check/plugins)
- [Exporting Linter Configs](https://docs.trunk.io/check/plugins/exported-configs)
- [trunk-io/configs example](https://github.com/trunk-io/configs)
