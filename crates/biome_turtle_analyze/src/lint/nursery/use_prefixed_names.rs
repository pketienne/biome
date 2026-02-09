use biome_analyze::{Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::TextRange;
use biome_turtle_syntax::TurtleRoot;

use crate::services::semantic::Semantic;

declare_lint_rule! {
    /// Suggest using prefixed names instead of full IRIs when a matching prefix is declared.
    ///
    /// When a prefix is declared that matches the namespace of a full IRI,
    /// using the prefixed form is more concise and readable.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> <http://xmlns.com/foaf/0.1/name> "Alice" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> foaf:name "Alice" .
    /// ```
    ///
    pub UsePrefixedNames {
        version: "next",
        name: "usePrefixedNames",
        language: "turtle",
        recommended: false,
        severity: Severity::Information,
    }
}

pub struct ExpandableIri {
    range: TextRange,
    suggested: String,
}

impl Rule for UsePrefixedNames {
    type Query = Semantic<TurtleRoot>;
    type State = ExpandableIri;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let model = ctx.model();

        model
            .expandable_iris()
            .filter_map(|iri_ref| {
                iri_ref.suggested_prefixed.as_ref().map(|suggested| ExpandableIri {
                    range: iri_ref.range,
                    suggested: suggested.clone(),
                })
            })
            .collect()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Use prefixed name "<Emphasis>{ &state.suggested }</Emphasis>" instead of full IRI."
                },
            )
            .note(markup! {
                "A matching prefix declaration is available. Using prefixed names is more concise."
            }),
        )
    }
}
