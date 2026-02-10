use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use crate::MarkdownRuleAction;
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdParagraph;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Disallow trailing spaces used for hard line breaks.
    ///
    /// Markdown allows two or more trailing spaces at the end of a line to create
    /// a hard line break (`<br>`). This is hard to see and easy to add by accident.
    /// Use a trailing backslash (`\`) instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// First line with two trailing spaces
    /// Second line
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// First line with backslash\
    /// Second line
    /// ```
    pub NoTrailingHardBreakSpaces {
        version: "next",
        name: "noTrailingHardBreakSpaces",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct TrailingHardBreak {
    range: TextRange,
}

impl Rule for NoTrailingHardBreakSpaces {
    type Query = Ast<MdParagraph>;
    type State = TrailingHardBreak;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let paragraph = ctx.query();
        let text = paragraph.syntax().text_with_trivia().to_string();
        let base = paragraph.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut offset = 0usize;

        for line in text.lines() {
            let trailing_spaces = line.bytes().rev().take_while(|&b| b == b' ').count();
            if trailing_spaces >= 2 {
                let space_start = offset + line.len() - trailing_spaces;
                let space_end = offset + line.len();
                signals.push(TrailingHardBreak {
                    range: TextRange::new(
                        base + TextSize::from(space_start as u32),
                        base + TextSize::from(space_end as u32),
                    ),
                });
            }
            offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, "\\")?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Replace trailing spaces with a backslash." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Trailing spaces used for hard line breaks."
                },
            )
            .note(markup! {
                "Use a trailing backslash instead of spaces for hard breaks."
            }),
        )
    }
}
