use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdHeader;
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt};

use crate::MarkdownRuleAction;

declare_lint_rule! {
    /// Require a space after the hash characters in atx headings.
    ///
    /// Atx-style headings require a space between the hash characters and the
    /// heading text. Without the space, some parsers may not recognize the heading.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// #Heading without space
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Heading with space
    /// ```
    pub NoMissingSpaceAtxHeading {
        version: "next",
        name: "noMissingSpaceAtxHeading",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

impl Rule for NoMissingSpaceAtxHeading {
    type Query = Ast<MdHeader>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let header = ctx.query();
        let level = header.before().len();
        if level == 0 {
            return None;
        }
        let text = header.syntax().text_trimmed().to_string();
        if text.len() > level {
            let after_hashes = &text[level..];
            if !after_hashes.starts_with(' ') && !after_hashes.trim().is_empty() {
                return Some(());
            }
        }
        None
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<MarkdownRuleAction> {
        let header = ctx.query();
        let content = header.content()?;
        let first_token = content.syntax().first_token()?;
        let token_text = first_token.text().to_string();
        let new_text = format!(" {}", token_text);
        let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            first_token.kind(),
            &new_text,
            [],
            [],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(first_token.into(), new_token.into());
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Add a space after the hash characters." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "Missing space after hash characters in atx heading."
                },
            )
            .note(markup! {
                "Add a space between the hash characters and the heading text."
            }),
        )
    }
}
