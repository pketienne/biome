use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::fence_utils::FenceTracker;
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
    type Query = Ast<MdDocument>;
    type State = SpaceInCode;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                offset += line.len() + 1;
                continue;
            }

            let code_spans = find_code_spans(line);
            let bytes = line.as_bytes();

            for span in &code_spans {
                // Content is between the backtick delimiters
                let content_start = span.open + span.backtick_count;
                let content_end = span.close - span.backtick_count;

                if content_start >= content_end {
                    continue;
                }

                let content = &line[content_start..content_end];
                // Allow spaces if content is only spaces (`` ` `` pattern)
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
        let root = ctx.root();

        // Collect all tokens overlapping the range
        let mut token = root
            .syntax()
            .token_at_offset(state.range.start())
            .right_biased()?;
        let mut tokens = vec![token.clone()];
        while token.text_range().end() < state.range.end() {
            token = token.next_token()?;
            tokens.push(token.clone());
        }

        // Build replacement: prefix from first token + corrected span + suffix from last token
        let first = &tokens[0];
        let last = tokens.last()?;
        let prefix_len = u32::from(state.range.start() - first.text_range().start()) as usize;
        let suffix_start = u32::from(state.range.end() - last.text_range().start()) as usize;
        let prefix = &first.text()[..prefix_len];
        let suffix = &last.text()[suffix_start..];
        let new_text = format!("{}{}{}", prefix, state.corrected_span, suffix);

        let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            first.kind(),
            &new_text,
            [],
            [],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(first.clone().into(), new_token.into());
        for t in &tokens[1..] {
            let empty = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
                t.kind(),
                "",
                [],
                [],
            );
            mutation.replace_element_discard_trivia(t.clone().into(), empty.into());
        }

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
