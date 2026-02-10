use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use crate::MarkdownRuleAction;
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdLinkBlock, MarkdownSyntaxKind};
use biome_rowan::{AstNode, BatchMutationExt, TextRange};

declare_lint_rule! {
    /// Disallow spacing issues in link reference definitions.
    ///
    /// Definitions should not have extra whitespace between the label,
    /// colon, and URL.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [foo]:   https://example.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// ```
    pub NoDefinitionSpacingIssues {
        version: "next",
        name: "noDefinitionSpacingIssues",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct DefinitionSpacingIssue {
    range: TextRange,
}

impl Rule for NoDefinitionSpacingIssues {
    type Query = Ast<MdLinkBlock>;
    type State = DefinitionSpacingIssue;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let link_block = ctx.query();
        let colon = link_block.colon_token().ok()?;

        // Count trailing whitespace after the colon
        let trailing_len = colon.trailing_trivia().text().len();

        if trailing_len > 1 {
            Some(DefinitionSpacingIssue {
                range: link_block.syntax().text_trimmed_range(),
            })
        } else {
            None
        }
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<MarkdownRuleAction> {
        let link_block = ctx.query();
        let colon = link_block.colon_token().ok()?;

        // Replace the colon token with one that has exactly one trailing space
        let new_colon = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            MarkdownSyntaxKind::COLON,
            ": ",
            [],
            [biome_rowan::TriviaPiece::whitespace(1)],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(colon.into(), new_colon.into());
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Normalize definition spacing." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Extra whitespace in link reference definition."
                },
            )
            .note(markup! {
                "Use a single space after the colon in definitions."
            }),
        )
    }
}
