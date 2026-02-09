# Plan 12: Compact Block Sequence Form

## Status: PENDING (depends on Plan 11)

## Context

When a block sequence entry contains a mapping, the formatter outputs the expanded form:
```yaml
-
  key: value
  other: value2
```

The standard YAML convention (used in GitHub Actions, Kubernetes, docker-compose) is the compact form:
```yaml
- key: value
  other: value2
```

### Why it depends on Plan 11

The compact form uses `align(2, ...)` from the Biome formatter, which aligns continuation lines with 2 spaces (matching `- ` width). This works correctly with **space indentation** but breaks with **tab indentation** because:

1. `align(2)` adds 2 spaces for direct children
2. `block_indent` inside `align(2)` with tabs adds a full tab (1 character in column tracking)
3. The YAML parser sees 2-space-aligned entries and 1-tab-indented nested entries at the same column, causing structural misinterpretation
4. The formatter output is not idempotent with tabs

With Plan 11 (default to spaces), the compact form works correctly because `align(2)` and `indent` both use consistent space-based indentation.

## Changes

### 12A. Use align(2) for block mapping values in sequence entries

**File**: `crates/biome_yaml_formatter/src/yaml/auxiliary/block_sequence_entry.rs`

Replace the `YamlBlockMapping` arm in `fmt_fields`:

```rust
// Before (expanded form):
AnyYamlBlockInBlockNode::YamlBlockMapping(mapping) => {
    f.comments().mark_suppression_checked(mapping.syntax());
    if let Some(properties) = mapping.properties() {
        write!(f, [space(), properties.format()])?;
    }
    write!(
        f,
        [hard_line_break(), block_indent(&format_with(|f| {
            write!(f, [format_synthetic_token(&mapping.mapping_start_token()?)])?;
            write!(f, [mapping.entries().format()])?;
            write!(f, [format_synthetic_token(&mapping.mapping_end_token()?)])
        }))]
    )?;
}

// After (compact form):
AnyYamlBlockInBlockNode::YamlBlockMapping(mapping) => {
    f.comments().mark_suppression_checked(mapping.syntax());
    write!(f, [format_synthetic_token(&mapping.mapping_start_token()?)])?;
    if let Some(properties) = mapping.properties() {
        write!(f, [space(), properties.format()])?;
    }
    write!(
        f,
        [align(2, &format_with(|f| {
            write!(f, [space(), mapping.entries().format()])?;
            write!(f, [format_synthetic_token(&mapping.mapping_end_token()?)])
        }))]
    )?;
}
```

How `align(2, ...)` works:
- First entry appears inline after `- ` (no line break before it)
- Subsequent entries get 2-space alignment on new lines
- With space indent_width=2: `align(2)` = 2 spaces, `block_indent` inside = 4 spaces total
- Nested mappings inside entries use normal `block_indent`, which adds `indent_width` more spaces

### 12B. Add comprehensive formatter test

**File**: `crates/biome_yaml_formatter/tests/specs/yaml/sequence/compact_nested.yaml`

```yaml
steps:
  - uses: actions/checkout@v4
  - uses: actions/setup-node@v4
    with:
      node-version: 18
  - run: npm ci
  - run: npm test
list_of_maps:
  - name: first
    value: 1
  - name: second
    value: 2
    details:
      type: important
      priority: high
```

## Verification
1. `cargo test -p biome_yaml_formatter` — all tests pass, accept snapshot updates
2. Verify idempotency: formatted output re-formatted produces identical result
3. Manual test with real-world YAML (GitHub Actions, K8s manifests)
4. Verify nested mappings inside compact entries are correctly indented

## Known Limitation
- Only works with space indentation (default after Plan 11)
- With explicit `indentStyle: "tab"`, the expanded form would be more correct, but since tabs violate the YAML spec, this is acceptable
