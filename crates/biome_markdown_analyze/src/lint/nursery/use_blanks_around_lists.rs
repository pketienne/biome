use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::line_utils::is_blank_line;
use crate::utils::list_utils::collect_list_blocks;

declare_lint_rule! {
    /// Enforce blank lines around list blocks.
    ///
    /// Lists should be surrounded by blank lines to separate them
    /// from adjacent content.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// Text
    /// - item
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// Text
    ///
    /// - item
    /// ```
    pub UseBlanksAroundLists {
        version: "next",
        name: "useBlanksAroundLists",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct MissingBlankAroundList {
    range: TextRange,
    position: &'static str,
}

impl Rule for UseBlanksAroundLists {
    type Query = Ast<MdDocument>;
    type State = MissingBlankAroundList;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let blocks = collect_list_blocks(&text);
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();

        for block in &blocks {
            // Check line before the block
            if block.start_line > 0 {
                let prev_line = lines[block.start_line - 1];
                if !is_blank_line(prev_line) {
                    signals.push(MissingBlankAroundList {
                        range: TextRange::new(
                            base + TextSize::from(block.byte_offset as u32),
                            base + TextSize::from(
                                (block.byte_offset + block.byte_len) as u32,
                            ),
                        ),
                        position: "before",
                    });
                }
            }

            // Check line after the block
            let next_line_idx = block.end_line + 1;
            if next_line_idx < lines.len() {
                let next_line = lines[next_line_idx];
                if !is_blank_line(next_line) {
                    signals.push(MissingBlankAroundList {
                        range: TextRange::new(
                            base + TextSize::from(block.byte_offset as u32),
                            base + TextSize::from(
                                (block.byte_offset + block.byte_len) as u32,
                            ),
                        ),
                        position: "after",
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
                    "Missing blank line "{state.position}" list block."
                },
            )
            .note(markup! {
                "Add a blank line before and after list blocks."
            }),
        )
    }
}
