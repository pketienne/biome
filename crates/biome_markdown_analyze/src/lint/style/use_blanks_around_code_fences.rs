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
    /// Require blank lines around fenced code blocks.
    ///
    /// Fenced code blocks should be surrounded by blank lines for readability.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ````md
    /// Some text
    /// ```js
    /// code
    /// ```
    /// More text
    /// ````
    ///
    /// ### Valid
    ///
    /// ````md
    /// Some text
    ///
    /// ```js
    /// code
    /// ```
    ///
    /// More text
    /// ````
    pub UseBlanksAroundCodeFences {
        version: "next",
        name: "useBlanksAroundCodeFences",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct MissingBlankAroundFence {
    range: TextRange,
    position: &'static str,
    corrected: String,
}

impl Rule for UseBlanksAroundCodeFences {
    type Query = Ast<MdDocument>;
    type State = MissingBlankAroundFence;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;
        let mut fence_open_line: Option<usize> = None;

        for (line_idx, line) in lines.iter().enumerate() {
            let was_inside = tracker.is_inside_fence();
            let fence_result = tracker.process_line(line_idx, line);

            if fence_result.is_some() && !was_inside {
                // Opening fence
                fence_open_line = Some(line_idx);
                // Check that previous line is blank (or this is the first line)
                if line_idx > 0 && !is_blank_line(lines[line_idx - 1]) {
                    signals.push(MissingBlankAroundFence {
                        range: TextRange::new(
                            base + TextSize::from(offset as u32),
                            base + TextSize::from((offset + line.len()) as u32),
                        ),
                        position: "before",
                        corrected: format!("\n{}", line),
                    });
                }
            } else if was_inside && !tracker.is_inside_fence() {
                // Closing fence
                // Check that next line is blank (or this is the last line)
                if line_idx + 1 < lines.len() && !is_blank_line(lines[line_idx + 1]) {
                    signals.push(MissingBlankAroundFence {
                        range: TextRange::new(
                            base + TextSize::from(offset as u32),
                            base + TextSize::from((offset + line.len()) as u32),
                        ),
                        position: "after",
                        corrected: format!("{}\n", line),
                    });
                }
                fence_open_line = None;
            }

            offset += line.len() + 1;
        }

        let _ = fence_open_line;
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
            markup! { "Insert blank line "{state.position}" code fence." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Missing blank line "{state.position}" code fence."
                },
            )
            .note(markup! {
                "Add a blank line around fenced code blocks for readability."
            }),
        )
    }
}
