use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, SyntaxElement, SyntaxNode, TextRange, TriviaPiece};
use biome_turtle_syntax::{
    AnyTurtleDirective, AnyTurtleStatement, TurtleLanguage, TurtleRoot, TurtleSyntaxKind,
    TurtleSyntaxToken,
};

type TurtleRuleAction = RuleAction<TurtleLanguage>;

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
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentStyle {
    range: TextRange,
    found_style: &'static str,
    expected_style: &'static str,
    directive: AnyTurtleDirective,
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
                            directive: directive.clone(),
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

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<TurtleRuleAction> {
        let mut mutation = ctx.root().begin();

        match &state.directive {
            AnyTurtleDirective::TurtleSparqlPrefixDeclaration(sparql_prefix) => {
                let namespace = sparql_prefix.namespace_token().ok()?;
                let iri = sparql_prefix.iri_token().ok()?;
                let ns_text = namespace.text_trimmed();
                let iri_text = iri.text_trimmed();

                // Build @prefix ns: <iri> . as a TurtlePrefixDeclaration node
                // Text must include trivia characters; trailing TriviaPiece describes the trivia portion
                let ws = TriviaPiece::whitespace(1);
                let prefix_token = TurtleSyntaxToken::new_detached(
                    TurtleSyntaxKind::PREFIX_KW,
                    "@prefix ",
                    [],
                    [ws],
                );
                let ns_token = TurtleSyntaxToken::new_detached(
                    TurtleSyntaxKind::TURTLE_PNAME_NS_LITERAL,
                    &std::format!("{ns_text} "),
                    [],
                    [ws],
                );
                let iri_token = TurtleSyntaxToken::new_detached(
                    TurtleSyntaxKind::TURTLE_IRIREF_LITERAL,
                    &std::format!("{iri_text} "),
                    [],
                    [ws],
                );
                let dot_token =
                    TurtleSyntaxToken::new_detached(TurtleSyntaxKind::DOT, ".", [], []);

                let new_node: SyntaxNode<TurtleLanguage> = SyntaxNode::new_detached(
                    TurtleSyntaxKind::TURTLE_PREFIX_DECLARATION,
                    [
                        Some(SyntaxElement::Token(prefix_token)),
                        Some(SyntaxElement::Token(ns_token)),
                        Some(SyntaxElement::Token(iri_token)),
                        Some(SyntaxElement::Token(dot_token)),
                    ],
                );

                mutation.replace_element(
                    sparql_prefix.syntax().clone().into(),
                    new_node.into(),
                );
            }
            AnyTurtleDirective::TurtleSparqlBaseDeclaration(sparql_base) => {
                let iri = sparql_base.iri_token().ok()?;
                let iri_text = iri.text_trimmed();

                // Build @base <iri> . as a TurtleBaseDeclaration node
                let ws = TriviaPiece::whitespace(1);
                let base_token = TurtleSyntaxToken::new_detached(
                    TurtleSyntaxKind::BASE_KW,
                    "@base ",
                    [],
                    [ws],
                );
                let iri_token = TurtleSyntaxToken::new_detached(
                    TurtleSyntaxKind::TURTLE_IRIREF_LITERAL,
                    &std::format!("{iri_text} "),
                    [],
                    [ws],
                );
                let dot_token =
                    TurtleSyntaxToken::new_detached(TurtleSyntaxKind::DOT, ".", [], []);

                let new_node: SyntaxNode<TurtleLanguage> = SyntaxNode::new_detached(
                    TurtleSyntaxKind::TURTLE_BASE_DECLARATION,
                    [
                        Some(SyntaxElement::Token(base_token)),
                        Some(SyntaxElement::Token(iri_token)),
                        Some(SyntaxElement::Token(dot_token)),
                    ],
                );

                mutation.replace_element(
                    sparql_base.syntax().clone().into(),
                    new_node.into(),
                );
            }
            _ => return None,
        }

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Convert to Turtle-style directive." }.to_owned(),
            mutation,
        ))
    }
}
