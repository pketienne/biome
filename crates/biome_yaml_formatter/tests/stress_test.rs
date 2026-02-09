use biome_formatter::QuoteStyle;
use biome_yaml_formatter::context::YamlFormatOptions;
use biome_yaml_formatter::format_node;
use biome_yaml_parser::parse_yaml;
use std::fs;
use std::path::Path;

fn stress_test_file(path: &Path) {
    let name = path.file_name().unwrap().to_str().unwrap();
    let src = fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {name}: {e}"));

    // Step 1: Parse
    let parse = parse_yaml(&src);
    let errors: Vec<_> = parse.diagnostics().iter().collect();
    // Note: some files may have parser errors for unsupported features; we log but don't fail
    if !errors.is_empty() {
        eprintln!("[{name}] Parser diagnostics: {}", errors.len());
        for err in &errors {
            eprintln!("  - {:?}", err);
        }
    }

    // Step 2: Format (default options — space indent)
    let options = YamlFormatOptions::default();
    let formatted = format_node(options.clone(), &parse.syntax())
        .unwrap_or_else(|e| panic!("[{name}] Format failed: {e}"));
    let result = formatted
        .print()
        .unwrap_or_else(|e| panic!("[{name}] Print failed: {e}"));
    let formatted_code = result.as_code();

    assert!(
        !formatted_code.is_empty(),
        "[{name}] Formatted output is empty"
    );

    // Step 3: Idempotency — format again, should produce identical output
    let parse2 = parse_yaml(formatted_code);
    let formatted2 = format_node(options, &parse2.syntax())
        .unwrap_or_else(|e| panic!("[{name}] Second format failed: {e}"));
    let result2 = formatted2
        .print()
        .unwrap_or_else(|e| panic!("[{name}] Second print failed: {e}"));
    let formatted_code2 = result2.as_code();

    if formatted_code != formatted_code2 {
        // Find first difference for debugging
        let lines1: Vec<&str> = formatted_code.lines().collect();
        let lines2: Vec<&str> = formatted_code2.lines().collect();
        for (i, (l1, l2)) in lines1.iter().zip(lines2.iter()).enumerate() {
            if l1 != l2 {
                panic!(
                    "[{name}] Idempotency failure at line {}:\n  pass 1: {:?}\n  pass 2: {:?}",
                    i + 1,
                    l1,
                    l2
                );
            }
        }
        if lines1.len() != lines2.len() {
            panic!(
                "[{name}] Idempotency failure: pass 1 has {} lines, pass 2 has {} lines",
                lines1.len(),
                lines2.len()
            );
        }
        panic!("[{name}] Idempotency failure (content differs but couldn't pinpoint line)");
    }

    // Step 4: Format with single quotes — should also be idempotent
    let options_single = YamlFormatOptions::default().with_quote_style(QuoteStyle::Single);
    let parse3 = parse_yaml(&src);
    let formatted3 = format_node(options_single.clone(), &parse3.syntax())
        .unwrap_or_else(|e| panic!("[{name}] Single-quote format failed: {e}"));
    let result3 = formatted3
        .print()
        .unwrap_or_else(|e| panic!("[{name}] Single-quote print failed: {e}"));
    let single_code = result3.as_code();

    let parse4 = parse_yaml(single_code);
    let formatted4 = format_node(options_single, &parse4.syntax())
        .unwrap_or_else(|e| panic!("[{name}] Single-quote second format failed: {e}"));
    let result4 = formatted4
        .print()
        .unwrap_or_else(|e| panic!("[{name}] Single-quote second print failed: {e}"));

    assert_eq!(
        single_code,
        result4.as_code(),
        "[{name}] Single-quote idempotency failure"
    );

    eprintln!(
        "[{name}] OK — {} bytes input, {} bytes formatted, {} parser diagnostics",
        src.len(),
        formatted_code.len(),
        errors.len()
    );
}

fn stress_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/stress")
        .join(name)
}

#[test]
fn stress_github_actions() {
    stress_test_file(&stress_path("github_actions.yaml"));
}

#[test]
fn stress_kubernetes() {
    stress_test_file(&stress_path("kubernetes.yaml"));
}

#[test]
fn stress_docker_compose() {
    stress_test_file(&stress_path("docker_compose.yaml"));
}

#[test]
fn stress_anchors_complex() {
    stress_test_file(&stress_path("anchors_complex.yaml"));
}

#[test]
fn stress_edge_cases() {
    stress_test_file(&stress_path("edge_cases.yaml"));
}

#[test]
fn stress_helm_values() {
    stress_test_file(&stress_path("helm_values.yaml"));
}

#[test]
fn stress_multi_document() {
    stress_test_file(&stress_path("multi_document.yaml"));
}
