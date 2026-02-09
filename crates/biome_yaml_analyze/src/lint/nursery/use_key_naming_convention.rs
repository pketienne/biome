use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::AstNode;
use biome_yaml_syntax::{AnyYamlMappingImplicitKey, YamlBlockMapImplicitEntry};

declare_lint_rule! {
    /// Enforce a consistent naming convention for YAML mapping keys.
    ///
    /// By default, this rule enforces camelCase naming for mapping keys.
    /// Consistent key naming improves readability and reduces errors from
    /// inconsistent casing.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// my_key: value
    /// ```
    ///
    /// ```yaml,expect_diagnostic
    /// my-key: value
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// myKey: value
    /// ```
    pub UseKeyNamingConvention {
        version: "next",
        name: "useKeyNamingConvention",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

fn is_camel_case(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }

    let first = s.chars().next().unwrap();
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }

    !s.contains('_') && !s.contains('-')
}

fn get_key_text_for_naming(key: &AnyYamlMappingImplicitKey) -> Option<String> {
    match key {
        AnyYamlMappingImplicitKey::YamlFlowJsonNode(node) => {
            node.content().map(|content| {
                let text = content.to_string();
                text.trim().trim_matches('"').trim_matches('\'').trim().to_string()
            })
        }
        AnyYamlMappingImplicitKey::YamlFlowYamlNode(node) => {
            node.content().map(|scalar| scalar.to_string().trim().to_string())
        }
    }
}

impl Rule for UseKeyNamingConvention {
    type Query = Ast<YamlBlockMapImplicitEntry>;
    type State = String;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let entry = ctx.query();
        let key = entry.key()?;
        let key_text = get_key_text_for_naming(&key)?;

        // Skip keys that are purely numeric
        if key_text.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }

        if !is_camel_case(&key_text) {
            return Some(key_text);
        }
        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let entry = ctx.query();
        let key = entry.key()?;
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                key.syntax().text_trimmed_range(),
                markup! {
                    "The key "<Emphasis>{state}</Emphasis>" does not match the expected camelCase naming convention."
                },
            )
            .note(markup! {
                "Consider renaming the key to use camelCase."
            }),
        )
    }
}
