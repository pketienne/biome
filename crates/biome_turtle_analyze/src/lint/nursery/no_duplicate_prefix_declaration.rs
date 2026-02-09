use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_turtle_syntax::{AnyTurtleDirective, AnyTurtleStatement, TurtleRoot};
use std::collections::HashMap;

declare_lint_rule! {
    /// Disallow duplicate prefix declarations in Turtle documents.
    ///
    /// Having multiple prefix declarations for the same namespace prefix is
    /// confusing and error-prone. Only the last declaration will take effect,
    /// which can lead to unexpected behavior.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// @prefix foaf: <http://example.org/foaf/> .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// @prefix dc: <http://purl.org/dc/elements/1.1/> .
    /// ```
    ///
    pub NoDuplicatePrefixDeclaration {
        version: "next",
        name: "noDuplicatePrefixDeclaration",
        language: "turtle",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct DuplicatePrefix {
    namespace: String,
    range: TextRange,
}

impl Rule for NoDuplicatePrefixDeclaration {
    type Query = Ast<TurtleRoot>;
    type State = DuplicatePrefix;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut seen: HashMap<String, TextRange> = HashMap::new();
        let mut signals = Vec::new();

        for statement in root.statements() {
            let directive = match statement {
                AnyTurtleStatement::AnyTurtleDirective(d) => d,
                _ => continue,
            };

            let namespace = match &directive {
                AnyTurtleDirective::TurtlePrefixDeclaration(decl) => {
                    decl.namespace_token().ok().map(|t| t.text_trimmed().to_string())
                }
                AnyTurtleDirective::TurtleSparqlPrefixDeclaration(decl) => {
                    decl.namespace_token().ok().map(|t| t.text_trimmed().to_string())
                }
                _ => None,
            };

            if let Some(ns) = namespace {
                let range = directive.syntax().text_trimmed_range();
                if seen.contains_key(&ns) {
                    signals.push(DuplicatePrefix {
                        namespace: ns,
                        range,
                    });
                } else {
                    seen.insert(ns, range);
                }
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
                    "Duplicate prefix declaration for '"{ &state.namespace }"'."
                },
            )
            .note(markup! {
                "Only the last declaration takes effect. Remove the duplicate."
            }),
        )
    }
}
