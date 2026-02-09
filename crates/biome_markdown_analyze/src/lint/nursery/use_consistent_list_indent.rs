use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::list_utils::collect_list_blocks;

declare_lint_rule! {
    /// Enforce consistent indentation for list items at the same level.
    ///
    /// All list items within the same list block should have the same
    /// indentation level.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// - item one
    ///   - item two
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
    pub UseConsistentListIndent {
        version: "next",
        name: "useConsistentListIndent",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct InconsistentListIndent {
    range: TextRange,
    expected: usize,
    actual: usize,
}

impl Rule for UseConsistentListIndent {
    type Query = Ast<MdDocument>;
    type State = InconsistentListIndent;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let blocks = collect_list_blocks(&text);
        let mut signals = Vec::new();

        for block in &blocks {
            if block.items.len() < 2 {
                continue;
            }

            // Find the most common indent level in this block
            let first_indent = block.items[0].indent;

            for item in block.items.iter().skip(1) {
                // Only flag items that should be at the same level as first
                // (same indent depth) but have different indent
                if item.indent != first_indent
                    && item.indent < first_indent + 2
                    && item.indent > first_indent.saturating_sub(2)
                {
                    signals.push(InconsistentListIndent {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        expected: first_indent,
                        actual: item.indent,
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
                    "Expected indentation of "{state.expected}" but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use consistent indentation for list items at the same level."
            }),
        )
    }
}
