use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

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
