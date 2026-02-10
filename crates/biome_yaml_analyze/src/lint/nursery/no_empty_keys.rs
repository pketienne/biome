use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::BatchMutationExt;
use biome_yaml_syntax::{YamlBlockMapImplicitEntry, YamlLanguage};

declare_lint_rule! {
    /// Disallow empty mapping keys in YAML.
    ///
    /// An empty mapping key (where the key is missing or an empty string) is
    /// usually a mistake. It can result from accidentally writing `: value`
    /// at the start of a line. If a null key is intended, use `null` explicitly.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// : value
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// key: value
    /// ```
    pub NoEmptyKeys {
        version: "next",
        name: "noEmptyKeys",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
        fix_kind: FixKind::Unsafe,
    }
}

impl Rule for NoEmptyKeys {
    type Query = Ast<YamlBlockMapImplicitEntry>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let entry = ctx.query();

        // Check if the key is missing entirely
        if entry.key().is_none() {
            return Some(());
        }

        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let entry = ctx.query();
        let range = entry.colon_token().ok()?.text_trimmed_range();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                range,
                markup! {
                    "Empty mapping keys are not allowed."
                },
            )
            .note(markup! {
                "Provide a key name or use "<Emphasis>"null"</Emphasis>" explicitly if a null key is intended."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let entry = ctx.query();
        let mut mutation = ctx.root().begin();

        mutation.remove_node(entry.clone());

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove the entry with the empty key." }.to_owned(),
            mutation,
        ))
    }
}
