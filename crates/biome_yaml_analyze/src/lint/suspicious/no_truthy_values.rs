use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt};
use biome_yaml_syntax::{YamlLanguage, YamlPlainScalar, YamlSyntaxToken};

declare_lint_rule! {
    /// Disallow non-standard boolean values in YAML.
    ///
    /// YAML 1.1 interprets values like `yes`, `no`, `on`, `off`, `y`, `n` as booleans.
    /// YAML 1.2 only recognizes `true` and `false`. Using the legacy values is a common
    /// source of bugs, especially in contexts like country codes (`NO` for Norway) or
    /// configuration flags.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// enabled: yes
    /// ```
    ///
    /// ```yaml,expect_diagnostic
    /// country: NO
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// enabled: true
    /// country: "NO"
    /// ```
    pub NoTruthyValues {
        version: "next",
        name: "noTruthyValues",
        language: "yaml",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

/// Known truthy/falsy values from YAML 1.1 that should be avoided
const TRUTHY_VALUES: &[&str] = &[
    "yes", "Yes", "YES", "no", "No", "NO", "on", "On", "ON", "off", "Off", "OFF", "y", "Y", "n",
    "N",
];

impl Rule for NoTruthyValues {
    type Query = Ast<YamlPlainScalar>;
    type State = String;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let scalar = ctx.query();
        let text = scalar.value_token().ok()?;
        let value = text.text_trimmed();

        if TRUTHY_VALUES.contains(&value) {
            return Some(value.to_string());
        }
        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();
        let is_truthy = matches!(
            state.as_str(),
            "yes" | "Yes" | "YES" | "on" | "On" | "ON" | "y" | "Y"
        );
        let suggestion = if is_truthy { "true" } else { "false" };

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.syntax().text_trimmed_range(),
                markup! {
                    "The value "<Emphasis>{state}</Emphasis>" is interpreted as a boolean in YAML 1.1."
                },
            )
            .note(markup! {
                "Use "<Emphasis>{suggestion}</Emphasis>" instead, or quote the value if it's meant as a string."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let scalar = ctx.query();
        let mut mutation = ctx.root().begin();
        let token = scalar.value_token().ok()?;

        let is_truthy = matches!(
            state.as_str(),
            "yes" | "Yes" | "YES" | "on" | "On" | "ON" | "y" | "Y"
        );
        let replacement = if is_truthy { "true" } else { "false" };

        let new_token = YamlSyntaxToken::new_detached(token.kind(), replacement, [], []);
        mutation.replace_token_transfer_trivia(token, new_token);

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Replace with "<Emphasis>{replacement}</Emphasis>"." }.to_owned(),
            mutation,
        ))
    }
}
