use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdParagraph;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::fix_utils::make_text_replacement;
use crate::utils::inline_utils::find_code_spans;

declare_lint_rule! {
    /// Disallow spaces at the edges of inline code spans.
    ///
    /// Inline code spans should not have leading or trailing spaces,
    /// unless the code content itself requires them (e.g., `` ` `` to render
    /// a single backtick).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// This is ` code ` text.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// This is `code` text.
    /// ```
    pub NoSpaceInCode {
        version: "next",
        name: "noSpaceInCode",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct SpaceInCode {
    range: TextRange,
    corrected_span: String,
}

impl Rule for NoSpaceInCode {
    type Query = Ast<MdParagraph>;
    type State = SpaceInCode;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let paragraph = ctx.query();
        let text = paragraph.syntax().text_trimmed().to_string();
        let base = paragraph.syntax().text_trimmed_range().start();
        let mut signals = Vec::new();
        let mut offset = 0usize;

        for line in text.lines() {
            let code_spans = find_code_spans(line);
            let bytes = line.as_bytes();

            for span in &code_spans {
                let content_start = span.open + span.backtick_count;
                let content_end = span.close - span.backtick_count;

                if content_start >= content_end {
                    continue;
                }

                let content = &line[content_start..content_end];
                if content.trim().is_empty() {
                    continue;
                }

                let has_leading_space = bytes[content_start] == b' ';
                let has_trailing_space = bytes[content_end - 1] == b' ';

                if has_leading_space || has_trailing_space {
                    let delimiter = &line[span.open..span.open + span.backtick_count];
                    let trimmed = content.trim();
                    let corrected_span = format!("{}{}{}", delimiter, trimmed, delimiter);
                    signals.push(SpaceInCode {
                        range: TextRange::new(
                            base + TextSize::from((offset + span.open) as u32),
                            base + TextSize::from((offset + span.close) as u32),
                        ),
                        corrected_span,
                    });
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, &state.corrected_span)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove spaces from code span edges." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Unexpected space at the edge of inline code span."
                },
            )
            .note(markup! {
                "Remove leading or trailing spaces from the code span content."
            }),
        )
    }
}
