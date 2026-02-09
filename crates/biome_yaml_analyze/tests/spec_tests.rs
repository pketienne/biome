use biome_analyze::{AnalysisFilter, ControlFlow, Never, RuleFilter};
use biome_diagnostics::advice::CodeSuggestionAdvice;
use biome_rowan::AstNode;
use biome_test_utils::{
    CheckActionType, assert_diagnostics_expectation_comment, code_fix_to_string,
    create_analyzer_options, diagnostic_to_string, parse_test_path, register_leak_checker,
    write_analyzer_snapshot,
};
use std::ops::Deref;
use biome_yaml_parser::parse_yaml;
use biome_yaml_syntax::YamlLanguage;
use camino::Utf8Path;
use std::{fs::read_to_string, slice};

tests_macros::gen_tests! {"tests/specs/**/*.yaml", crate::run_test, "module"}

fn run_test(input: &'static str, _: &str, _: &str, _: &str) {
    register_leak_checker();

    let input_file = Utf8Path::new(input);
    let file_name = input_file.file_name().unwrap();

    if file_name.starts_with("_ignore") {
        return;
    }

    let (group, rule) = parse_test_path(input_file);
    if rule == "specs" {
        panic!("the test file must be placed in the specs/<group-name>/<rule-name>/ directory");
    }
    if group == "specs" {
        panic!("the test file must be placed in the specs/{rule}/<rule-name>/ directory");
    }

    if biome_yaml_analyze::METADATA
        .deref()
        .find_rule(group, rule)
        .is_none()
    {
        panic!("could not find rule {group}/{rule}");
    }

    let rule_filter = RuleFilter::Rule(group, rule);
    let filter = AnalysisFilter {
        enabled_rules: Some(slice::from_ref(&rule_filter)),
        ..AnalysisFilter::default()
    };

    let mut snapshot = String::new();

    let input_code = read_to_string(input_file)
        .unwrap_or_else(|err| panic!("failed to read {input_file:?}: {err:?}"));

    analyze_and_snap(
        &mut snapshot,
        &input_code,
        filter,
        file_name,
        input_file,
        CheckActionType::Lint,
    );

    insta::with_settings!({
        prepend_module_to_snapshot => false,
        snapshot_path => input_file.parent().unwrap(),
    }, {
        insta::assert_snapshot!(file_name, snapshot, file_name);
    });
}

pub(crate) fn analyze_and_snap(
    snapshot: &mut String,
    input_code: &str,
    filter: AnalysisFilter,
    file_name: &str,
    input_file: &Utf8Path,
    action_type: CheckActionType,
) {
    let mut diagnostics = Vec::new();
    let parsed = parse_yaml(input_code);
    if !parsed.diagnostics().is_empty() {
        for diag in parsed.diagnostics() {
            let formatted = diagnostic_to_string(file_name, input_code, diag.clone().into());
            diagnostics.push(formatted);
        }
    }
    let root = parsed.tree();

    let mut code_fixes = Vec::new();
    let options = create_analyzer_options::<YamlLanguage>(input_file, &mut diagnostics);
    let (_, errors) = biome_yaml_analyze::analyze(&root, filter, &options, |event| {
        if let Some(mut diag) = event.diagnostic() {
            for action in event.actions() {
                if action.is_suppression() {
                    if action_type.is_suppression() {
                        diag = diag.add_code_suggestion(CodeSuggestionAdvice::from(action));
                    }
                } else if !action.is_suppression() {
                    diag = diag.add_code_suggestion(CodeSuggestionAdvice::from(action));
                }
            }

            diagnostics.push(diagnostic_to_string(file_name, input_code, diag.into()));
            return ControlFlow::Continue(());
        }

        for action in event.actions() {
            if !action.is_suppression() {
                code_fixes.push(code_fix_to_string(input_code, action));
            }
        }

        ControlFlow::<Never>::Continue(())
    });

    for error in errors {
        diagnostics.push(diagnostic_to_string(file_name, input_code, error));
    }
    write_analyzer_snapshot(
        snapshot,
        input_code,
        diagnostics.as_slice(),
        code_fixes.as_slice(),
        "yaml",
        parsed.diagnostics().len(),
    );

    assert_diagnostics_expectation_comment(input_file, root.syntax(), diagnostics);
}
