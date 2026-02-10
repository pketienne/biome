use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use crate::MarkdownRuleAction;
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdParagraph;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fix_utils::make_text_replacement;
use crate::utils::inline_utils::{find_matching_bracket, looks_like_url};

declare_lint_rule! {
    /// Disallow reversed link syntax.
    ///
    /// Detects `(text)[url]` which should be `[text](url)`. This is a common
    /// mistake when writing Markdown links.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// (click here)[https://example.com]
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [click here](https://example.com)
    /// ```
    pub NoReversedLinks {
        version: "next",
        name: "noReversedLinks",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct ReversedLink {
    range: TextRange,
    corrected: String,
}

impl Rule for NoReversedLinks {
    type Query = Ast<MdParagraph>;
    type State = ReversedLink;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let paragraph = ctx.query();
        let text = paragraph.syntax().text_trimmed().to_string();
        let start = paragraph.syntax().text_trimmed_range().start();
        let mut signals = Vec::new();

        let bytes = text.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'(' {
                if let Some(close_paren) = find_matching_bracket(bytes, i, b'(', b')') {
                    if close_paren + 1 < bytes.len() && bytes[close_paren + 1] == b'[' {
                        if let Some(close_bracket) =
                            find_matching_bracket(bytes, close_paren + 1, b'[', b']')
                        {
                            let paren_content = &text[i + 1..close_paren];
                            let bracket_content = &text[close_paren + 2..close_bracket];

                            if !paren_content.is_empty()
                                && !bracket_content.is_empty()
                                && looks_like_url(bracket_content)
                            {
                                let offset = TextSize::from(i as u32);
                                let len = TextSize::from((close_bracket - i + 1) as u32);
                                let corrected =
                                    format!("[{}]({})", paren_content, bracket_content);
                                signals.push(ReversedLink {
                                    range: TextRange::new(start + offset, start + offset + len),
                                    corrected,
                                });
                            }
                        }
                    }
                }
            }
            i += 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, &state.corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Fix the link syntax." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Reversed link syntax detected. Use "<Emphasis>"[text](url)"</Emphasis>" instead of "<Emphasis>"(text)[url]"</Emphasis>"."
                },
            )
            .note(markup! {
                "Swap the parentheses and brackets to fix the link syntax."
            }),
        )
    }
}
