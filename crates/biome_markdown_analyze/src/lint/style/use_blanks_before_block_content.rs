use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;
use crate::utils::line_utils::is_blank_line;

declare_lint_rule! {
    /// Enforce blank lines before block-level content.
    ///
    /// Block-level elements (headings, code fences, blockquotes) should
    /// be preceded by a blank line.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ````md
    /// Some text.
    /// ```
    /// code
    /// ```
    /// ````
    ///
    /// ### Valid
    ///
    /// ````md
    /// Some text.
    ///
    /// ```
    /// code
    /// ```
    /// ````
    pub UseBlanksBeforeBlockContent {
        version: "next",
        name: "useBlanksBeforeBlockContent",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct MissingBlankBeforeBlock {
    range: TextRange,
    block_type: &'static str,
    corrected: String,
}

impl Rule for UseBlanksBeforeBlockContent {
    type Query = Ast<MdDocument>;
    type State = MissingBlankBeforeBlock;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        for (line_idx, line) in lines.iter().enumerate() {
            tracker.process_line(line_idx, line);

            if line_idx == 0 {
                continue;
            }

            let prev_line = lines[line_idx - 1];
            if is_blank_line(prev_line) || prev_line.trim().is_empty() {
                continue;
            }

            // Check if current line starts a block element
            let trimmed = line.trim_start();
            let block_type = if trimmed.starts_with('#') && !tracker.is_inside_fence() {
                Some("heading")
            } else if (trimmed.starts_with("```") || trimmed.starts_with("~~~"))
                && !tracker.is_inside_fence()
            {
                Some("code fence")
            } else if trimmed.starts_with("> ") && !tracker.is_inside_fence() {
                Some("blockquote")
            } else {
                None
            };

            if let Some(block_type) = block_type {
                // Don't flag if previous line is also a heading (back-to-back headings)
                if block_type == "heading" && prev_line.trim_start().starts_with('#') {
                    continue;
                }

                let line_offset: usize =
                    lines[..line_idx].iter().map(|l| l.len() + 1).sum();
                signals.push(MissingBlankBeforeBlock {
                    range: TextRange::new(
                        base + TextSize::from(line_offset as u32),
                        base + TextSize::from((line_offset + line.len()) as u32),
                    ),
                    block_type,
                    corrected: format!("\n{}", line),
                });
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let mut token = root
            .syntax()
            .token_at_offset(state.range.start())
            .right_biased()?;
        let mut tokens = vec![token.clone()];
        while token.text_range().end() < state.range.end() {
            token = token.next_token()?;
            tokens.push(token.clone());
        }
        let first = &tokens[0];
        let last = tokens.last()?;
        let prefix_len = u32::from(state.range.start() - first.text_range().start()) as usize;
        let suffix_start = u32::from(state.range.end() - last.text_range().start()) as usize;
        let prefix = &first.text()[..prefix_len];
        let suffix = &last.text()[suffix_start..];
        let new_text = format!("{}{}{}", prefix, state.corrected, suffix);
        let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            first.kind(),
            &new_text,
            [],
            [],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(first.clone().into(), new_token.into());
        for t in &tokens[1..] {
            let empty =
                biome_markdown_syntax::MarkdownSyntaxToken::new_detached(t.kind(), "", [], []);
            mutation.replace_element_discard_trivia(t.clone().into(), empty.into());
        }
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Insert blank line before "{state.block_type}"." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Missing blank line before "{state.block_type}"."
                },
            )
            .note(markup! {
                "Add a blank line before block-level content."
            }),
        )
    }
}
