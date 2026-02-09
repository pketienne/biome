use biome_analyze::{AnalysisFilter, AnalyzerOptions, ControlFlow, Never, RuleFilter};
use biome_diagnostics::{Diagnostic, DiagnosticExt, Severity, print_diagnostic_to_string};
use biome_turtle_analyze::analyze;
use biome_turtle_parser::parse_turtle;
use biome_turtle_syntax::TextRange;
use std::slice;

// use this test check if your snippet produces the diagnostics you wish, without using a snapshot
#[ignore]
#[test]
fn quick_test() {
    const FILENAME: &str = "dummyFile.ttl";
    const SOURCE: &str = r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<http://example.org/alice> a foaf:Person .
"#;

    let parsed = parse_turtle(SOURCE);

    let mut error_ranges: Vec<TextRange> = Vec::new();
    let options = AnalyzerOptions::default();
    let rule_filter = RuleFilter::Rule("nursery", "noDuplicatePrefixDeclaration");

    analyze(
        &parsed.tree(),
        AnalysisFilter {
            enabled_rules: Some(slice::from_ref(&rule_filter)),
            ..AnalysisFilter::default()
        },
        &options,
        |signal| {
            if let Some(diag) = signal.diagnostic() {
                error_ranges.push(diag.location().span.unwrap());
                let error = diag
                    .with_severity(Severity::Warning)
                    .with_file_path(FILENAME)
                    .with_file_source_code(SOURCE);
                let text = print_diagnostic_to_string(&error);
                eprintln!("{text}");
            }

            for action in signal.actions() {
                let new_code = action.mutation.commit();
                eprintln!("{new_code}");
            }

            ControlFlow::<Never>::Continue(())
        },
    );

    // assert_eq!(error_ranges.as_slice(), &[]);
}
