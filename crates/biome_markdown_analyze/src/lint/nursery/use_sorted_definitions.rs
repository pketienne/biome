use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::definition_utils::collect_definitions;

declare_lint_rule! {
    /// Enforce alphabetically sorted link reference definitions.
    ///
    /// Definitions should be sorted by their label for easier navigation
    /// and to avoid duplicates.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [beta]: https://beta.com
    /// [alpha]: https://alpha.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [alpha]: https://alpha.com
    /// [beta]: https://beta.com
    /// ```
    pub UseSortedDefinitions {
        version: "next",
        name: "useSortedDefinitions",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct UnsortedDefinition {
    range: TextRange,
    label: String,
    expected_after: String,
}

impl Rule for UseSortedDefinitions {
    type Query = Ast<MdDocument>;
    type State = UnsortedDefinition;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let definitions = collect_definitions(&text);

        let mut signals = Vec::new();

        for window in definitions.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            if curr.label < prev.label {
                signals.push(UnsortedDefinition {
                    range: TextRange::new(
                        base + TextSize::from(curr.byte_offset as u32),
                        base + TextSize::from((curr.byte_offset + curr.byte_len) as u32),
                    ),
                    label: curr.label.clone(),
                    expected_after: prev.label.clone(),
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
                    "Definition \""{ &state.label }"\" should come before \""{ &state.expected_after }"\"."
                },
            )
            .note(markup! {
                "Sort definitions alphabetically by their label."
            }),
        )
    }
}
