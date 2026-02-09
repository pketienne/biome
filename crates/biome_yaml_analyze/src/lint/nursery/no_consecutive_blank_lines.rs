use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange, TextSize};
use biome_yaml_syntax::YamlRoot;

declare_lint_rule! {
    /// Disallow more than one consecutive blank line in YAML files.
    ///
    /// Multiple consecutive blank lines add unnecessary vertical space and reduce
    /// readability. This rule enforces a maximum of one blank line between content.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// key: value
    ///
    ///
    /// other: data
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// key: value
    ///
    /// other: data
    /// ```
    pub NoConsecutiveBlankLines {
        version: "next",
        name: "noConsecutiveBlankLines",
        language: "yaml",
        recommended: true,
        severity: Severity::Warning,
    }
}

impl Rule for NoConsecutiveBlankLines {
    type Query = Ast<YamlRoot>;
    type State = TextRange;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let text = root.syntax().to_string();
        let mut violations = Vec::new();
        let mut consecutive_newlines = 0u32;
        let mut blank_start = 0u32;
        let mut offset = 0u32;

        for line in text.split('\n') {
            let is_blank = line.trim().is_empty();
            if is_blank {
                consecutive_newlines += 1;
                if consecutive_newlines == 2 {
                    // Mark the start of the extra blank lines
                    blank_start = offset;
                }
            } else {
                if consecutive_newlines > 1 {
                    // We had multiple consecutive blank lines
                    // Report from the second blank line to the current position
                    let start = TextSize::from(blank_start);
                    let end = TextSize::from(offset);
                    violations.push(TextRange::new(start, end));
                }
                consecutive_newlines = 0;
            }
            offset += line.len() as u32 + 1; // +1 for \n
        }

        // Handle trailing consecutive blank lines at end of file
        if consecutive_newlines > 1 {
            let start = TextSize::from(blank_start);
            let end = TextSize::from((text.len() as u32).saturating_sub(1));
            if start < end {
                violations.push(TextRange::new(start, end));
            }
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state,
                markup! {
                    "Multiple consecutive blank lines are not allowed."
                },
            )
            .note(markup! {
                "Remove the extra blank lines. At most one blank line is allowed between content."
            }),
        )
    }
}
