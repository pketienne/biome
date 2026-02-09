use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_ordered_list_marker_value::UseConsistentOrderedListMarkerValueOptions;

use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Enforce consistent ordered list marker values.
    ///
    /// Ordered list markers can use incrementing numbers (`1, 2, 3`)
    /// or all ones (`1, 1, 1`). This rule enforces consistency.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"ordered"` (default):
    ///
    /// ```md
    /// 1. first
    /// 1. second
    /// 1. third
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// 1. first
    /// 2. second
    /// 3. third
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which numbering style to enforce. Default: `"ordered"`.
    pub UseConsistentOrderedListMarkerValue {
        version: "next",
        name: "useConsistentOrderedListMarkerValue",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentMarkerValue {
    range: TextRange,
    expected: usize,
    actual: usize,
}

impl Rule for UseConsistentOrderedListMarkerValue {
    type Query = Ast<MdDocument>;
    type State = InconsistentMarkerValue;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentOrderedListMarkerValueOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let items = collect_list_items(&text);
        let mut signals = Vec::new();

        // Only look at ordered items at indent 0 (top-level)
        let ordered_items: Vec<_> = items
            .iter()
            .filter(|i| i.marker_kind.is_ordered())
            .collect();

        if ordered_items.is_empty() {
            return signals;
        }

        // Group consecutive ordered items into sequences
        let mut sequences: Vec<Vec<&crate::utils::list_utils::ListItem>> = Vec::new();
        let mut current_seq = vec![ordered_items[0]];

        for item in ordered_items.iter().skip(1) {
            let prev = current_seq.last().unwrap();
            if item.line_index <= prev.line_index + 2 && item.indent == prev.indent {
                current_seq.push(item);
            } else {
                sequences.push(current_seq);
                current_seq = vec![item];
            }
        }
        sequences.push(current_seq);

        for seq in &sequences {
            if seq.len() < 2 {
                continue;
            }

            for (idx, item) in seq.iter().enumerate() {
                // Parse the number from the marker
                let num: usize = item
                    .marker
                    .trim_end_matches('.')
                    .trim_end_matches(')')
                    .parse()
                    .unwrap_or(0);

                let expected_num = match style {
                    "one" => 1,
                    _ => idx + 1, // "ordered"
                };

                if num != expected_num {
                    signals.push(InconsistentMarkerValue {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        expected: expected_num,
                        actual: num,
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
                    "Expected ordered list value "{state.expected}" but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use consistent ordered list marker values."
            }),
        )
    }
}
