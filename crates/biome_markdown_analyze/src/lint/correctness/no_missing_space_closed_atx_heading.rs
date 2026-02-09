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
    /// Require a space before the closing hashes in closed ATX headings.
    ///
    /// If a heading uses closing hashes (e.g., `## Heading ##`), there
    /// should be a space before the closing hashes.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ## Heading##
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ## Heading ##
    /// ```
    pub NoMissingSpaceClosedAtxHeading {
        version: "next",
        name: "noMissingSpaceClosedAtxHeading",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct MissingSpaceClosedAtx {
    range: TextRange,
}

impl Rule for NoMissingSpaceClosedAtxHeading {
    type Query = Ast<MdDocument>;
    type State = MissingSpaceClosedAtx;
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
            if !trimmed.starts_with('#') {
                continue;
            }

            // Check if it ends with closing hashes
            if !trimmed.ends_with('#') {
                continue;
            }

            // Find the closing hash sequence
            let content = trimmed.trim_end_matches('#');
            if content.is_empty() {
                continue;
            }

            // Check if there's a space before the closing hashes
            if !content.ends_with(' ') {
                let line_offset: usize =
                    text.lines().take(line_idx).map(|l| l.len() + 1).sum();
                signals.push(MissingSpaceClosedAtx {
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

        // Find the closing hashes and the insertion point
        let closing_hash_count = trimmed.chars().rev().take_while(|&c| c == '#').count();
        let content = &trimmed[..trimmed.len() - closing_hash_count];
        if content.ends_with(' ') {
            return None;
        }
        // The insertion point is right before the closing hashes
        let insert_offset = line_start + content.len();
        let insert_pos = base + TextSize::from(insert_offset as u32);

        let root = ctx.root();
        let token = root
            .syntax()
            .token_at_offset(insert_pos)
            .right_biased()?;
        let token_text = token.text().to_string();
        let token_start = token.text_range().start();
        let rel_pos = u32::from(insert_pos - token_start) as usize;
        // Insert a space at the insertion point
        let mut new_text = String::with_capacity(token_text.len() + 1);
        new_text.push_str(&token_text[..rel_pos]);
        new_text.push(' ');
        new_text.push_str(&token_text[rel_pos..]);
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
            markup! { "Add a space before the closing hashes." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Missing space before closing hashes in ATX heading."
                },
            )
            .note(markup! {
                "Add a space before the closing hash characters."
            }),
        )
    }
}
