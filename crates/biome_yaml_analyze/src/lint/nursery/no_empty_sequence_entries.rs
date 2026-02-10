use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstSeparatedList, BatchMutationExt};
use biome_yaml_syntax::{YamlFlowSequence, YamlLanguage};

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
        fix_kind: FixKind::Safe,
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

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let sequence = ctx.query();
        let entries = sequence.entries();

        // Collect text of non-empty entries
        let non_empty: Vec<String> = entries
            .elements()
            .filter_map(|element| {
                let node = element.node().ok()?;
                Some(node.syntax().text_trimmed().to_string())
            })
            .collect();

        let new_text = format!("[{}]", non_empty.join(", "));

        // Replace the opening bracket token with the full new text,
        // and remove all other tokens in the sequence
        let open_bracket = sequence.l_brack_token().ok()?;
        let close_bracket = sequence.r_brack_token().ok()?;

        let mut mutation = ctx.root().begin();

        // Replace the open bracket with the full reconstructed sequence text
        let new_open = biome_yaml_syntax::YamlSyntaxToken::new_detached(
            open_bracket.kind(),
            &new_text,
            [],
            [],
        );
        mutation.replace_token_transfer_trivia(open_bracket, new_open);

        // Replace close bracket with empty string (content already included above)
        let new_close = biome_yaml_syntax::YamlSyntaxToken::new_detached(
            close_bracket.kind(),
            "",
            [],
            [],
        );
        mutation.replace_token_transfer_trivia(close_bracket, new_close);

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove the empty entries." }.to_owned(),
            mutation,
        ))
    }
}
