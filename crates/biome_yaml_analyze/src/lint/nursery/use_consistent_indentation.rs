use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode as _, TextRange, TextSize};
use biome_yaml_syntax::YamlRoot;

declare_lint_rule! {
    /// Enforce consistent indentation width in YAML files.
    ///
    /// YAML files should use a consistent number of spaces for indentation.
    /// Mixing different indentation widths (e.g., 2 spaces and 4 spaces) makes
    /// files harder to read and can cause unexpected parsing behavior.
    ///
    /// This rule detects the base indentation unit from the first indented line
    /// and flags lines that use indentation levels not aligned to that unit.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// parent:
    ///   child:
    ///       grandchild: value
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// parent:
    ///   child:
    ///     grandchild: value
    /// ```
    pub UseConsistentIndentation {
        version: "next",
        name: "useConsistentIndentation",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentIndent {
    line_number: usize,
    indent: usize,
    base_unit: usize,
    range: TextRange,
}

impl Rule for UseConsistentIndentation {
    type Query = Ast<YamlRoot>;
    type State = InconsistentIndent;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let text = root.syntax().to_string();
        let mut results = Vec::new();

        // Find the base indentation unit from the first indented line
        let mut base_unit: Option<usize> = None;
        let mut offset = 0usize;

        for (line_number, line) in text.split('\n').enumerate() {
            let trimmed = line.trim_start_matches(' ');
            let indent: usize = line.len() - trimmed.len();

            if indent > 0 && !trimmed.is_empty() && !trimmed.starts_with('#') {
                if base_unit.is_none() {
                    base_unit = Some(indent);
                } else if let Some(unit) = base_unit {
                    if indent % unit != 0 {
                        let start = TextSize::from(offset as u32);
                        let end = TextSize::from((offset + indent) as u32);
                        results.push(InconsistentIndent {
                            line_number,
                            indent,
                            base_unit: unit,
                            range: TextRange::new(start, end),
                        });
                    }
                }
            }

            offset += line.len() + 1; // +1 for newline
        }

        results
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Inconsistent indentation on line "{(state.line_number + 1).to_string()}": found "
                    {state.indent.to_string()}" spaces, expected a multiple of "{state.base_unit.to_string()}"."
                },
            )
            .note(markup! {
                "Use a consistent indentation width throughout the file."
            }),
        )
    }
}
