use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

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
