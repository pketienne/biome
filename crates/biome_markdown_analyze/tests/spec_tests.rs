use biome_analyze::{AnalysisFilter, AnalyzerAction, ControlFlow, Never, RuleFilter};
use biome_diagnostics::advice::CodeSuggestionAdvice;
use biome_markdown_analyze::analyze;
use biome_markdown_parser::parse_markdown;
use biome_markdown_syntax::{MarkdownLanguage, MdDocument};
use biome_rowan::AstNode;
use biome_test_utils::{
    CheckActionType, code_fix_to_string, create_analyzer_options, diagnostic_to_string,
    has_bogus_nodes_or_empty_slots, parse_test_path, register_leak_checker,
    write_analyzer_snapshot,
};
use camino::Utf8Path;
use std::ops::Deref;
use std::{fs::read_to_string, slice};

tests_macros::gen_tests! {"tests/specs/**/*.md", crate::run_test, "module"}

fn run_test(input: &'static str, _: &str, _: &str, _: &str) {
    register_leak_checker();

    let input_file = Utf8Path::new(input);
    let file_name = input_file.file_name().unwrap();

    let (group, rule) = parse_test_path(input_file);
    if rule == "specs" || rule == "suppression" {
        panic!("the test file must be placed in the {rule}/<group-name>/<rule-name>/ directory");
    }
    if group == "specs" || group == "suppression" {
        panic!("the test file must be placed in the {group}/{rule}/<rule-name>/ directory");
    }

    if biome_markdown_analyze::METADATA
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

    // The markdown parser does not support HTML comments, so valid test files
    // use the convention of an HTML comment on the first line as a signal.
    // We strip the comment before parsing so the parser doesn't produce a bogus tree.
    let (analysis_code, expects_no_diagnostics) = strip_markdown_test_comment(&input_code);

    analyze_and_snap(
        &mut snapshot,
        &analysis_code,
        filter,
        file_name,
        input_file,
        CheckActionType::Lint,
        expects_no_diagnostics,
    );

    insta::with_settings!({
        prepend_module_to_snapshot => false,
        snapshot_path => input_file.parent().unwrap(),
    }, {
        insta::assert_snapshot!(file_name, snapshot, file_name);
    });
}

/// Strip the `<!-- should not generate diagnostics -->` comment from the
/// beginning of a markdown test file. Returns the remaining content and
/// whether the expectation comment was present.
fn strip_markdown_test_comment(input: &str) -> (String, bool) {
    let no_diag = "<!-- should not generate diagnostics -->";
    if let Some(rest) = input.strip_prefix(no_diag) {
        // Strip leading newline after the comment
        let rest = rest.strip_prefix('\n').unwrap_or(rest);
        (rest.to_string(), true)
    } else {
        (input.to_string(), false)
    }
}

#[expect(clippy::too_many_arguments)]
fn analyze_and_snap(
    snapshot: &mut String,
    input_code: &str,
    filter: AnalysisFilter,
    file_name: &str,
    input_file: &Utf8Path,
    check_action_type: CheckActionType,
    expects_no_diagnostics: bool,
) {
    let mut diagnostics = Vec::new();
    let mut code_fixes = Vec::new();
    let options = create_analyzer_options::<MarkdownLanguage>(input_file, &mut diagnostics);

    let parsed = parse_markdown(input_code);
    let root = match MdDocument::cast(parsed.syntax()) {
        Some(doc) => doc,
        None => {
            // The parser produced a bogus root node (e.g. for unrecognized syntax
            // like unordered lists or tables). Write a snapshot noting the parse
            // failure so the test still records something useful.
            write_analyzer_snapshot(
                snapshot,
                input_code,
                &[],
                &[],
                "md",
                parsed.diagnostics().len(),
            );
            return;
        }
    };

    let (_, errors) = analyze(&root, filter, &options, |event| {
        if let Some(mut diag) = event.diagnostic() {
            for action in event.actions() {
                if check_action_type.is_suppression() {
                    if action.is_suppression() {
                        check_code_action(input_file, input_code, &action);
                        diag = diag.add_code_suggestion(CodeSuggestionAdvice::from(action));
                    }
                } else if !action.is_suppression() {
                    check_code_action(input_file, input_code, &action);
                    diag = diag.add_code_suggestion(CodeSuggestionAdvice::from(action));
                }
            }

            diagnostics.push(diagnostic_to_string(file_name, input_code, diag.into()));
            return ControlFlow::Continue(());
        }

        for action in event.actions() {
            if check_action_type.is_suppression() {
                if action.category.matches("quickfix.suppressRule") {
                    check_code_action(input_file, input_code, &action);
                    code_fixes.push(code_fix_to_string(input_code, action));
                }
            } else if !action.category.matches("quickfix.suppressRule") {
                check_code_action(input_file, input_code, &action);
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
        "md",
        parsed.diagnostics().len(),
    );

    let has_diagnostics = !diagnostics.is_empty();

    if expects_no_diagnostics && has_diagnostics {
        panic!(
            "This test should not generate diagnostics\nFile: {input_file}\n\nDiagnostics: {}",
            diagnostics.join("\n")
        );
    }

    // For "valid" files without the expectation comment, require the comment.
    if !expects_no_diagnostics {
        let name = file_name.to_ascii_lowercase();
        if name.contains("valid") && !name.contains("invalid") {
            panic!(
                "Valid test files should contain comment `should not generate diagnostics`\nFile: {input_file}",
            );
        }
    }
}

fn check_code_action(_path: &Utf8Path, source: &str, action: &AnalyzerAction<MarkdownLanguage>) {
    assert!(!action.mutation.is_empty(), "Mutation must not be empty");

    let (new_tree, text_edit) = match action
        .mutation
        .clone()
        .commit_with_text_range_and_edit(true)
    {
        (new_tree, Some((_, text_edit))) => (new_tree, text_edit),
        (new_tree, None) => (new_tree, Default::default()),
    };

    let output = text_edit.new_string(source);

    assert_eq!(
        new_tree.to_string(),
        output,
        "Code action and syntax tree differ"
    );

    if has_bogus_nodes_or_empty_slots(&new_tree) {
        panic!("modified tree has bogus nodes or empty slots:\n{new_tree:#?} \n\n {new_tree}")
    }

    if format!("{new_tree:?}").contains("missing (required)") {
        panic!("modified tree has missing children:\n{new_tree:#?}")
    }
}
