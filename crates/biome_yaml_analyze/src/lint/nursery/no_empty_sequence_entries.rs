use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstSeparatedList};
use biome_yaml_syntax::YamlFlowSequence;

declare_lint_rule! {
    /// Disallow empty entries in flow sequences.
    ///
    /// An empty entry in a flow sequence like `[1, , 3]` is usually a typo.
    /// The missing element becomes an implicit null, which is rarely intentional.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// items: [1, , 3]
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// items: [1, 2, 3]
    /// ```
    pub NoEmptySequenceEntries {
        version: "next",
        name: "noEmptySequenceEntries",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
    }
}

impl Rule for NoEmptySequenceEntries {
    type Query = Ast<YamlFlowSequence>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let sequence = ctx.query();
        let entries = sequence.entries();

        for element in entries.elements() {
            if element.node().is_err() {
                return Some(());
            }
        }

        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let sequence = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                sequence.syntax().text_trimmed_range(),
                markup! {
                    "Empty entries in flow sequences are not allowed."
                },
            )
            .note(markup! {
                "Remove the extra comma or provide a value for the entry."
            }),
        )
    }
}
