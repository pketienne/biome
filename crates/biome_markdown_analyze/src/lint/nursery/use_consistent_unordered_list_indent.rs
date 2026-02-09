use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Enforce consistent indentation for nested unordered list items.
    ///
    /// Nested unordered list items should use a consistent number of
    /// spaces for indentation (typically 2 or 4).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// - item
    ///    - odd indent
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - item
    ///   - nested item
    /// ```
    pub UseConsistentUnorderedListIndent {
        version: "next",
        name: "useConsistentUnorderedListIndent",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct OddIndentation {
    range: TextRange,
    indent: usize,
}

impl Rule for UseConsistentUnorderedListIndent {
    type Query = Ast<MdDocument>;
    type State = OddIndentation;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let items = collect_list_items(&text);
        let mut signals = Vec::new();

        for item in &items {
            if item.marker_kind.is_unordered() && item.indent > 0 {
                // Indent should be a multiple of 2
                if item.indent % 2 != 0 {
                    signals.push(OddIndentation {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        indent: item.indent,
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
                    "Unordered list item has odd indentation of "{state.indent}" spaces."
                },
            )
            .note(markup! {
                "Use a consistent multiple of 2 spaces for nested list indentation."
            }),
        )
    }
}
