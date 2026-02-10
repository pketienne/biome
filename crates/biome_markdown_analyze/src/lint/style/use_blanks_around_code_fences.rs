use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdFencedCodeBlock;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;

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
    type Query = Ast<MdFencedCodeBlock>;
    type State = MissingBlankAroundFence;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let code_block = ctx.query();
        let root = ctx.root();
        let root_text = root.syntax().text_with_trivia().to_string();
        let root_start = root.syntax().text_range_with_trivia().start();

        let l_fence = match code_block.l_fence_token().ok() {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut signals = Vec::new();

        // Find the opening fence byte offset in root text
        let fence_offset =
            u32::from(l_fence.text_trimmed_range().start() - root_start) as usize;

        // Check if the line immediately before the opening fence is non-blank
        let before = &root_text[..fence_offset];
        if let Some(last_nl) = before.rfind('\n') {
            // Find the line before the fence (between previous \n and last_nl)
            let prev_content = &before[..last_nl];
            let prev_line_start = prev_content.rfind('\n').map(|p| p + 1).unwrap_or(0);
            let prev_line = &before[prev_line_start..last_nl];
            if !prev_line.trim().is_empty() {
                // Compute the opening fence line range and text
                let line_end = root_text[fence_offset..]
                    .find('\n')
                    .map(|p| fence_offset + p)
                    .unwrap_or(root_text.len());
                let line_text = &root_text[fence_offset..line_end];
                signals.push(MissingBlankAroundFence {
                    range: TextRange::new(
                        root_start + TextSize::from(fence_offset as u32),
                        root_start + TextSize::from(line_end as u32),
                    ),
                    position: "before",
                    corrected: format!("\n{}", line_text),
                });
            }
        }

        // Check if the line after the closing fence is non-blank
        if let Some(r_fence) = code_block.r_fence_token().ok() {
            let close_offset =
                u32::from(r_fence.text_trimmed_range().start() - root_start) as usize;
            let close_line_end = root_text[close_offset..]
                .find('\n')
                .map(|p| close_offset + p)
                .unwrap_or(root_text.len());
            let next_line_start = close_line_end + 1;
            if next_line_start < root_text.len() {
                let rest = &root_text[next_line_start..];
                let next_line_end = rest.find('\n').unwrap_or(rest.len());
                let next_line = &rest[..next_line_end];
                if !next_line.trim().is_empty() {
                    let line_text = &root_text[close_offset..close_line_end];
                    signals.push(MissingBlankAroundFence {
                        range: TextRange::new(
                            root_start + TextSize::from(close_offset as u32),
                            root_start + TextSize::from(close_line_end as u32),
                        ),
                        position: "after",
                        corrected: format!("{}\n", line_text),
                    });
                }
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
