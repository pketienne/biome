use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdHeader;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Disallow multiple spaces before closing hashes in closed ATX headings.
    ///
    /// If a heading uses closing hashes (e.g., `## Heading ##`), there
    /// should be only one space before the closing hashes.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ## Heading  ##
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ## Heading ##
    /// ```
    pub NoMultipleSpaceClosedAtxHeading {
        version: "next",
        name: "noMultipleSpaceClosedAtxHeading",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct MultipleSpaceClosedAtx {
    range: TextRange,
}

impl Rule for NoMultipleSpaceClosedAtxHeading {
    type Query = Ast<MdHeader>;
    type State = MultipleSpaceClosedAtx;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let header = ctx.query();
        let text = header.syntax().text_trimmed().to_string();
        let trimmed = text.trim_end();

        if !trimmed.starts_with('#') || !trimmed.ends_with('#') {
            return None;
        }

        let content = trimmed.trim_end_matches('#');
        if content.is_empty() {
            return None;
        }

        let trailing_spaces = content.len() - content.trim_end().len();
        if trailing_spaces <= 1 {
            return None;
        }

        let base = header.syntax().text_trimmed_range().start();
        let content_trimmed_len = content.trim_end().len();
        // Range of the extra spaces (keep one space, remove the rest)
        let range = TextRange::new(
            base + TextSize::from((content_trimmed_len + 1) as u32),
            base + TextSize::from(content.len() as u32),
        );

        Some(MultipleSpaceClosedAtx { range })
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, "")?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Use a single space before the closing hashes." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "Multiple spaces before closing hashes in ATX heading."
                },
            )
            .note(markup! {
                "Use only one space before the closing hash characters."
            }),
        )
    }
}
