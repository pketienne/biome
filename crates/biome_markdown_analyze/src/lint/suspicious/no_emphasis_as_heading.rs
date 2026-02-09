use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_emphasis_markers};

declare_lint_rule! {
    /// Disallow using emphasis or bold as a heading substitute.
    ///
    /// A paragraph that consists entirely of bold or italic text
    /// likely should be a proper heading instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ````md
    /// **This should be a heading**
    /// ````
    ///
    /// ### Valid
    ///
    /// ```md
    /// ## This is a heading
    /// ```
    pub NoEmphasisAsHeading {
        version: "next",
        name: "noEmphasisAsHeading",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct EmphasisHeading {
    range: TextRange,
    corrected: String,
}

impl Rule for NoEmphasisAsHeading {
    type Query = Ast<MdDocument>;
    type State = EmphasisHeading;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let lines: Vec<&str> = text.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                continue;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check if the line is a standalone bold/emphasis paragraph
            // Pattern: **text** or __text__ or *text* or _text_ as entire line
            let is_emphasis_line = (trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() > 4)
                || (trimmed.starts_with("__") && trimmed.ends_with("__") && trimmed.len() > 4);

            if !is_emphasis_line {
                continue;
            }

            // Verify the line before is blank or start of document (standalone paragraph)
            let prev_blank = line_idx == 0
                || lines[line_idx - 1].trim().is_empty();
            let next_blank = line_idx + 1 >= lines.len()
                || lines[line_idx + 1].trim().is_empty();

            if prev_blank && next_blank {
                // Check it has actual emphasis markers, not just asterisks
                let code_spans = find_code_spans(line);
                let markers = find_emphasis_markers(line, &code_spans);
                if markers.len() >= 2 {
                    let line_offset: usize =
                        lines[..line_idx].iter().map(|l| l.len() + 1).sum();
                    // Extract heading text by stripping emphasis markers
                    let inner = if trimmed.starts_with("**") && trimmed.ends_with("**") {
                        &trimmed[2..trimmed.len() - 2]
                    } else if trimmed.starts_with("__") && trimmed.ends_with("__") {
                        &trimmed[2..trimmed.len() - 2]
                    } else {
                        trimmed
                    };
                    let corrected = format!("## {}", inner);
                    signals.push(EmphasisHeading {
                        range: TextRange::new(
                            base + TextSize::from(line_offset as u32),
                            base + TextSize::from((line_offset + line.len()) as u32),
                        ),
                        corrected,
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
            markup! { "Convert to ATX heading." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Emphasis used as a heading substitute."
                },
            )
            .note(markup! {
                "Use a proper heading (e.g. ## Heading) instead of bold/italic text."
            }),
        )
    }
}
