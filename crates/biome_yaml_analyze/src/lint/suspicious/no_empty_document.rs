use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstNodeList};
use biome_yaml_syntax::YamlDocument;

declare_lint_rule! {
    /// Disallow empty YAML documents.
    ///
    /// An empty document with no content is usually a mistake. If you have
    /// an intentionally empty document in a multi-document stream, add a
    /// comment explaining why.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// ---
    /// ...
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// ---
    /// key: value
    /// ...
    /// ```
    pub NoEmptyDocument {
        version: "next",
        name: "noEmptyDocument",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

impl Rule for NoEmptyDocument {
    type Query = Ast<YamlDocument>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();

        if document.node().is_none() && document.directives().is_empty() {
            return Some(());
        }
        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let document = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                document.syntax().text_trimmed_range(),
                markup! {
                    "Empty YAML documents are not allowed."
                },
            )
            .note(markup! {
                "Add content to the document or remove the empty document markers."
            }),
        )
    }
}
