use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

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
