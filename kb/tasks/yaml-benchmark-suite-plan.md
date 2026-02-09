# Benchmark Suite for YAML Parser/Formatter

## Date: 2026-02-09

## Problem

There are no benchmarks for the YAML parser or formatter. As the codebase grows with more
features (escape validation, tab diagnostics, flow-to-block conversion), we need baseline
performance measurements and regression detection. Other Biome language crates (JS, CSS)
have criterion-based benchmark suites that we should follow as a template.

## Reference: JS Parser Benchmark Pattern

File: `crates/biome_js_parser/benches/js_parser.rs`

Key patterns:
- Uses `codspeed-criterion-compat` v3.0.5 (aliased as `criterion` in Cargo.toml)
- Platform-specific global allocator: jemalloc (linux/macOS), mimalloc (Windows), system (musl aarch64)
- `BenchCase` from `biome_test_utils` for loading test files from `libs-*.txt`
- Two benchmark variants: **uncached** (fresh parse) and **cached** (`NodeCache` reuse)
- `Throughput::Bytes` for MB/s reporting
- `criterion_group!` / `criterion_main!` macros
- `[[bench]]` section in `Cargo.toml` with `harness = false`

## Implementation

### 1. Parser Benchmark

#### File: `crates/biome_yaml_parser/Cargo.toml`

Add benchmark configuration and dependencies:

```toml
[[bench]]
harness = false
name    = "yaml_parser"

[dev-dependencies]
# ... existing dev-dependencies ...
criterion = { package = "codspeed-criterion-compat", version = "=3.0.5" }
```

Also add allocator dependencies (following JS pattern):

```toml
[target.'cfg(windows)'.dev-dependencies]
mimalloc = "0.1.43"

[target.'cfg(all(any(target_os = "macos", target_os = "linux"), not(target_env = "musl")))'.dev-dependencies]
tikv-jemallocator = "0.6.0"
```

> **Note**: Check if these allocator dependencies are already in the workspace `Cargo.toml`.
> If so, use `workspace = true` syntax. If not available in workspace, use direct version
> pinning as shown above, or add to workspace first.

#### File: `crates/biome_yaml_parser/benches/yaml_parser.rs` (new)

```rust
use biome_rowan::NodeCache;
use biome_yaml_parser::{parse_yaml, parse_yaml_with_cache};
use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    not(target_env = "musl"),
))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(all(target_env = "musl", target_os = "linux", target_arch = "aarch64"))]
#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

/// Inline YAML test cases with varying complexity.
/// Using inline strings avoids external file dependencies and makes benchmarks self-contained.
fn yaml_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("simple_mapping", include_str!("fixtures/simple_mapping.yaml")),
        ("nested_structure", include_str!("fixtures/nested_structure.yaml")),
        ("large_sequence", include_str!("fixtures/large_sequence.yaml")),
        ("flow_collections", include_str!("fixtures/flow_collections.yaml")),
        ("mixed_styles", include_str!("fixtures/mixed_styles.yaml")),
    ]
}

fn bench_parser(criterion: &mut Criterion) {
    let test_cases = yaml_test_cases();

    let mut group = criterion.benchmark_group("yaml_parser");
    for (name, code) in &test_cases {
        group.throughput(Throughput::Bytes(code.len() as u64));

        // Uncached: fresh parse each iteration
        group.bench_with_input(
            BenchmarkId::new(*name, "uncached"),
            code,
            |b, code| {
                b.iter(|| {
                    black_box(parse_yaml(code));
                })
            },
        );

        // Cached: reuse NodeCache across iterations
        group.bench_with_input(
            BenchmarkId::new(*name, "cached"),
            code,
            |b, code| {
                b.iter_batched(
                    || {
                        let mut cache = NodeCache::default();
                        parse_yaml_with_cache(code, &mut cache);
                        cache
                    },
                    |mut cache| {
                        black_box(parse_yaml_with_cache(code, &mut cache));
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

criterion_group!(yaml_parser, bench_parser);
criterion_main!(yaml_parser);
```

#### Benchmark Fixture Files

Create `crates/biome_yaml_parser/benches/fixtures/` with representative YAML files:

**`simple_mapping.yaml`** (~20 lines): Basic key-value pairs
```yaml
name: project
version: 1.0.0
description: A sample project
author: John Doe
license: MIT
homepage: https://example.com
repository: https://github.com/example/project
keywords:
  - yaml
  - parser
  - benchmark
```

**`nested_structure.yaml`** (~80 lines): Deeply nested mappings and sequences
```yaml
server:
  host: localhost
  port: 8080
  ssl:
    enabled: true
    cert: /path/to/cert.pem
    key: /path/to/key.pem
  logging:
    level: info
    format: json
    outputs:
      - type: file
        path: /var/log/app.log
        rotation:
          max_size: 10MB
          max_files: 5
      - type: stdout
        colorize: true
database:
  primary:
    host: db.example.com
    port: 5432
    name: mydb
    pool:
      min: 5
      max: 20
      timeout: 30
  replicas:
    - host: replica1.example.com
      port: 5432
      weight: 3
    - host: replica2.example.com
      port: 5432
      weight: 1
```

**`large_sequence.yaml`** (~200 lines): Long list of items (generated with repetition)

**`flow_collections.yaml`** (~50 lines): Flow sequences and mappings
```yaml
colors: [red, green, blue, yellow, cyan, magenta]
config: {debug: true, verbose: false, timeout: 30}
matrix:
  - [1, 0, 0]
  - [0, 1, 0]
  - [0, 0, 1]
nested_flow: {a: [1, 2], b: [3, 4], c: {x: 1, y: 2}}
```

**`mixed_styles.yaml`** (~100 lines): Block scalars, quoted strings, anchors/aliases
```yaml
description: |
  This is a block scalar
  that spans multiple lines
  and preserves newlines.
summary: >
  This is a folded scalar
  that wraps into a single
  paragraph.
quoted: "hello\nworld"
single_quoted: 'no escapes here'
anchor: &defaults
  timeout: 30
  retries: 3
usage:
  <<: *defaults
  endpoint: /api/v1
```

### 2. Formatter Benchmark

#### File: `crates/biome_yaml_formatter/Cargo.toml`

Add benchmark configuration:

```toml
[[bench]]
harness = false
name    = "yaml_formatter"

[dev-dependencies]
# ... existing dev-dependencies ...
criterion = { package = "codspeed-criterion-compat", version = "=3.0.5" }
```

Add allocator target dependencies (same pattern as parser).

#### File: `crates/biome_yaml_formatter/benches/yaml_formatter.rs` (new)

```rust
use biome_formatter::Printed;
use biome_yaml_formatter::{YamlFormatOptions, format_node};
use biome_yaml_parser::parse_yaml;
use criterion::{
    BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    not(target_env = "musl"),
))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(all(target_env = "musl", target_os = "linux", target_arch = "aarch64"))]
#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

fn yaml_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("simple_mapping", include_str!("fixtures/simple_mapping.yaml")),
        ("nested_structure", include_str!("fixtures/nested_structure.yaml")),
        ("flow_collections", include_str!("fixtures/flow_collections.yaml")),
        ("mixed_styles", include_str!("fixtures/mixed_styles.yaml")),
    ]
}

fn bench_formatter(criterion: &mut Criterion) {
    let test_cases = yaml_test_cases();
    let options = YamlFormatOptions::default();

    let mut group = criterion.benchmark_group("yaml_formatter");
    for (name, code) in &test_cases {
        // Pre-parse so we benchmark formatting only (not parsing)
        let parsed = parse_yaml(code);

        group.throughput(Throughput::Bytes(code.len() as u64));

        group.bench_with_input(
            BenchmarkId::new(*name, "format"),
            &parsed,
            |b, parsed| {
                b.iter(|| {
                    let formatted = format_node(options.clone(), &parsed.syntax())
                        .expect("formatting failed");
                    let _printed: Printed = black_box(formatted.print().unwrap());
                })
            },
        );
    }
    group.finish();
}

criterion_group!(yaml_formatter, bench_formatter);
criterion_main!(yaml_formatter);
```

The formatter benchmark fixtures can be symlinked from the parser fixtures or duplicated.
Using `include_str!` with shared paths:

```
crates/biome_yaml_formatter/benches/fixtures/ → symlink or copy of parser fixtures
```

Alternatively, create a shared `benches/fixtures/` directory at the workspace level and
reference via relative path in `include_str!`.

### 3. Running Benchmarks

```bash
# Run parser benchmarks
cargo bench -p biome_yaml_parser

# Run formatter benchmarks
cargo bench -p biome_yaml_formatter

# Run a specific benchmark
cargo bench -p biome_yaml_parser -- yaml_parser/simple_mapping

# Save baseline for comparison
cargo bench -p biome_yaml_parser -- --save-baseline main

# Compare against baseline
cargo bench -p biome_yaml_parser -- --baseline main
```

## Fixture Design Principles

1. **Representative**: Cover block mappings, block sequences, flow collections, scalars,
   quoted strings, block scalars, anchors/aliases, nested structures
2. **Varied size**: Small (~20 lines), medium (~80 lines), large (~200 lines) to catch
   both per-item and per-file overhead
3. **Self-contained**: Use `include_str!` with fixture files in `benches/fixtures/` —
   no external downloads or `libs-*.txt` needed (YAML ecosystem doesn't have "standard
   libraries" to benchmark against like JS/TS do)
4. **Stable**: Fixture content should not change between runs. Pin exact content.

## CI Integration

The existing Biome CI uses `codspeed-criterion-compat` which integrates with
[CodSpeed](https://codspeed.io/) for automatic performance tracking. By using the same
crate, YAML benchmarks will automatically appear in CodSpeed dashboards alongside JS/CSS
benchmarks once the PR is merged.

No additional CI configuration is needed — the `[[bench]]` sections in `Cargo.toml` are
automatically discovered by `cargo bench` and CodSpeed's runner.

## Verification

1. `cargo build -p biome_yaml_parser` — compiles with new bench target
2. `cargo build -p biome_yaml_formatter` — compiles with new bench target
3. `cargo bench -p biome_yaml_parser` — runs parser benchmarks, reports throughput
4. `cargo bench -p biome_yaml_formatter` — runs formatter benchmarks, reports throughput
5. Verify output includes MB/s throughput numbers
6. Verify uncached vs cached parser shows improvement (cached should be faster)
7. All existing tests still pass

## Files Summary

| File | Action |
|------|--------|
| `crates/biome_yaml_parser/Cargo.toml` | Add `[[bench]]`, criterion + allocator dev-deps |
| `crates/biome_yaml_parser/benches/yaml_parser.rs` | New parser benchmark |
| `crates/biome_yaml_parser/benches/fixtures/*.yaml` | New benchmark fixture files (5 files) |
| `crates/biome_yaml_formatter/Cargo.toml` | Add `[[bench]]`, criterion + allocator dev-deps |
| `crates/biome_yaml_formatter/benches/yaml_formatter.rs` | New formatter benchmark |
| `crates/biome_yaml_formatter/benches/fixtures/*.yaml` | Benchmark fixtures (shared or symlinked) |
