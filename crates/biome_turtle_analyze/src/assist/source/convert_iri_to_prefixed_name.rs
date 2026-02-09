use crate::TurtleRuleAction;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_source_rule};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, BatchMutationExt, Direction, TextRange};
use biome_turtle_syntax::{
    AnyTurtleDirective, AnyTurtleStatement, TurtleRoot, TurtleSyntaxKind, TurtleSyntaxToken,
};
use std::collections::HashMap;

declare_source_rule! {
    /// Convert full IRIs to prefixed names when a matching prefix is declared.
    ///
    /// Replaces `<http://xmlns.com/foaf/0.1/name>` with `foaf:name` when
    /// `@prefix foaf: <http://xmlns.com/foaf/0.1/>` is declared.
    ///
    /// ## Examples
    ///
    /// ```turtle,expect_diff
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> <http://xmlns.com/foaf/0.1/name> "Alice" .
    /// ```
    ///
    pub ConvertIriToPrefixedName {
        version: "next",
        name: "convertIriToPrefixedName",
        language: "turtle",
        fix_kind: FixKind::Safe,
    }
}

pub struct ConvertibleIris {
    range: TextRange,
    count: usize,
    replacements: Vec<(TurtleSyntaxToken, String)>,
}

impl Rule for ConvertIriToPrefixedName {
    type Query = Ast<TurtleRoot>;
    type State = ConvertibleIris;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut prefix_map: HashMap<String, String> = HashMap::new();

        // Collect all prefix declarations: namespace -> IRI expansion
        for statement in root.statements() {
            if let AnyTurtleStatement::AnyTurtleDirective(directive) = &statement {
                let info = match directive {
                    AnyTurtleDirective::TurtlePrefixDeclaration(decl) => {
                        let ns = decl.namespace_token().ok()?;
                        let iri = decl.iri_token().ok()?;
                        Some((ns.text_trimmed().to_string(), iri.text_trimmed().to_string()))
                    }
                    AnyTurtleDirective::TurtleSparqlPrefixDeclaration(decl) => {
                        let ns = decl.namespace_token().ok()?;
                        let iri = decl.iri_token().ok()?;
                        Some((ns.text_trimmed().to_string(), iri.text_trimmed().to_string()))
                    }
                    _ => None,
                };
                if let Some((ns, iri_text)) = info {
                    // Strip angle brackets from the IRI
                    let expansion = iri_text
                        .strip_prefix('<')
                        .and_then(|s| s.strip_suffix('>'))
                        .unwrap_or(&iri_text)
                        .to_string();
                    prefix_map.insert(expansion, ns);
                }
            }
        }

        if prefix_map.is_empty() {
            return None;
        }

        // Find all IRIREF tokens that can be converted
        let mut replacements = Vec::new();
        for token_or_node in root.syntax().descendants_with_tokens(Direction::Next) {
            if let Some(token) = token_or_node.into_token() {
                if token.kind() == TurtleSyntaxKind::TURTLE_IRIREF_LITERAL {
                    let text = token.text_trimmed();
                    let inner = text
                        .strip_prefix('<')
                        .and_then(|s| s.strip_suffix('>'))
                        .unwrap_or(text);

                    // Skip IRIs that are inside prefix/base declarations
                    if is_in_directive(&token) {
                        continue;
                    }

                    // Check if any prefix expansion matches the start of this IRI
                    for (expansion, ns) in &prefix_map {
                        if inner.starts_with(expansion.as_str()) {
                            let local_name = &inner[expansion.len()..];
                            // Validate local name (must be valid PN_LOCAL)
                            if is_valid_local_name(local_name) {
                                let prefixed = std::format!("{ns}{local_name}");
                                replacements.push((token.clone(), prefixed));
                                break;
                            }
                        }
                    }
                }
            }
        }

        if replacements.is_empty() {
            return None;
        }

        let first_range = replacements.first()?.0.text_trimmed_range();
        let last_range = replacements.last()?.0.text_trimmed_range();
        let count = replacements.len();

        Some(ConvertibleIris {
            range: TextRange::new(first_range.start(), last_range.end()),
            count,
            replacements,
        })
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                category!("assist/source/convertIriToPrefixedName"),
                state.range,
                markup! { {std::format!("{} IRI(s) can be converted to prefixed names.", state.count)} },
            )
            .note(markup! { "Use prefixed names for more concise and readable Turtle." }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<TurtleRuleAction> {
        let mut mutation = ctx.root().begin();

        for (old_token, prefixed_name) in &state.replacements {
            let new_token = TurtleSyntaxToken::new_detached(
                TurtleSyntaxKind::TURTLE_PNAME_LN_LITERAL,
                prefixed_name,
                [],
                [],
            );
            mutation.replace_token_transfer_trivia(old_token.clone(), new_token);
        }

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Convert IRIs to prefixed names." }.to_owned(),
            mutation,
        ))
    }
}

/// Check if a token is inside a directive (prefix or base declaration).
fn is_in_directive(token: &TurtleSyntaxToken) -> bool {
    let mut parent = token.parent();
    while let Some(node) = parent {
        match node.kind() {
            TurtleSyntaxKind::TURTLE_PREFIX_DECLARATION
            | TurtleSyntaxKind::TURTLE_SPARQL_PREFIX_DECLARATION
            | TurtleSyntaxKind::TURTLE_BASE_DECLARATION
            | TurtleSyntaxKind::TURTLE_SPARQL_BASE_DECLARATION => return true,
            _ => parent = node.parent(),
        }
    }
    false
}

/// Validate that a string is a valid PN_LOCAL (Turtle local name).
fn is_valid_local_name(name: &str) -> bool {
    if name.is_empty() {
        return true; // Empty local name is valid (e.g., foaf:)
    }
    // Simplified validation: alphanumeric, underscore, hyphen, dot (not at end)
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        && !name.ends_with('.')
}
