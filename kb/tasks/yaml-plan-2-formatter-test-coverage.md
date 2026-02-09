# Plan 2: Formatter Test Coverage

## Status: COMPLETE

## Context

The formatter has 17 snapshot tests but is missing coverage for several important patterns. Adding these tests ensures the formatter handles edge cases correctly and catches regressions.

## New Test Specs

### 2A. `document/multiple.yaml`
```yaml
---
doc1: value1
...
---
doc2: value2
...
```

### 2B. `mapping/complex.yaml`
```yaml
simple: value
nested:
  level2:
    level3: deep
  sibling: here
mixed_flow: {a: 1, b: [2, 3]}
empty_value:
list_of_maps:
  - name: first
    value: 1
  - name: second
    value: 2
```

### 2C. `scalar/edge_cases.yaml`
```yaml
empty_double: ""
empty_single: ''
special_chars: "hello\nworld"
unicode: "cafe\u0301"
long_plain: this is a somewhat long plain scalar value that tests basic formatting
number_like: "123"
bool_like: "true"
null_like: "null"
```

## Files to Create
- `crates/biome_yaml_formatter/tests/specs/yaml/document/multiple.yaml`
- `crates/biome_yaml_formatter/tests/specs/yaml/mapping/complex.yaml`
- `crates/biome_yaml_formatter/tests/specs/yaml/scalar/edge_cases.yaml`

## Verification
1. `cargo test -p biome_yaml_formatter` — all tests pass
2. Review snapshots for correctness
