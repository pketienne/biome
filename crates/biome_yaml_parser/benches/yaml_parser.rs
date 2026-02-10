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
            "large_sequence",
            include_str!("fixtures/large_sequence.yaml"),
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
