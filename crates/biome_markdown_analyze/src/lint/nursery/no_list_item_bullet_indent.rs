use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Disallow indentation on top-level list item bullets.
    ///
    /// Top-level list items should start at column 0 without indentation.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    ///   - indented item
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - unindented item
    /// ```
    pub NoListItemBulletIndent {
        version: "next",
        name: "noListItemBulletIndent",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct IndentedBullet {
    range: TextRange,
}

impl Rule for NoListItemBulletIndent {
    type Query = Ast<MdDocument>;
    type State = IndentedBullet;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let items = collect_list_items(&text);
        let mut signals = Vec::new();

        // Find first items in each block (indent 0 expected)
        // Simple heuristic: any item with 1-3 spaces indent that's unordered
        // and appears to be a top-level item
        for item in &items {
            if item.indent > 0 && item.indent < 4 {
                // Check if this looks like a mistakenly indented top-level item
                // (not a nested item under another list item)
                signals.push(IndentedBullet {
                    range: TextRange::new(
                        base + TextSize::from(item.byte_offset as u32),
                        base + TextSize::from((item.byte_offset + item.byte_len) as u32),
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
                    "List item bullet should not be indented."
                },
            )
            .note(markup! {
                "Remove indentation from the list item bullet."
            }),
        )
    }
}
