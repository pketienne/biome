use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_turtle_syntax::{
    AnyTurtleDirective, AnyTurtleStatement, TurtleRoot, TurtleSyntaxKind,
};
use std::collections::HashMap;

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
    full_iri: String,
    suggested: String,
}

impl Rule for UsePrefixedNames {
    type Query = Ast<TurtleRoot>;
    type State = ExpandableIri;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut signals = Vec::new();

        // Collect prefix declarations: namespace -> IRI expansion
        let mut prefixes: HashMap<String, String> = HashMap::new();
        for statement in root.statements() {
            if let AnyTurtleStatement::AnyTurtleDirective(directive) = &statement {
                match directive {
                    AnyTurtleDirective::TurtlePrefixDeclaration(decl) => {
                        if let (Ok(ns), Ok(iri)) =
                            (decl.namespace_token(), decl.iri_token())
                        {
                            let iri_text = iri.text_trimmed();
                            // Remove angle brackets from IRI
                            if iri_text.starts_with('<') && iri_text.ends_with('>') {
                                let expansion = &iri_text[1..iri_text.len() - 1];
                                prefixes.insert(
                                    expansion.to_string(),
                                    ns.text_trimmed().to_string(),
                                );
                            }
                        }
                    }
                    AnyTurtleDirective::TurtleSparqlPrefixDeclaration(decl) => {
                        if let (Ok(ns), Ok(iri)) =
                            (decl.namespace_token(), decl.iri_token())
                        {
                            let iri_text = iri.text_trimmed();
                            if iri_text.starts_with('<') && iri_text.ends_with('>') {
                                let expansion = &iri_text[1..iri_text.len() - 1];
                                prefixes.insert(
                                    expansion.to_string(),
                                    ns.text_trimmed().to_string(),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if prefixes.is_empty() {
            return signals;
        }

        // Scan all IRI reference tokens in the document
        for token in root.syntax().descendants_tokens(biome_rowan::Direction::Next) {
            if token.kind() == TurtleSyntaxKind::TURTLE_IRIREF_LITERAL {
                let text = token.text_trimmed();
                if text.starts_with('<') && text.ends_with('>') {
                    let iri = &text[1..text.len() - 1];
                    // Check if any declared prefix matches
                    for (expansion, ns) in &prefixes {
                        if iri.starts_with(expansion.as_str()) {
                            let local_name = &iri[expansion.len()..];
                            // Only suggest if local_name is a valid PN_LOCAL
                            if !local_name.is_empty()
                                && local_name
                                    .chars()
                                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
                            {
                                signals.push(ExpandableIri {
                                    range: token.text_trimmed_range(),
                                    full_iri: text.to_string(),
                                    suggested: format!("{ns}{local_name}"),
                                });
                            }
                            break;
                        }
                    }
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
                    "Use prefixed name "<Emphasis>{ &state.suggested }</Emphasis>" instead of full IRI."
                },
            )
            .note(markup! {
                "A matching prefix declaration is available. Using prefixed names is more concise."
            }),
        )
    }
}
