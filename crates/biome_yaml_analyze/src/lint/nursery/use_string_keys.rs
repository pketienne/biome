use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::AstNode;
use biome_yaml_syntax::YamlBlockMapImplicitEntry;

use crate::lint::nursery::no_duplicate_keys::get_key_text;

declare_lint_rule! {
    /// Disallow non-string mapping keys in YAML.
    ///
    /// YAML allows any value type as a mapping key, including numbers, booleans,
    /// and null. Non-string keys can lead to subtle bugs, especially when consumed
    /// by languages that only support string keys (like JSON or JavaScript).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// 123: numeric key
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// name: string key
    /// ```
    pub UseStringKeys {
        version: "next",
        name: "useStringKeys",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

/// Check if a string represents a non-string YAML value (number, boolean, null)
fn is_non_string_value(s: &str) -> bool {
    // Check for null
    if matches!(s, "null" | "Null" | "NULL" | "~") {
        return true;
    }

    // Check for booleans
    if matches!(
        s,
        "true" | "True" | "TRUE" | "false" | "False" | "FALSE" | "yes" | "Yes" | "YES" | "no"
            | "No" | "NO" | "on" | "On" | "ON" | "off" | "Off" | "OFF" | "y" | "Y" | "n" | "N"
    ) {
        return true;
    }

    // Check for integers (including hex, octal, binary)
    let trimmed = s.strip_prefix(['+', '-']).unwrap_or(s);
    if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        return true;
    }
    if trimmed.starts_with("0o") || trimmed.starts_with("0O") {
        return true;
    }
    if trimmed.starts_with("0b") || trimmed.starts_with("0B") {
        return true;
    }

    // Check for floats
    if trimmed.contains('.') || trimmed.contains('e') || trimmed.contains('E') {
        let without_exp = trimmed.split(['e', 'E']).next().unwrap_or("");
        let parts: Vec<&str> = without_exp.splitn(2, '.').collect();
        if parts.iter().all(|p| p.is_empty() || p.chars().all(|c| c.is_ascii_digit())) {
            if parts.iter().any(|p| !p.is_empty()) {
                return true;
            }
        }
    }

    // Check for special floats
    if matches!(trimmed, ".inf" | ".Inf" | ".INF" | ".nan" | ".NaN" | ".NAN") {
        return true;
    }

    false
}

impl Rule for UseStringKeys {
    type Query = Ast<YamlBlockMapImplicitEntry>;
    type State = String;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let entry = ctx.query();
        let key = entry.key()?;
        let key_text = get_key_text(&key)?;

        if is_non_string_value(&key_text) {
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
                    "The key "<Emphasis>{state}</Emphasis>" is not a string. Use string keys for compatibility."
                },
            )
            .note(markup! {
                "Non-string keys may cause issues when consumed by JSON or JavaScript. Quote the key or use a string value."
            }),
        )
    }
}
