use biome_yaml_formatter::{context::YamlFormatOptions, format_node};
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

// Jemallocator does not work on aarch64 with musl, so we'll use the system allocator instead
#[cfg(all(target_env = "musl", target_os = "linux", target_arch = "aarch64"))]
#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

fn yaml_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "simple_mapping",
            include_str!("fixtures/simple_mapping.yaml"),
        ),
        (
            "nested_structure",
            include_str!("fixtures/nested_structure.yaml"),
        ),
        (
            "flow_collections",
            include_str!("fixtures/flow_collections.yaml"),
        ),
        ("mixed_styles", include_str!("fixtures/mixed_styles.yaml")),
        (
            "kubernetes_deployment",
            include_str!("fixtures/kubernetes_deployment.yaml"),
        ),
        (
            "github_actions",
            include_str!("fixtures/github_actions.yaml"),
        ),
        (
            "docker_compose",
            include_str!("fixtures/docker_compose.yaml"),
        ),
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

        // Skip fixtures that have parse errors (formatter rejects them)
        if parsed.has_errors() {
            continue;
        }

        group.bench_with_input(
            BenchmarkId::new(*name, "format"),
            &parsed,
            |b, parsed| {
                b.iter(|| {
                    let formatted = format_node(options.clone(), &parsed.syntax())
                        .expect("formatting failed");
                    black_box(formatted.print().unwrap());
                })
            },
        );
    }
    group.finish();
}

criterion_group!(yaml_formatter, bench_formatter);
criterion_main!(yaml_formatter);
