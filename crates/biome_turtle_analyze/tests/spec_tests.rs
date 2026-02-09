use biome_analyze::{AnalysisFilter, AnalyzerAction, ControlFlow, Never, RuleFilter};
use biome_diagnostics::advice::CodeSuggestionAdvice;
use biome_rowan::AstNode;
use biome_test_utils::{
    assert_errors_are_absent, code_fix_to_string, diagnostic_to_string, has_bogus_nodes_or_empty_slots,
    parse_test_path, register_leak_checker, write_analyzer_snapshot,
};
use biome_turtle_analyze::analyze;
use biome_turtle_parser::parse_turtle;
use biome_turtle_syntax::TurtleLanguage;
use camino::Utf8Path;
use std::ops::Deref;
use std::{fs::read_to_string, slice};

tests_macros::gen_tests! {"tests/specs/**/*.ttl", crate::run_test, "module"}

fn run_test(input: &'static str, _: &str, _: &str, _: &str) {
    register_leak_checker();

    let input_file = Utf8Path::new(input);
    let file_name = input_file.file_name().unwrap();

    let (group, rule) = parse_test_path(input_file);
    if rule == "specs" {
        panic!("the test file must be placed in the {rule}/<group-name>/<rule-name>/ directory");
    }
    if group == "specs" {
        panic!("the test file must be placed in the {group}/{rule}/<rule-name>/ directory");
    }
    if biome_turtle_analyze::METADATA
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

    let input_code = read_to_string(input_file)
        .unwrap_or_else(|err| panic!("failed to read {input_file:?}: {err:?}"));

    let parsed = parse_turtle(&input_code);
    let root = parsed.tree();

    let mut diagnostics = Vec::new();
    let mut code_fixes = Vec::new();
    let options = biome_analyze::AnalyzerOptions::default();

    let (_, errors) = analyze(&root, filter, &options, |signal| {
        if let Some(mut diag) = signal.diagnostic() {
            for action in signal.actions() {
                if !action.is_suppression() {
                    check_code_action(input_file, &input_code, &action);
                    diag = diag.add_code_suggestion(CodeSuggestionAdvice::from(action));
                }
            }

            diagnostics.push(diagnostic_to_string(file_name, &input_code, diag.into()));
            return ControlFlow::Continue(());
        }

        for action in signal.actions() {
            if !action.category.matches("quickfix.suppressRule") {
                check_code_action(input_file, &input_code, &action);
                code_fixes.push(code_fix_to_string(&input_code, action));
            }
        }

        ControlFlow::<Never>::Continue(())
    });

    for error in errors {
        diagnostics.push(diagnostic_to_string(file_name, &input_code, error));
    }

    let mut snapshot = String::new();
    write_analyzer_snapshot(
        &mut snapshot,
        &input_code,
        diagnostics.as_slice(),
        code_fixes.as_slice(),
        "turtle",
        parsed.diagnostics().len(),
    );

    insta::with_settings!({
        prepend_module_to_snapshot => false,
        snapshot_path => input_file.parent().unwrap(),
    }, {
        insta::assert_snapshot!(file_name, snapshot, file_name);
    });
}

fn check_code_action(
    path: &Utf8Path,
    source: &str,
    action: &AnalyzerAction<TurtleLanguage>,
) {
    let (new_tree, text_edit) = match action
        .mutation
        .clone()
        .commit_with_text_range_and_edit(true)
    {
        (new_tree, Some((_, text_edit))) => (new_tree, text_edit),
        (new_tree, None) => (new_tree, Default::default()),
    };

    let output = text_edit.new_string(source);

    // Checks that applying the text edits returned by the BatchMutation
    // returns the same code as printing the modified syntax tree
    assert_eq!(new_tree.to_string(), output);

    if has_bogus_nodes_or_empty_slots(&new_tree) {
        panic!("modified tree has bogus nodes or empty slots:\n{new_tree:#?} \n\n {new_tree}")
    }

    // Checks the returned tree contains no missing children node
    if format!("{new_tree:?}").contains("missing (required)") {
        panic!("modified tree has missing children:\n{new_tree:#?}")
    }

    // Re-parse the modified code and panic if the resulting tree has syntax errors
    let re_parse = parse_turtle(&output);
    assert_errors_are_absent(re_parse.tree().syntax(), re_parse.diagnostics(), path);
}
