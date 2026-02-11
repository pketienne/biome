#![deny(clippy::use_self)]

mod lint;
mod registry;
mod suppression_action;

pub use crate::registry::visit_registry;
use crate::suppression_action::YamlSuppressionAction;
use biome_analyze::{
    AnalysisFilter, AnalyzerOptions, AnalyzerSignal, AnalyzerSuppression, ControlFlow,
    LanguageRoot, MatchQueryParams, MetadataRegistry, RuleRegistry, to_analyzer_suppressions,
};
use biome_diagnostics::Error;
use biome_suppression::{SuppressionDiagnostic, parse_suppression_comment};
use biome_yaml_syntax::{TextRange, YamlFileSource, YamlLanguage};
use std::ops::Deref;
use std::sync::LazyLock;

pub static METADATA: LazyLock<MetadataRegistry> = LazyLock::new(|| {
    let mut metadata = MetadataRegistry::default();
    visit_registry(&mut metadata);
    metadata
});

pub struct YamlAnalyzeServices {
    /// The source file
    pub file_source: YamlFileSource,
}

/// Run the analyzer on the provided `root`: this process will use the given `filter`
/// to selectively restrict analysis to specific rules / a specific source range,
/// then call `emit_signal` when an analysis rule emits a diagnostic or action
pub fn analyze<'a, F, B>(
    root: &LanguageRoot<YamlLanguage>,
    filter: AnalysisFilter,
    options: &'a AnalyzerOptions,
    yaml_services: YamlAnalyzeServices,
    emit_signal: F,
) -> (Option<B>, Vec<Error>)
where
    F: FnMut(&dyn AnalyzerSignal<YamlLanguage>) -> ControlFlow<B> + 'a,
    B: 'a,
{
    analyze_with_inspect_matcher(root, filter, |_| {}, options, yaml_services, emit_signal)
}

/// Run the analyzer on the provided `root`: this process will use the given `filter`
/// to selectively restrict analysis to specific rules / a specific source range,
/// then call `emit_signal` when an analysis rule emits a diagnostic or action.
/// Additionally, this function takes a `inspect_matcher` function that can be
/// used to inspect the "query matches" emitted by the analyzer before they are
/// processed by the lint rules registry
pub fn analyze_with_inspect_matcher<'a, V, F, B>(
    root: &LanguageRoot<YamlLanguage>,
    filter: AnalysisFilter,
    inspect_matcher: V,
    options: &'a AnalyzerOptions,
    yaml_services: YamlAnalyzeServices,
    mut emit_signal: F,
) -> (Option<B>, Vec<Error>)
where
    V: FnMut(&MatchQueryParams<YamlLanguage>) + 'a,
    F: FnMut(&dyn AnalyzerSignal<YamlLanguage>) -> ControlFlow<B> + 'a,
    B: 'a,
{
    fn parse_linter_suppression_comment(
        text: &str,
        piece_range: TextRange,
    ) -> Vec<Result<AnalyzerSuppression<'_>, SuppressionDiagnostic>> {
        let mut result = Vec::new();

        for suppression in parse_suppression_comment(text) {
            let suppression = match suppression {
                Ok(suppression) => suppression,
                Err(err) => {
                    result.push(Err(err));
                    continue;
                }
            };

            let analyzer_suppressions: Vec<_> = to_analyzer_suppressions(suppression, piece_range)
                .into_iter()
                .map(Ok)
                .collect();

            result.extend(analyzer_suppressions)
        }

        result
    }

    let mut registry = RuleRegistry::builder(&filter, root);
    visit_registry(&mut registry);

    let (registry, mut services, diagnostics, visitors) = registry.build();

    // Bail if we can't parse a rule option
    if !diagnostics.is_empty() {
        return (None, diagnostics);
    }

    let mut analyzer = biome_analyze::Analyzer::new(
        METADATA.deref(),
        biome_analyze::InspectMatcher::new(registry, inspect_matcher),
        parse_linter_suppression_comment,
        Box::new(YamlSuppressionAction),
        &mut emit_signal,
    );

    for ((phase, _), visitor) in visitors {
        analyzer.add_visitor(phase, visitor);
    }

    services.insert_service(yaml_services.file_source);

    (
        analyzer.run(biome_analyze::AnalyzerContext {
            root: root.clone(),
            range: filter.range,
            services,
            options,
        }),
        diagnostics,
    )
}

#[cfg(test)]
mod tests {
    use biome_analyze::{AnalyzerOptions, Never, RuleFilter};
    use biome_console::fmt::{Formatter, Termcolor};
    use biome_console::{Markup, markup};
    use biome_diagnostics::termcolor::NoColor;
    use biome_diagnostics::{Diagnostic, DiagnosticExt, PrintDiagnostic, Severity};
    use biome_yaml_parser::parse_yaml;
    use biome_yaml_syntax::{TextRange, YamlFileSource};
    use std::slice;

    use crate::{AnalysisFilter, ControlFlow, YamlAnalyzeServices, analyze};

    #[test]
    fn quick_test() {
        fn markup_to_string(markup: Markup) -> String {
            let mut buffer = Vec::new();
            let mut write = Termcolor(NoColor::new(&mut buffer));
            let mut fmt = Formatter::new(&mut write);
            fmt.write_markup(markup).unwrap();

            String::from_utf8(buffer).unwrap()
        }

        const SOURCE: &str = r#"name: John
age: 30
name: Jane
"#;

        let parsed = parse_yaml(SOURCE);

        let mut error_ranges: Vec<TextRange> = Vec::new();
        let rule_filter = RuleFilter::Rule("suspicious", "noDuplicateKeys");
        let options = AnalyzerOptions::default();
        let services = YamlAnalyzeServices {
            file_source: YamlFileSource::default(),
        };
        analyze(
            &parsed.tree(),
            AnalysisFilter {
                enabled_rules: Some(slice::from_ref(&rule_filter)),
                ..AnalysisFilter::default()
            },
            &options,
            services,
            |signal| {
                if let Some(diag) = signal.diagnostic() {
                    error_ranges.push(diag.location().span.unwrap());
                    let error = diag
                        .with_severity(Severity::Warning)
                        .with_file_path("test.yaml")
                        .with_file_source_code(SOURCE);
                    let text = markup_to_string(markup! {
                        {PrintDiagnostic::verbose(&error)}
                    });
                    eprintln!("{text}");
                }

                for action in signal.actions() {
                    let new_code = action.mutation.commit();
                    eprintln!("{new_code}");
                }

                ControlFlow::<Never>::Continue(())
            },
        );

        assert!(!error_ranges.is_empty(), "Expected duplicate key diagnostics but found none");
    }
}
