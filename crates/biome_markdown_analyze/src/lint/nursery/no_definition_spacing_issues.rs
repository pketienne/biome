use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Disallow spacing issues in link reference definitions.
    ///
    /// Definitions should not have extra whitespace between the label,
    /// colon, and URL.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [foo]:   https://example.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// ```
    pub NoDefinitionSpacingIssues {
        version: "next",
        name: "noDefinitionSpacingIssues",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct DefinitionSpacingIssue {
    range: TextRange,
}

impl Rule for NoDefinitionSpacingIssues {
    type Query = Ast<MdDocument>;
    type State = DefinitionSpacingIssue;
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

            if !tracker.is_inside_fence() {
                let trimmed = line.trim_start();
                let indent = line.len() - trimmed.len();

                // Check if this looks like a definition
                if indent <= 3 && trimmed.starts_with('[') {
                    if let Some(bracket_end) = trimmed.find("]:") {
                        let after_colon = &trimmed[bracket_end + 2..];
                        // Check for multiple spaces/tabs after ":"
                        let space_count = after_colon
                            .bytes()
                            .take_while(|&b| b == b' ' || b == b'\t')
                            .count();

                        if space_count > 1 {
                            signals.push(DefinitionSpacingIssue {
                                range: TextRange::new(
                                    base + TextSize::from(offset as u32),
                                    base + TextSize::from((offset + line.len()) as u32),
                                ),
                            });
                        }
                    }
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Extra whitespace in link reference definition."
                },
            )
            .note(markup! {
                "Use a single space after the colon in definitions."
            }),
        )
    }
}
