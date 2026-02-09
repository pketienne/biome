use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::blockquote_utils::collect_blockquote_blocks;

declare_lint_rule! {
    /// Disallow broken blockquote continuation.
    ///
    /// Every line in a blockquote should start with `>`. Lines without
    /// the marker break the blockquote continuation.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// > first line
    /// second line without marker
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// > first line
    /// > second line
    /// ```
    pub NoBlockquoteBrokenContinuation {
        version: "next",
        name: "noBlockquoteBrokenContinuation",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct BrokenContinuation {
    range: TextRange,
}

impl Rule for NoBlockquoteBrokenContinuation {
    type Query = Ast<MdDocument>;
    type State = BrokenContinuation;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let blocks = collect_blockquote_blocks(&text);
        let mut signals = Vec::new();

        for block in &blocks {
            for line in &block.lines {
                if !line.has_marker {
                    signals.push(BrokenContinuation {
                        range: TextRange::new(
                            base + TextSize::from(line.byte_offset as u32),
                            base + TextSize::from((line.byte_offset + line.byte_len) as u32),
                        ),
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
                    "Blockquote continuation line is missing the > marker."
                },
            )
            .note(markup! {
                "Add > at the beginning of continuation lines in blockquotes."
            }),
        )
    }
}
