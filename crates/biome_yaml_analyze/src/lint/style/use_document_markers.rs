use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt};
use biome_yaml_syntax::{YamlDocument, YamlLanguage, YamlSyntaxKind, YamlSyntaxToken};

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
        fix_kind: FixKind::Safe,
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

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let document = ctx.query();
        let mut mutation = ctx.root().begin();

        // Insert a `---` token using the with_dashdashdash_token builder
        let marker = YamlSyntaxToken::new_detached(
            YamlSyntaxKind::DIRECTIVE_END,
            "---",
            [],
            [biome_rowan::TriviaPiece::new(
                biome_rowan::TriviaPieceKind::Newline,
                1,
            )],
        );
        let new_document = document.clone().with_dashdashdash_token(Some(marker));
        mutation.replace_node(document.clone(), new_document);

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Add document start marker." }.to_owned(),
            mutation,
        ))
    }
}
