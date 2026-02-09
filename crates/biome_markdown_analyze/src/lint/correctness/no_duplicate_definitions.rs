use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::definition_utils::collect_definitions;

declare_lint_rule! {
    /// Disallow duplicate link reference definitions.
    ///
    /// Each label should only be defined once. Duplicate definitions are
    /// confusing and the second definition is ignored by parsers.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// [foo]: https://other.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// [bar]: https://other.com
    /// ```
    pub NoDuplicateDefinitions {
        version: "next",
        name: "noDuplicateDefinitions",
        language: "md",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct DuplicateDefinition {
    range: TextRange,
    label: String,
}

impl Rule for NoDuplicateDefinitions {
    type Query = Ast<MdDocument>;
    type State = DuplicateDefinition;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let definitions = collect_definitions(&text);

        let mut seen = std::collections::HashSet::new();
        let mut signals = Vec::new();

        for def in &definitions {
            if !seen.insert(def.label.clone()) {
                signals.push(DuplicateDefinition {
                    range: TextRange::new(
                        base + TextSize::from(def.byte_offset as u32),
                        base + TextSize::from((def.byte_offset + def.byte_len) as u32),
                    ),
                    label: def.label.clone(),
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
                    "Duplicate definition for label \""{ &state.label }"\"."
                },
            )
            .note(markup! {
                "Remove the duplicate definition. Only the first definition is used."
            }),
        )
    }
}
