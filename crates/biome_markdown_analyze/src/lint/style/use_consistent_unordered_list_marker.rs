use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_unordered_list_marker::UseConsistentUnorderedListMarkerOptions;

use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Enforce consistent unordered list marker style.
    ///
    /// Unordered lists can use `-`, `*`, or `+` as markers.
    /// This rule enforces a consistent marker.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"-"` (default):
    ///
    /// ```md
    /// * item one
    /// * item two
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - item one
    /// - item two
    /// ```
    ///
    /// ## Options
    ///
    /// ### `marker`
    ///
    /// Which marker to enforce. Default: `"-"`.
    pub UseConsistentUnorderedListMarker {
        version: "next",
        name: "useConsistentUnorderedListMarker",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentListMarker {
    range: TextRange,
    expected: String,
    actual: String,
}

impl Rule for UseConsistentUnorderedListMarker {
    type Query = Ast<MdDocument>;
    type State = InconsistentListMarker;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentUnorderedListMarkerOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let marker_opt = ctx.options().marker();
        let items = collect_list_items(&text);

        let mut signals = Vec::new();

        // For "consistent" mode, use first unordered item's marker
        let expected_marker = if marker_opt == "consistent" {
            items
                .iter()
                .find(|i| i.marker_kind.is_unordered())
                .map(|i| i.marker.clone())
        } else {
            Some(marker_opt.to_string())
        };

        let expected_marker: String = match expected_marker {
            Some(m) => m,
            None => return signals,
        };

        for item in &items {
            if item.marker_kind.is_unordered() && item.marker != expected_marker {
                signals.push(InconsistentListMarker {
                    range: TextRange::new(
                        base + TextSize::from(item.byte_offset as u32),
                        base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                    ),
                    expected: expected_marker.clone(),
                    actual: item.marker.clone(),
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
                    "Expected \""{ &state.expected }"\" but found \""{ &state.actual }"\" as list marker."
                },
            )
            .note(markup! {
                "Use a consistent unordered list marker."
            }),
        )
    }
}
