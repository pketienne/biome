use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Disallow extra spacing after checkbox brackets.
    ///
    /// There should be exactly one space between the closing `]`
    /// of a checkbox and the content text.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// - [x]  extra space
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - [x] correct
    /// ```
    pub NoCheckboxContentIndent {
        version: "next",
        name: "noCheckboxContentIndent",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct BadCheckboxIndent {
    range: TextRange,
    spaces: usize,
}

impl Rule for NoCheckboxContentIndent {
    type Query = Ast<MdDocument>;
    type State = BadCheckboxIndent;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let items = collect_list_items(&text);
        let mut signals = Vec::new();

        for item in &items {
            if let Some(ref cb) = item.checkbox {
                if cb.content_spacing != 1 {
                    signals.push(BadCheckboxIndent {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        spaces: cb.content_spacing,
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
                    "Checkbox has "{state.spaces}" space(s) after brackets instead of 1."
                },
            )
            .note(markup! {
                "Use exactly one space between the checkbox brackets and content."
            }),
        )
    }
}
