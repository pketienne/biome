use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdParagraph;
use biome_rowan::AstNode;

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Disallow heading-like lines that exceed the valid h1-h6 range.
    ///
    /// Markdown only supports headings from level 1 (`#`) to level 6 (`######`).
    /// Lines starting with 7 or more `#` characters are not valid headings
    /// and are rendered as plain paragraphs, which is likely a mistake.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ####### This is not a valid heading
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ###### This is a valid h6 heading
    /// ```
    pub NoHeadingLikeParagraph {
        version: "next",
        name: "noHeadingLikeParagraph",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InvalidHeading {
    hash_count: usize,
}

impl Rule for NoHeadingLikeParagraph {
    type Query = Ast<MdParagraph>;
    type State = InvalidHeading;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let paragraph = ctx.query();
        let text = paragraph.syntax().text_trimmed().to_string();
        let trimmed = text.trim_start();

        // Count leading # characters
        let hash_count = trimmed.bytes().take_while(|&b| b == b'#').count();

        // Only flag lines with 7+ hashes that look like heading attempts
        // (followed by space or end of line)
        if hash_count >= 7 {
            let after_hashes = &trimmed[hash_count..];
            if after_hashes.is_empty() || after_hashes.starts_with(' ') {
                return Some(InvalidHeading { hash_count });
            }
        }

        None
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<MarkdownRuleAction> {
        let paragraph = ctx.query();
        let text = paragraph.syntax().text_trimmed().to_string();
        let trimmed = text.trim_start();
        let hash_count = trimmed.bytes().take_while(|&b| b == b'#').count();
        let leading_ws = &text[..text.len() - trimmed.len()];
        let corrected = format!("{}######{}", leading_ws, &trimmed[hash_count..]);
        let range = paragraph.syntax().text_trimmed_range();
        let mutation = make_text_replacement(&ctx.root(), range, &corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Convert to a valid heading level." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "Line starts with "{state.hash_count}" '#' characters, but headings only support levels 1-6."
                },
            )
            .note(markup! {
                "Use a heading level between 1 and 6, or remove the leading '#' characters."
            }),
        )
    }
}
