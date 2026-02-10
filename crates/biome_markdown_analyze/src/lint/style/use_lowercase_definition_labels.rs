use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use crate::MarkdownRuleAction;
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdLinkBlock, MarkdownSyntaxKind};
use biome_rowan::{AstNode, BatchMutationExt, TextRange};

declare_lint_rule! {
    /// Enforce lowercase labels in link reference definitions.
    ///
    /// Definition labels should be lowercase for consistency.
    /// While label matching is case-insensitive in markdown,
    /// using lowercase labels avoids confusion.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [FOO]: https://example.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// ```
    pub UseLowercaseDefinitionLabels {
        version: "next",
        name: "useLowercaseDefinitionLabels",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct UppercaseLabel {
    range: TextRange,
    label: String,
}

impl Rule for UseLowercaseDefinitionLabels {
    type Query = Ast<MdLinkBlock>;
    type State = UppercaseLabel;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let link_block = ctx.query();
        let label_text = link_block.label().syntax().text_trimmed().to_string();

        if label_text != label_text.to_lowercase() {
            Some(UppercaseLabel {
                range: link_block.syntax().text_trimmed_range(),
                label: label_text,
            })
        } else {
            None
        }
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<MarkdownRuleAction> {
        let link_block = ctx.query();
        let mut mutation = ctx.root().begin();

        // Replace each label token with its lowercase equivalent
        for token in link_block
            .label()
            .syntax()
            .descendants_with_tokens(biome_rowan::Direction::Next)
        {
            if let biome_rowan::SyntaxElement::Token(token) = token {
                if token.kind() == MarkdownSyntaxKind::MD_TEXTUAL_LITERAL {
                    let lower = token.text_trimmed().to_lowercase();
                    if lower != token.text_trimmed() {
                        let new_token =
                            biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
                                token.kind(),
                                &lower,
                                [],
                                [],
                            );
                        mutation.replace_element_discard_trivia(
                            token.into(),
                            new_token.into(),
                        );
                    }
                }
            }
        }

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Lowercase the definition label." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Definition label \""{ &state.label }"\" should be lowercase."
                },
            )
            .note(markup! {
                "Use lowercase labels for consistency."
            }),
        )
    }
}
