use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_emphasis_markers};

declare_lint_rule! {
    /// Disallow spaces inside emphasis markers.
    ///
    /// Spaces immediately after opening or before closing emphasis markers
    /// can prevent proper rendering in some parsers.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// This is * not emphasized * text.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// This is *emphasized* text.
    /// ```
    pub NoSpaceInEmphasis {
        version: "next",
        name: "noSpaceInEmphasis",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct SpaceInEmphasis {
    range: TextRange,
    is_opening: bool,
}

impl Rule for NoSpaceInEmphasis {
    type Query = Ast<MdDocument>;
    type State = SpaceInEmphasis;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                offset += line.len() + 1;
                continue;
            }

            let bytes = line.as_bytes();
            let code_spans = find_code_spans(line);
            let markers = find_emphasis_markers(line, &code_spans);

            for m in &markers {
                if m.is_opening {
                    // Check for space after opening marker
                    let after = m.start + m.count;
                    if after < bytes.len() && bytes[after] == b' ' {
                        signals.push(SpaceInEmphasis {
                            range: TextRange::new(
                                base + TextSize::from((offset + m.start) as u32),
                                base + TextSize::from((offset + after + 1) as u32),
                            ),
                            is_opening: true,
                        });
                    }
                } else {
                    // Check for space before closing marker
                    if m.start > 0 && bytes[m.start - 1] == b' ' {
                        signals.push(SpaceInEmphasis {
                            range: TextRange::new(
                                base + TextSize::from((offset + m.start - 1) as u32),
                                base + TextSize::from((offset + m.start + m.count) as u32),
                            ),
                            is_opening: false,
                        });
                    }
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let position = if state.is_opening { "after opening" } else { "before closing" };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Unexpected space "{position}" emphasis marker."
                },
            )
            .note(markup! {
                "Remove the space to ensure emphasis renders correctly."
            }),
        )
    }
}
