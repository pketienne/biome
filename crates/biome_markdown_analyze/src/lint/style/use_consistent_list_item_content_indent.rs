use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Enforce consistent content indentation for continuation lines.
    ///
    /// When a list item's content wraps to the next line, the
    /// continuation should be indented to align with the first line's
    /// content.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// - item
    /// not indented continuation
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - item
    ///   continuation
    /// ```
    pub UseConsistentListItemContentIndent {
        version: "next",
        name: "useConsistentListItemContentIndent",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct BadContentIndent {
    range: TextRange,
    expected: usize,
    actual: usize,
}

impl Rule for UseConsistentListItemContentIndent {
    type Query = Ast<MdDocument>;
    type State = BadContentIndent;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let items = collect_list_items(&text);
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();

        // Build a set of line indices that are list items
        let item_lines: std::collections::HashSet<usize> =
            items.iter().map(|i| i.line_index).collect();

        for item in &items {
            // Expected content indent: indent + marker length + content_offset
            let expected_indent = item.indent + item.marker.len() + item.content_offset;

            // Check continuation lines (lines after this item, before the next item or blank)
            for line_idx in (item.line_index + 1)..lines.len() {
                if item_lines.contains(&line_idx) {
                    break;
                }
                let line = lines[line_idx];
                if line.trim().is_empty() {
                    break;
                }

                let actual_indent = line
                    .bytes()
                    .take_while(|&b| b == b' ' || b == b'\t')
                    .count();

                if actual_indent != expected_indent {
                    // Compute byte offset for this line
                    let line_byte_offset: usize =
                        lines[..line_idx].iter().map(|l| l.len() + 1).sum();
                    signals.push(BadContentIndent {
                        range: TextRange::new(
                            base + TextSize::from(line_byte_offset as u32),
                            base + TextSize::from((line_byte_offset + line.len()) as u32),
                        ),
                        expected: expected_indent,
                        actual: actual_indent,
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
                    "Expected content indentation of "{state.expected}" but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Align continuation content with the first line of the list item."
            }),
        )
    }
}
