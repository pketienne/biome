# Plan 4: YAML-Specific Formatter Config Options

## Status: DEFERRED (lower priority, only enable/disable toggles needed for now)

## Context

The YAML formatter currently only supports generic options (indent_style, indent_width, line_ending, line_width). Adding YAML-specific options would allow finer control. This is lower priority and requires changes across multiple crates.

## Potential Options

- `quote_style`: Prefer single or double quotes for string scalars
- `trailing_newline`: Ensure file ends with newline
- `max_blank_lines`: Maximum consecutive blank lines (collapse extras)

## Files to Modify

1. `crates/biome_yaml_formatter/src/context.rs` — Add options to `YamlFormatOptions`
2. `crates/biome_configuration/src/yaml.rs` — Add config fields for deserialization
3. `crates/biome_service/src/file_handlers/yaml.rs` — Wire options through service layer
4. Individual formatter files that need to respect the new options

## Complexity

Medium-high. Each option requires:
- Config schema definition
- Deserialization support
- Plumbing through context
- Formatter logic changes
- Tests

## Verification
1. Unit tests for each option
2. CLI tests with biome.json config
