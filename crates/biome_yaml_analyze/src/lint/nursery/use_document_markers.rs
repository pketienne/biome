use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::AstNode;
use biome_yaml_syntax::YamlDocument;

declare_lint_rule! {
    /// Require the use of document start markers (`---`) in YAML.
    ///
    /// The document start marker `---` explicitly marks the beginning of a
    /// YAML document. Using it consistently makes multi-document streams
    /// unambiguous and improves readability.
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
    /// ---
    /// key: value
    /// ```
    pub UseDocumentMarkers {
        version: "next",
        name: "useDocumentMarkers",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

impl Rule for UseDocumentMarkers {
    type Query = Ast<YamlDocument>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();

        // Only flag documents that have actual data content but no --- marker
        if document.node().is_some() && document.dashdashdash_token().is_none() {
            // Check if the document node has any meaningful text content
            // (not just whitespace from comments being parsed as trivia)
            let node_text = document.node()?.syntax().text_trimmed_range();
            if !node_text.is_empty() {
                return Some(());
            }
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
                    "Missing document start marker "<Emphasis>"---"</Emphasis>"."
                },
            )
            .note(markup! {
                "Add a "<Emphasis>"---"</Emphasis>" marker at the beginning of the document."
            }),
        )
    }
}
