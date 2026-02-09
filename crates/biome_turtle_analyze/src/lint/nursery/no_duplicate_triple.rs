use biome_analyze::{Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::TextRange;
use biome_turtle_syntax::TurtleRoot;

use crate::services::semantic::Semantic;

declare_lint_rule! {
    /// Disallow duplicate triples in Turtle documents.
    ///
    /// Duplicate subject-predicate-object triples are redundant in RDF.
    /// They add no new information and may indicate copy-paste errors.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:name "Alice" .
    /// ex:alice ex:name "Alice" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:name "Alice" .
    /// ex:bob ex:name "Bob" .
    /// ```
    ///
    pub NoDuplicateTriple {
        version: "next",
        name: "noDuplicateTriple",
        language: "turtle",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct DuplicateTriple {
    range: TextRange,
    triple_text: String,
}

impl Rule for NoDuplicateTriple {
    type Query = Semantic<TurtleRoot>;
    type State = DuplicateTriple;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let model = ctx.model();
        let triples = model.triples();
        let mut signals = Vec::new();

        for &(_, dup_idx) in model.duplicate_triples() {
            let triple = &triples[dup_idx];
            signals.push(DuplicateTriple {
                range: triple.statement_range,
                triple_text: format!("{} {} {}", triple.subject, triple.predicate, triple.object),
            });
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Duplicate triple: "{ &state.triple_text }"."
                },
            )
            .note(markup! {
                "Remove the duplicate triple."
            }),
        )
    }
}
