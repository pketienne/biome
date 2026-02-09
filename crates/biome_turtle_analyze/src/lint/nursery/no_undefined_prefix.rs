use biome_analyze::{Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::TextRange;
use biome_turtle_syntax::TurtleRoot;

use crate::services::semantic::Semantic;

declare_lint_rule! {
    /// Disallow use of undeclared prefixes in Turtle documents.
    ///
    /// Using a prefixed name whose prefix has not been declared with a
    /// `@prefix` or `PREFIX` directive will cause a parsing error in most
    /// Turtle processors. This rule catches such issues early.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> dc:title "Alice" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> foaf:name "Alice" .
    /// ```
    ///
    pub NoUndefinedPrefix {
        version: "next",
        name: "noUndefinedPrefix",
        language: "turtle",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct UndefinedPrefix {
    prefix: String,
    range: TextRange,
}

impl Rule for NoUndefinedPrefix {
    type Query = Semantic<TurtleRoot>;
    type State = UndefinedPrefix;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let model = ctx.model();
        let prefix_map = model.prefix_map();
        let mut signals = Vec::new();

        for prefix_ref in model.prefix_references() {
            if !prefix_map.contains_key(&prefix_ref.namespace) {
                signals.push(UndefinedPrefix {
                    prefix: prefix_ref.namespace.clone(),
                    range: prefix_ref.range,
                });
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Prefix '"{ &state.prefix }"' is used but not declared."
                },
            )
            .note(markup! {
                "Add a prefix declaration, e.g. @prefix "{ &state.prefix }" <...> ."
            }),
        )
    }
}
