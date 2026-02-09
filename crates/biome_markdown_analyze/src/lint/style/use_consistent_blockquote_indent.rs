use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::blockquote_utils::collect_blockquote_blocks;

declare_lint_rule! {
    /// Enforce consistent blockquote indentation.
    ///
    /// All lines in a blockquote should have the same number of spaces
    /// after the `>` marker.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// > first line
    /// >  second line with extra space
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// > first line
    /// > second line
    /// ```
    pub UseConsistentBlockquoteIndent {
        version: "next",
        name: "useConsistentBlockquoteIndent",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentBlockquoteIndent {
    range: TextRange,
    expected: usize,
    actual: usize,
}

impl Rule for UseConsistentBlockquoteIndent {
    type Query = Ast<MdDocument>;
    type State = InconsistentBlockquoteIndent;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let blocks = collect_blockquote_blocks(&text);
        let mut signals = Vec::new();

        for block in &blocks {
            // Find the expected spacing (from the first marker line)
            let first_marker_line = block.lines.iter().find(|l| l.has_marker);
            let expected = match first_marker_line {
                Some(l) => l.spaces_after_marker,
                None => continue,
            };

            for line in &block.lines {
                if line.has_marker && line.spaces_after_marker != expected {
                    signals.push(InconsistentBlockquoteIndent {
                        range: TextRange::new(
                            base + TextSize::from(line.byte_offset as u32),
                            base + TextSize::from((line.byte_offset + line.byte_len) as u32),
                        ),
                        expected,
                        actual: line.spaces_after_marker,
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
                    "Expected "{state.expected}" space(s) after > but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use consistent spacing after the blockquote marker."
            }),
        )
    }
}
