use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_list_item_indent::UseConsistentListItemIndentOptions;

use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Enforce consistent spacing between list marker and content.
    ///
    /// The space between the marker and content should follow a consistent style.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"one"` (default), extra spaces are flagged:
    ///
    /// ```md
    /// -  item with extra space
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - item with one space
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which indent style to enforce. Default: `"one"`.
    pub UseConsistentListItemIndent {
        version: "next",
        name: "useConsistentListItemIndent",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct InconsistentItemIndent {
    range: TextRange,
    spaces: usize,
}

impl Rule for UseConsistentListItemIndent {
    type Query = Ast<MdDocument>;
    type State = InconsistentItemIndent;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentListItemIndentOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let items = collect_list_items(&text);
        let mut signals = Vec::new();

        let expected_offset = match style {
            "one" => Some(1usize),
            "tab" => Some(4usize),
            _ => Some(1usize),
        };

        for item in &items {
            if let Some(expected) = expected_offset {
                if item.content_offset != expected {
                    signals.push(InconsistentItemIndent {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        spaces: item.content_offset,
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
                    "List item has "{state.spaces}" space(s) between marker and content."
                },
            )
            .note(markup! {
                "Use consistent spacing between list marker and content."
            }),
        )
    }
}
