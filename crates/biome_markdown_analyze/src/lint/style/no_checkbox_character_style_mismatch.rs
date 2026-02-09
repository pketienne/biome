use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::no_checkbox_character_style_mismatch::NoCheckboxCharacterStyleMismatchOptions;

use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Enforce consistent checkbox character style.
    ///
    /// Checked checkboxes can use `x` or `X`. This rule enforces
    /// a consistent character.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"lowercase"` (default):
    ///
    /// ```md
    /// - [X] done
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - [x] done
    /// ```
    ///
    /// ## Options
    ///
    /// ### `checked`
    ///
    /// Which character to use for checked checkboxes. Default: `"lowercase"`.
    pub NoCheckboxCharacterStyleMismatch {
        version: "next",
        name: "noCheckboxCharacterStyleMismatch",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct MismatchedCheckbox {
    range: TextRange,
    expected: char,
    actual: char,
}

impl Rule for NoCheckboxCharacterStyleMismatch {
    type Query = Ast<MdDocument>;
    type State = MismatchedCheckbox;
    type Signals = Vec<Self::State>;
    type Options = NoCheckboxCharacterStyleMismatchOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let checked_style = ctx.options().checked();
        let items = collect_list_items(&text);
        let mut signals = Vec::new();

        let expected_char = match checked_style {
            "uppercase" => 'X',
            "consistent" => {
                // Use the first checked checkbox's character
                items
                    .iter()
                    .filter_map(|i| i.checkbox.as_ref())
                    .find(|c| c.check_char == 'x' || c.check_char == 'X')
                    .map(|c| c.check_char)
                    .unwrap_or('x')
            }
            _ => 'x', // "lowercase" (default)
        };

        for item in &items {
            if let Some(ref cb) = item.checkbox {
                if (cb.check_char == 'x' || cb.check_char == 'X')
                    && cb.check_char != expected_char
                {
                    signals.push(MismatchedCheckbox {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        expected: expected_char,
                        actual: cb.check_char,
                    });
                }
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let expected = state.expected.to_string();
        let actual = state.actual.to_string();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected checkbox character \""{ expected }"\" but found \""{ actual }"\"."
                },
            )
            .note(markup! {
                "Use a consistent checkbox character style."
            }),
        )
    }
}
