use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_turtle_syntax::{AnyTurtleDirective, AnyTurtleStatement, TurtleRoot};

declare_lint_rule! {
    /// Enforce alphabetical ordering of prefix declarations.
    ///
    /// Keeping prefix declarations sorted alphabetically makes it easier
    /// to find and manage prefixes in large Turtle documents.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// @prefix dc: <http://purl.org/dc/elements/1.1/> .
    /// <http://example.org/alice> foaf:name "Alice" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix dc: <http://purl.org/dc/elements/1.1/> .
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> foaf:name "Alice" .
    /// ```
    ///
    pub UseSortedPrefixes {
        version: "next",
        name: "useSortedPrefixes",
        language: "turtle",
        recommended: false,
        severity: Severity::Information,
    }
}

pub struct UnsortedPrefix {
    range: TextRange,
    namespace: String,
    previous_namespace: String,
}

impl Rule for UseSortedPrefixes {
    type Query = Ast<TurtleRoot>;
    type State = UnsortedPrefix;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut signals = Vec::new();
        let mut prev_ns: Option<String> = None;

        for statement in root.statements() {
            if let AnyTurtleStatement::AnyTurtleDirective(directive) = &statement {
                let namespace = match directive {
                    AnyTurtleDirective::TurtlePrefixDeclaration(decl) => {
                        decl.namespace_token().ok().map(|t| t.text_trimmed().to_string())
                    }
                    AnyTurtleDirective::TurtleSparqlPrefixDeclaration(decl) => {
                        decl.namespace_token().ok().map(|t| t.text_trimmed().to_string())
                    }
                    _ => None,
                };

                if let Some(ns) = namespace {
                    if let Some(prev) = &prev_ns {
                        if ns.to_lowercase() < prev.to_lowercase() {
                            signals.push(UnsortedPrefix {
                                range: directive.syntax().text_trimmed_range(),
                                namespace: ns.clone(),
                                previous_namespace: prev.clone(),
                            });
                        }
                    }
                    prev_ns = Some(ns);
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
                    "Prefix '"{ &state.namespace }"' should come before '"{ &state.previous_namespace }"'."
                },
            )
            .note(markup! {
                "Sort prefix declarations alphabetically for consistency."
            }),
        )
    }
}
