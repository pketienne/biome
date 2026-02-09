use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_turtle_syntax::{AnyTurtleDirective, AnyTurtleStatement, TurtlePrefixedName, TurtleRoot};
use std::collections::HashSet;

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
    type Query = Ast<TurtleRoot>;
    type State = UndefinedPrefix;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut declared: HashSet<String> = HashSet::new();
        let mut signals = Vec::new();

        // Collect all declared prefixes
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
                    declared.insert(ns);
                }
            }
        }

        // Check all prefixed names
        for node in root.syntax().descendants() {
            if let Some(prefixed_name) = TurtlePrefixedName::cast_ref(&node) {
                if let Ok(token) = prefixed_name.value() {
                    let text = token.text_trimmed();
                    // Extract prefix part (everything before the first ':')
                    if let Some(colon_pos) = text.find(':') {
                        let prefix = &text[..=colon_pos]; // includes the ':'
                        if !declared.contains(prefix) {
                            signals.push(UndefinedPrefix {
                                prefix: prefix.to_string(),
                                range: prefixed_name.syntax().text_trimmed_range(),
                            });
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
                    "Prefix '"{ &state.prefix }"' is used but not declared."
                },
            )
            .note(markup! {
                "Add a prefix declaration, e.g. @prefix "{ &state.prefix }" <...> ."
            }),
        )
    }
}
