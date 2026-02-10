use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt};
use biome_yaml_syntax::{YamlLanguage, YamlPlainScalar, YamlSyntaxToken};

declare_lint_rule! {
    /// Disallow implicit octal number values in YAML.
    ///
    /// In YAML 1.1, numbers with a leading zero (like `0777`) are interpreted as
    /// octal values. This is a common source of bugs, as users often intend
    /// decimal values. YAML 1.2 removed implicit octal support, but some parsers
    /// still interpret them as octal for backward compatibility.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// port: 0777
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// port: 777
    /// ```
    pub NoImplicitOctalValues {
        version: "next",
        name: "noImplicitOctalValues",
        language: "yaml",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

impl Rule for NoImplicitOctalValues {
    type Query = Ast<YamlPlainScalar>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let scalar = ctx.query();
        let token = scalar.value_token().ok()?;
        let text = token.text_trimmed().trim();

        // Check for implicit octal: starts with 0, followed by digits, all 0-7
        if text.len() >= 2
            && text.starts_with('0')
            && !text.starts_with("0x")
            && !text.starts_with("0X")
            && !text.starts_with("0o")
            && !text.starts_with("0O")
            && !text.starts_with("0b")
            && !text.starts_with("0B")
            && !text.contains('.')
            && !text.contains('e')
            && !text.contains('E')
            && text[1..].chars().all(|c| c.is_ascii_digit())
        {
            // It's a number starting with 0 followed by more digits
            // This is an implicit octal in YAML 1.1
            return Some(());
        }

        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let scalar = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                scalar.syntax().text_trimmed_range(),
                markup! {
                    "Ambiguous number with leading zero may be interpreted as octal."
                },
            )
            .note(markup! {
                "Remove the leading zero or use explicit octal notation (0o prefix) to clarify intent."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let scalar = ctx.query();
        let mut mutation = ctx.root().begin();
        let token = scalar.value_token().ok()?;
        let text = token.text_trimmed().trim();

        // Convert implicit octal (e.g., 0777) to explicit octal (0o777)
        let new_value = format!("0o{}", &text[1..]);

        let new_token = YamlSyntaxToken::new_detached(token.kind(), &new_value, [], []);
        mutation.replace_token_transfer_trivia(token, new_token);

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Convert to explicit octal notation." }.to_owned(),
            mutation,
        ))
    }
}
