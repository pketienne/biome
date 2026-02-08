# YAML Formatting and Linting Tools Audit

## Background

The `ChecklistSync` class generates YAML checklist files from markdown sources. These files were failing `trunk check` due to formatting differences between Ruby's `YAML.dump` output and yamlfmt's expected style.

### Problem

Ruby's `YAML.dump` outputs array items flush with their parent key:

```yaml
checks:
- id: STR-001
  title: foo
  check:
    paths:
    - recipes/*.rb
```

yamlfmt expects array items indented 2 spaces under the parent:

```yaml
checks:
  - id: STR-001
    title: foo
    check:
      paths:
        - recipes/*.rb
```

### Solution

Added `indent_yaml_arrays` post-processor to `ChecklistSync` that:

1. Tracks array nesting levels using a stack
2. Applies +2 spaces of indentation at each array level
3. Handles nested arrays with cumulative adjustments

### Related Files

- `lib/symmetra/audit/checklist_sync.rb` - Generator with YAML post-processing
- `lib/symmetra/audit/checklists/*.yml` - Generated checklist files
- `.trunk/trunk.yaml` - Trunk configuration enabling yamlfmt
- `/home/pke/Projects/trunk-pke/configs/.yamlfmt` - yamlfmt configuration

### Trunk Plugin Issue

The trunk-pke plugin's cached configs in `.trunk/plugins/trunk-pke/configs/` were missing `.yamlfmt` and `.yamllint.yaml`. These had to be manually copied from the source plugin directory. This suggests the plugin export mechanism may have issues with dotfiles or the cache wasn't refreshed after adding new configs.

---

## TODO: Audit All Linting and Formatting Tools

Perform a comprehensive audit of all linting and formatting tools and rules configured in this repository.

### Scope

1. **Trunk Configuration** (`.trunk/trunk.yaml`)
   - List all enabled linters and formatters
   - Document version pins and update policy
   - Review ignore patterns for appropriateness

2. **trunk-pke Plugin** (`/home/pke/Projects/trunk-pke/`)
   - Audit all exported configs in `configs/`
   - Verify all configs are properly cached in consumer projects
   - Document custom linter definitions (cookstyle, dprint, serdi)
   - Review action triggers (pre-commit, pre-push hooks)

3. **Per-Tool Configuration**
   - `.yamlfmt` - YAML formatting rules
   - `.yamllint.yaml` - YAML linting rules
   - `.rubocop.yml` - Ruby style rules
   - `.markdownlint.json` - Markdown linting rules
   - `biome.json` - JS/TS/JSON formatting
   - `dprint.json` - Markdown formatting
   - `.taplo.toml` - TOML formatting
   - `rustfmt.toml` - Rust formatting
   - `deny.toml` - Cargo dependency checks

4. **Tool Interactions**
   - Identify any conflicts between tools (e.g., prettier vs biome)
   - Document which tools handle which file types
   - Verify no gaps in coverage for project file types

5. **Auto-generation Considerations**
   - Identify all auto-generated files in the project
   - Ensure linter configs handle generated code appropriately
   - Document any files that need ignore rules

### Deliverables

- [ ] Inventory of all linting/formatting tools with versions
- [ ] Matrix of file types to responsible tools
- [ ] List of configuration inconsistencies or gaps
- [ ] Recommendations for simplification or consolidation
- [ ] Updated documentation in CLAUDE.md or similar
