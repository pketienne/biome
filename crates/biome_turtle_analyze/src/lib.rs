#![deny(clippy::use_self)]

mod lint;
pub mod options;
mod registry;
mod suppression_action;

pub use crate::registry::visit_registry;
use crate::suppression_action::TurtleSuppressionAction;
use biome_analyze::{
    AnalysisFilter, AnalyzerOptions, AnalyzerSignal, AnalyzerSuppression, ControlFlow,
    LanguageRoot, MatchQueryParams, MetadataRegistry, RuleAction, RuleRegistry,
    to_analyzer_suppressions,
};
use biome_rowan::TextRange;
use biome_diagnostics::Error;
use biome_suppression::{SuppressionDiagnostic, parse_suppression_comment};
use biome_turtle_syntax::TurtleLanguage;
use std::ops::Deref;
use std::sync::LazyLock;

pub(crate) type TurtleRuleAction = RuleAction<TurtleLanguage>;

pub static METADATA: LazyLock<MetadataRegistry> = LazyLock::new(|| {
    let mut metadata = MetadataRegistry::default();
    visit_registry(&mut metadata);
    metadata
});

/// Run the analyzer on the provided `root`: this process will use the given `filter`
/// to selectively restrict analysis to specific rules / a specific source range,
/// then call `emit_signal` when an analysis rule emits a diagnostic or action
pub fn analyze<'a, F, B>(
    root: &LanguageRoot<TurtleLanguage>,
    filter: AnalysisFilter,
    options: &'a AnalyzerOptions,
    emit_signal: F,
) -> (Option<B>, Vec<Error>)
where
    F: FnMut(&dyn AnalyzerSignal<TurtleLanguage>) -> ControlFlow<B> + 'a,
    B: 'a,
{
    analyze_with_inspect_matcher(root, filter, |_| {}, options, emit_signal)
}

pub fn analyze_with_inspect_matcher<'a, V, F, B>(
    root: &LanguageRoot<TurtleLanguage>,
    filter: AnalysisFilter,
    inspect_matcher: V,
    options: &'a AnalyzerOptions,
    mut emit_signal: F,
) -> (Option<B>, Vec<Error>)
where
    V: FnMut(&MatchQueryParams<TurtleLanguage>) + 'a,
    F: FnMut(&dyn AnalyzerSignal<TurtleLanguage>) -> ControlFlow<B> + 'a,
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

    let (registry, services, diagnostics, visitors) = registry.build();

    // Bail if we can't parse a rule option
    if !diagnostics.is_empty() {
        return (None, diagnostics);
    }

    let mut analyzer = biome_analyze::Analyzer::new(
        METADATA.deref(),
        biome_analyze::InspectMatcher::new(registry, inspect_matcher),
        parse_linter_suppression_comment,
        Box::new(TurtleSuppressionAction),
        &mut emit_signal,
    );

    for ((phase, _), visitor) in visitors {
        analyzer.add_visitor(phase, visitor);
    }

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
    use crate::analyze;
    use biome_analyze::{AnalysisFilter, AnalyzerOptions, ControlFlow, Never};
    use biome_turtle_parser::parse_turtle;

    #[test]
    fn analyzer_smoke_test() {
        let src = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
<http://example.org/alice> a rdf:Person .
"#;

        let parsed = parse_turtle(src);
        let options = AnalyzerOptions::default();
        let (result, errors) = analyze(
            &parsed.tree(),
            AnalysisFilter::default(),
            &options,
            |_signal| ControlFlow::<Never>::Continue(()),
        );

        assert!(result.is_none());
        assert!(errors.is_empty());
    }

    #[test]
    fn lint_detects_duplicate_prefix() {
        let src = r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
<http://example.org/alice> a foaf:Person .
"#;

        let parsed = parse_turtle(src);
        let options = AnalyzerOptions::default();
        let mut diagnostics = vec![];
        let (_, errors) = analyze(
            &parsed.tree(),
            AnalysisFilter::default(),
            &options,
            |signal| {
                if let Some(diag) = signal.diagnostic() {
                    let text = format!("{:?}", diag);
                    diagnostics.push(text);
                }
                ControlFlow::<Never>::Continue(())
            },
        );

        assert!(errors.is_empty());
        assert!(
            !diagnostics.is_empty(),
            "Expected at least one diagnostic for duplicate prefix, got none"
        );
    }

    #[test]
    fn lint_detects_rdf_type_shorthand() {
        let src = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
<http://example.org/alice> rdf:type foaf:Person .
"#;

        let parsed = parse_turtle(src);
        let options = AnalyzerOptions::default();
        let mut diagnostics = vec![];
        let (_, errors) = analyze(
            &parsed.tree(),
            AnalysisFilter::default(),
            &options,
            |signal| {
                if let Some(diag) = signal.diagnostic() {
                    let text = format!("{:?}", diag);
                    diagnostics.push(text);
                }
                ControlFlow::<Never>::Continue(())
            },
        );

        assert!(errors.is_empty());
        assert!(
            !diagnostics.is_empty(),
            "Expected at least one diagnostic for rdf:type usage, got none"
        );
    }
}
