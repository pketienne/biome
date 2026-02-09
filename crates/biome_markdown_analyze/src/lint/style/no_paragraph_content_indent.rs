use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::fence_utils::FenceTracker;
use crate::utils::line_utils::leading_indent;

declare_lint_rule! {
    /// Disallow indentation on paragraph content lines.
    ///
    /// Paragraph text should start at column 0 (no leading spaces).
    /// Indented text (4+ spaces) becomes a code block in CommonMark.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    ///   Indented paragraph text.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// Normal paragraph text.
    /// ```
    pub NoParagraphContentIndent {
        version: "next",
        name: "noParagraphContentIndent",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct IndentedParagraph {
    range: TextRange,
    indent: usize,
}

impl Rule for NoParagraphContentIndent {
    type Query = Ast<MdDocument>;
    type State = IndentedParagraph;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                continue;
            }

            // Skip blank lines and headings
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }

            // Skip blockquote lines
            if line.trim_start().starts_with('>') {
                continue;
            }

            // Skip list items (ordered and unordered)
            let trimmed = line.trim_start();
            if trimmed.starts_with("+ ")
                || trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
            {
                continue;
            }
            // Skip ordered list items
            if trimmed.len() > 1 && trimmed.as_bytes()[0].is_ascii_digit() {
                let rest = &trimmed[1..];
                if rest.starts_with(". ") || rest.starts_with(") ") {
                    continue;
                }
                // Multi-digit numbers
                if rest.len() > 1 && rest.as_bytes()[0].is_ascii_digit() {
                    let rest2 = &rest[1..];
                    if rest2.starts_with(". ") || rest2.starts_with(") ") {
                        continue;
                    }
                }
            }

            let indent = leading_indent(line);
            if indent > 0 && indent < 4 {
                // 1-3 spaces of indentation on plain paragraph text
                let line_offset: usize =
                    text.lines().take(line_idx).map(|l| l.len() + 1).sum();
                signals.push(IndentedParagraph {
                    range: TextRange::new(
                        base + TextSize::from(line_offset as u32),
                        base + TextSize::from((line_offset + line.len()) as u32),
                    ),
                    indent,
                });
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        // The range covers the full line; remove the leading indent (state.indent bytes).
        let indent_start = state.range.start();
        let indent_end = indent_start + TextSize::from(state.indent as u32);
        let root = ctx.root();
        let token = root
            .syntax()
            .token_at_offset(indent_start)
            .right_biased()?;
        let token_text = token.text().to_string();
        let token_start = token.text_range().start();
        let rel_start = u32::from(indent_start - token_start) as usize;
        let rel_end = u32::from(indent_end - token_start) as usize;
        let mut new_text = String::with_capacity(token_text.len());
        new_text.push_str(&token_text[..rel_start]);
        new_text.push_str(&token_text[rel_end..]);
        let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            token.kind(),
            &new_text,
            [],
            [],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(token.into(), new_token.into());
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove leading indentation." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Paragraph content has "{state.indent}" space(s) of indentation."
                },
            )
            .note(markup! {
                "Remove leading indentation from paragraph text."
            }),
        )
    }
}
