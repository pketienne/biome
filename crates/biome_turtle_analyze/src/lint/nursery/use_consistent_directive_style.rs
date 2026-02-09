use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_turtle_syntax::{AnyTurtleDirective, AnyTurtleStatement, TurtleRoot};

declare_lint_rule! {
    /// Disallow mixing `@prefix`/`@base` and `PREFIX`/`BASE` directive styles.
    ///
    /// Turtle supports two syntaxes for prefix and base declarations:
    /// the Turtle-native `@prefix`/`@base` (with trailing `.`) and the
    /// SPARQL-compatible `PREFIX`/`BASE` (without trailing `.`).
    /// Mixing both styles in a single document is confusing.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// PREFIX dc: <http://purl.org/dc/elements/1.1/>
    /// <http://example.org/alice> foaf:name "Alice" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// @prefix dc: <http://purl.org/dc/elements/1.1/> .
    /// <http://example.org/alice> foaf:name "Alice" .
    /// ```
    ///
    pub UseConsistentDirectiveStyle {
        version: "next",
        name: "useConsistentDirectiveStyle",
        language: "turtle",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct InconsistentStyle {
    range: TextRange,
    found_style: &'static str,
    expected_style: &'static str,
}

impl Rule for UseConsistentDirectiveStyle {
    type Query = Ast<TurtleRoot>;
    type State = InconsistentStyle;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut signals = Vec::new();
        let mut has_turtle_style = false;
        let mut has_sparql_style = false;

        // First pass: determine which styles are used
        for statement in root.statements() {
            if let AnyTurtleStatement::AnyTurtleDirective(directive) = &statement {
                match directive {
                    AnyTurtleDirective::TurtlePrefixDeclaration(_)
                    | AnyTurtleDirective::TurtleBaseDeclaration(_) => {
                        has_turtle_style = true;
                    }
                    AnyTurtleDirective::TurtleSparqlPrefixDeclaration(_)
                    | AnyTurtleDirective::TurtleSparqlBaseDeclaration(_) => {
                        has_sparql_style = true;
                    }
                }
            }
        }

        // Only report if both styles are mixed
        if !(has_turtle_style && has_sparql_style) {
            return signals;
        }

        // Second pass: report the minority style (SPARQL style, since Turtle is more common)
        for statement in root.statements() {
            if let AnyTurtleStatement::AnyTurtleDirective(directive) = &statement {
                match directive {
                    AnyTurtleDirective::TurtleSparqlPrefixDeclaration(_)
                    | AnyTurtleDirective::TurtleSparqlBaseDeclaration(_) => {
                        signals.push(InconsistentStyle {
                            range: directive.syntax().text_trimmed_range(),
                            found_style: "SPARQL",
                            expected_style: "Turtle (@prefix/@base)",
                        });
                    }
                    _ => {}
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
                    "Mixed directive styles: found "{ state.found_style }" style."
                },
            )
            .note(markup! {
                "Use "{ state.expected_style }" style consistently throughout the document."
            }),
        )
    }
}
