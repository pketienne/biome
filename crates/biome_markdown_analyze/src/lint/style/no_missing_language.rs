use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Require a language tag on fenced code blocks.
    ///
    /// Fenced code blocks without a language specifier make it harder for readers
    /// to understand the code and prevent syntax highlighting in rendered output.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ````md
    /// ```
    /// const x = 1;
    /// ```
    /// ````
    ///
    /// ### Valid
    ///
    /// ````md
    /// ```js
    /// const x = 1;
    /// ```
    /// ````
    pub NoMissingLanguage {
        version: "next",
        name: "noMissingLanguage",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct MissingLanguage {
    range: TextRange,
}

impl Rule for NoMissingLanguage {
    type Query = Ast<MdDocument>;
    type State = MissingLanguage;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_trimmed().to_string();
        let start = document.syntax().text_trimmed_range().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        for (line_idx, line) in text.lines().enumerate() {
            if let Some(fence_open) = tracker.process_line(line_idx, line) {
                if fence_open.info_string.is_empty() {
                    let line_offset: usize =
                        text.lines().take(line_idx).map(|l: &str| l.len() + 1).sum();
                    let offset = TextSize::from(line_offset as u32);
                    let len = TextSize::from(line.len() as u32);
                    signals.push(MissingLanguage {
                        range: TextRange::new(start + offset, start + offset + len),
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
                    "Fenced code blocks should have a language tag."
                },
            )
            .note(markup! {
                "Add a language identifier after the opening fence to enable syntax highlighting."
            }),
        )
    }
}
