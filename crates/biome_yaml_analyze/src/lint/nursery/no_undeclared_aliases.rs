use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, Direction, TextRange, WalkEvent};
use biome_yaml_syntax::{YamlRoot, YamlSyntaxKind};
use rustc_hash::FxHashSet;

declare_lint_rule! {
    /// Disallow aliases that reference undeclared anchors.
    ///
    /// An alias must reference an anchor that has been previously declared in the
    /// same document. Referencing a non-existent anchor is an error that will cause
    /// YAML parsers to fail or produce unexpected results.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// first: value1
    /// second: *missing
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// first: &anchor value1
    /// second: *anchor
    /// ```
    pub NoUndeclaredAliases {
        version: "next",
        name: "noUndeclaredAliases",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct UndeclaredAliasState {
    alias_name: String,
    range: TextRange,
}

impl Rule for NoUndeclaredAliases {
    type Query = Ast<YamlRoot>;
    type State = UndeclaredAliasState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut anchors = FxHashSet::<String>::default();
        let mut undeclared_aliases = Vec::new();

        for event in root.syntax().preorder_with_tokens(Direction::Next) {
            if let WalkEvent::Enter(element) = event {
                if let Some(token) = element.as_token() {
                    match token.kind() {
                        YamlSyntaxKind::ANCHOR_PROPERTY_LITERAL => {
                            let text = token.text_trimmed();
                            let name = text.strip_prefix('&').unwrap_or(text).to_string();
                            anchors.insert(name);
                        }
                        YamlSyntaxKind::ALIAS_LITERAL => {
                            let text = token.text_trimmed();
                            let name = text.strip_prefix('*').unwrap_or(text).to_string();
                            if !anchors.contains(&name) {
                                undeclared_aliases.push(UndeclaredAliasState {
                                    alias_name: name,
                                    range: token.text_trimmed_range(),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        undeclared_aliases.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "The alias references an undeclared anchor "<Emphasis>{&state.alias_name}</Emphasis>"."
                },
            )
            .note(markup! {
                "Declare the anchor before referencing it with an alias."
            }),
        )
    }
}
