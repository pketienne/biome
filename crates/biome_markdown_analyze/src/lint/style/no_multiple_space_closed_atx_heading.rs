use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::fence_utils::FenceTracker;

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
    type Query = Ast<MdDocument>;
    type State = MultipleSpaceClosedAtx;
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

            let trimmed = line.trim_end();
            if !trimmed.starts_with('#') || !trimmed.ends_with('#') {
                continue;
            }

            let content = trimmed.trim_end_matches('#');
            if content.is_empty() {
                continue;
            }

            // Count trailing spaces before closing hashes
            let trailing_spaces = content.len() - content.trim_end().len();
            if trailing_spaces > 1 {
                let line_offset: usize =
                    text.lines().take(line_idx).map(|l| l.len() + 1).sum();
                signals.push(MultipleSpaceClosedAtx {
                    range: TextRange::new(
                        base + TextSize::from(line_offset as u32),
                        base + TextSize::from((line_offset + line.len()) as u32),
                    ),
                });
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        // Reconstruct the line text from the document source
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let line_start = u32::from(state.range.start() - base) as usize;
        let line_end = u32::from(state.range.end() - base) as usize;
        let line_text = &text[line_start..line_end];
        let trimmed = line_text.trim_end();

        // Find the closing hashes and the extra spaces before them
        let closing_hash_count = trimmed.chars().rev().take_while(|&c| c == '#').count();
        let content = &trimmed[..trimmed.len() - closing_hash_count];
        let content_trimmed_len = content.trim_end().len();
        let trailing_spaces = content.len() - content_trimmed_len;
        if trailing_spaces <= 1 {
            return None;
        }
        // The extra spaces start after the trimmed content + 1 space
        let extra_space_start = line_start + content_trimmed_len + 1;
        let extra_space_end = line_start + content.len();
        let extra_range = TextRange::new(
            base + TextSize::from(extra_space_start as u32),
            base + TextSize::from(extra_space_end as u32),
        );

        let root = ctx.root();
        let token = root
            .syntax()
            .token_at_offset(extra_range.start())
            .right_biased()?;
        let token_text = token.text().to_string();
        let token_start = token.text_range().start();
        let rel_start = u32::from(extra_range.start() - token_start) as usize;
        let rel_end = std::cmp::min(
            u32::from(extra_range.end() - token_start) as usize,
            token_text.len(),
        );
        // Build replacement text with the extra spaces removed
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
            markup! { "Use a single space before the closing hashes." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
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
