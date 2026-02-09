use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::definition_utils::collect_definitions;

declare_lint_rule! {
    /// Enforce lowercase labels in link reference definitions.
    ///
    /// Definition labels should be lowercase for consistency.
    /// While label matching is case-insensitive in markdown,
    /// using lowercase labels avoids confusion.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [FOO]: https://example.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// ```
    pub UseLowercaseDefinitionLabels {
        version: "next",
        name: "useLowercaseDefinitionLabels",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct UppercaseLabel {
    range: TextRange,
    label: String,
}

impl Rule for UseLowercaseDefinitionLabels {
    type Query = Ast<MdDocument>;
    type State = UppercaseLabel;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let definitions = collect_definitions(&text);

        let mut signals = Vec::new();

        for def in &definitions {
            if def.raw_label != def.raw_label.to_lowercase() {
                signals.push(UppercaseLabel {
                    range: TextRange::new(
                        base + TextSize::from(def.byte_offset as u32),
                        base + TextSize::from((def.byte_offset + def.byte_len) as u32),
                    ),
                    label: def.raw_label.clone(),
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
                    "Definition label \""{ &state.label }"\" should be lowercase."
                },
            )
            .note(markup! {
                "Use lowercase labels for consistency."
            }),
        )
    }
}
