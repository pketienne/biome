use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange, TextSize};
use biome_yaml_syntax::YamlRoot;

declare_lint_rule! {
    /// Enforce a maximum line length in YAML files.
    ///
    /// Long lines are harder to read and can cause horizontal scrolling.
    /// This rule flags lines that exceed 120 characters.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// very_long_key: this is a very long value that exceeds the maximum line length limit and should be wrapped or shortened to improve readability of the file
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// key: short value
    /// ```
    pub UseLineLength {
        version: "next",
        name: "useLineLength",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

const MAX_LINE_LENGTH: usize = 120;

impl Rule for UseLineLength {
    type Query = Ast<YamlRoot>;
    type State = TextRange;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let text = root.syntax().to_string();
        let mut violations = Vec::new();
        let mut offset = 0u32;

        for line in text.split('\n') {
            let line_len = line.len();
            if line_len > MAX_LINE_LENGTH {
                let start = TextSize::from(offset);
                let end = TextSize::from(offset + line_len as u32);
                violations.push(TextRange::new(start, end));
            }
            offset += line_len as u32 + 1; // +1 for \n
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state,
                markup! {
                    "Line exceeds the maximum length of 120 characters."
                },
            )
            .note(markup! {
                "Break long lines into multiple lines for better readability."
            }),
        )
    }
}
