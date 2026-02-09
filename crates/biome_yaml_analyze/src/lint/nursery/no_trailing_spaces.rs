use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange, TextSize};
use biome_yaml_syntax::YamlRoot;

declare_lint_rule! {
    /// Disallow trailing whitespace at the end of lines.
    ///
    /// Trailing whitespace is invisible and serves no purpose. It can cause
    /// noise in version control diffs and is considered bad practice.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// key: value
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// key: value
    /// ```
    pub NoTrailingSpaces {
        version: "next",
        name: "noTrailingSpaces",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
    }
}

impl Rule for NoTrailingSpaces {
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
            let trimmed_len = line.trim_end_matches(|c| c == ' ' || c == '\t').len();
            let line_len = line.len();
            if trimmed_len < line_len {
                let start = TextSize::from(offset + trimmed_len as u32);
                let end = TextSize::from(offset + line_len as u32);
                violations.push(TextRange::new(start, end));
            }
            offset += line_len as u32 + 1; // +1 for the \n
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state,
                markup! {
                    "Trailing whitespace is not allowed."
                },
            )
            .note(markup! {
                "Remove the trailing whitespace at the end of the line."
            }),
        )
    }
}
