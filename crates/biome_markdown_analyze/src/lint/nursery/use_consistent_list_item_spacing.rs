use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_list_item_spacing::UseConsistentListItemSpacingOptions;

use crate::utils::list_utils::collect_list_blocks;

declare_lint_rule! {
    /// Enforce consistent spacing between list items.
    ///
    /// Lists should either be compact (no blank lines) or loose
    /// (blank lines between every item), but not mixed.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// - item one
    ///
    /// - item two
    /// - item three
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - item one
    /// - item two
    /// - item three
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which style to enforce. Default: `"consistent"`.
    pub UseConsistentListItemSpacing {
        version: "next",
        name: "useConsistentListItemSpacing",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentSpacing {
    range: TextRange,
    expected: &'static str,
    actual: &'static str,
}

impl Rule for UseConsistentListItemSpacing {
    type Query = Ast<MdDocument>;
    type State = InconsistentSpacing;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentListItemSpacingOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let blocks = collect_list_blocks(&text);
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();

        for block in &blocks {
            if block.items.len() < 2 {
                continue;
            }

            // Determine if gaps between items have blank lines
            let mut has_loose = false;
            let mut has_compact = false;

            for pair in block.items.windows(2) {
                let gap_has_blank = (pair[0].line_index + 1..pair[1].line_index)
                    .any(|l| l < lines.len() && lines[l].trim().is_empty());
                if gap_has_blank {
                    has_loose = true;
                } else if pair[1].line_index == pair[0].line_index + 1 {
                    has_compact = true;
                }
            }

            let expected_style = match style {
                "compact" => "compact",
                "loose" => "loose",
                _ => {
                    // consistent: use the first gap's style
                    if has_loose { "loose" } else { "compact" }
                }
            };

            // Only flag if there's a mix, or if forcing a specific style
            if style == "consistent" && !(has_loose && has_compact) {
                continue;
            }

            for pair in block.items.windows(2) {
                let gap_has_blank = (pair[0].line_index + 1..pair[1].line_index)
                    .any(|l| l < lines.len() && lines[l].trim().is_empty());
                let is_compact = !gap_has_blank && pair[1].line_index == pair[0].line_index + 1;

                let actual = if gap_has_blank { "loose" } else if is_compact { "compact" } else { continue };

                if actual != expected_style {
                    signals.push(InconsistentSpacing {
                        range: TextRange::new(
                            base + TextSize::from(pair[1].byte_offset as u32),
                            base + TextSize::from(
                                (pair[1].byte_offset + pair[1].byte_len) as u32,
                            ),
                        ),
                        expected: if expected_style == "compact" {
                            "compact"
                        } else {
                            "loose"
                        },
                        actual: if actual == "compact" { "compact" } else { "loose" },
                    });
                }
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
                    "Expected "{state.expected}" spacing but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use consistent spacing between list items."
            }),
        )
    }
}
