use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdThematicBreakBlock};
use biome_rowan::AstNode;

use biome_rule_options::use_consistent_horizontal_rule_style::UseConsistentHorizontalRuleStyleOptions;

declare_lint_rule! {
    /// Enforce a consistent style for horizontal rules.
    ///
    /// Horizontal rules (thematic breaks) can be written in many styles
    /// (`---`, `***`, `___`, etc.). Using a consistent style improves
    /// readability and maintainability.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// With the default style `"---"`:
    ///
    /// ```md
    /// ***
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ---
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// The expected horizontal rule style. Default: `"---"`.
    /// Common styles: `"---"`, `"***"`, `"___"`.
    pub UseConsistentHorizontalRuleStyle {
        version: "next",
        name: "useConsistentHorizontalRuleStyle",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct InconsistentHorizontalRule {
    range: biome_rowan::TextRange,
    actual: String,
}

impl Rule for UseConsistentHorizontalRuleStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentHorizontalRule;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentHorizontalRuleStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let expected_style = ctx.options().style();
        let mut signals = Vec::new();

        for node in document.syntax().descendants() {
            if let Some(thematic_break) = MdThematicBreakBlock::cast_ref(&node) {
                if let Ok(token) = thematic_break.value_token() {
                    let actual = token.text_trimmed().trim().to_string();
                    if actual != expected_style {
                        signals.push(InconsistentHorizontalRule {
                            range: thematic_break.syntax().text_trimmed_range(),
                            actual,
                        });
                    }
                }
            }
        }

        signals
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let expected = ctx.options().style();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Horizontal rule style '"{&state.actual}"' does not match expected '"{expected}"'."
                },
            )
            .note(markup! {
                "Use '"{expected}"' for horizontal rules."
            }),
        )
    }
}
