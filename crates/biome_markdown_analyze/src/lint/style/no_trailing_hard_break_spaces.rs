use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use crate::MarkdownRuleAction;
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;

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
    type Query = Ast<MdDocument>;
    type State = TrailingHardBreak;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_trimmed().to_string();
        let base = document.syntax().text_trimmed_range().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if !tracker.is_inside_fence() {
                let trailing_spaces = line.bytes().rev().take_while(|&b| b == b' ').count();
                // Two or more trailing spaces create a hard break
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
            }

            offset += line.len() + 1; // +1 for newline
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        // Trailing spaces range - replace with backslash
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
        let new_text = format!("{}\\{}", prefix, suffix);
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
