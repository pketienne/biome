use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_turtle_syntax::{AnyTurtleDirective, AnyTurtleStatement, TurtlePrefixedName, TurtleRoot};
use std::collections::{HashMap, HashSet};

declare_lint_rule! {
    /// Disallow unused prefix declarations in Turtle documents.
    ///
    /// Prefix declarations that are never used add unnecessary clutter to the
    /// document and may indicate leftover code from refactoring.
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
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> foaf:name "Alice" .
    /// ```
    ///
    pub NoUnusedPrefix {
        version: "next",
        name: "noUnusedPrefix",
        language: "turtle",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct UnusedPrefix {
    namespace: String,
    range: TextRange,
}

impl Rule for NoUnusedPrefix {
    type Query = Ast<TurtleRoot>;
    type State = UnusedPrefix;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut declared: HashMap<String, TextRange> = HashMap::new();
        let mut used: HashSet<String> = HashSet::new();

        // Collect all declared prefixes
        for statement in root.statements() {
            if let AnyTurtleStatement::AnyTurtleDirective(directive) = &statement {
                let info = match directive {
                    AnyTurtleDirective::TurtlePrefixDeclaration(decl) => {
                        decl.namespace_token().ok().map(|t| {
                            (t.text_trimmed().to_string(), directive.syntax().text_trimmed_range())
                        })
                    }
                    AnyTurtleDirective::TurtleSparqlPrefixDeclaration(decl) => {
                        decl.namespace_token().ok().map(|t| {
                            (t.text_trimmed().to_string(), directive.syntax().text_trimmed_range())
                        })
                    }
                    _ => None,
                };
                if let Some((ns, range)) = info {
                    declared.insert(ns, range);
                }
            }
        }

        // Collect all used prefixes
        for node in root.syntax().descendants() {
            if let Some(prefixed_name) = TurtlePrefixedName::cast_ref(&node) {
                if let Ok(token) = prefixed_name.value() {
                    let text = token.text_trimmed();
                    if let Some(colon_pos) = text.find(':') {
                        let prefix = &text[..=colon_pos];
                        used.insert(prefix.to_string());
                    }
                }
            }
        }

        // Report unused
        declared
            .into_iter()
            .filter(|(ns, _)| !used.contains(ns.as_str()))
            .map(|(namespace, range)| UnusedPrefix { namespace, range })
            .collect()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Prefix '"{ &state.namespace }"' is declared but never used."
                },
            )
            .note(markup! {
                "Remove the unused prefix declaration."
            }),
        )
    }
}
