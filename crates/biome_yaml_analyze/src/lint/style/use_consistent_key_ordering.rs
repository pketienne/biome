use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstNodeList, TextRange};
use biome_yaml_syntax::{AnyYamlBlockMapEntry, YamlBlockMapping};

use crate::lint::correctness::no_duplicate_keys::get_key_text;

declare_lint_rule! {
    /// Enforce alphabetical ordering of keys in YAML mappings.
    ///
    /// Sorted keys make it easier to find entries in large configuration files
    /// and reduce merge conflicts in version control.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// zebra: 1
    /// apple: 2
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// apple: 1
    /// zebra: 2
    /// ```
    pub UseConsistentKeyOrdering {
        version: "next",
        name: "useConsistentKeyOrdering",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct OutOfOrderKey {
    key_text: String,
    key_range: TextRange,
    previous_key: String,
}

impl Rule for UseConsistentKeyOrdering {
    type Query = Ast<YamlBlockMapping>;
    type State = OutOfOrderKey;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let mapping = ctx.query();
        let mut violations = Vec::new();
        let mut prev_key: Option<String> = None;

        for entry in mapping.entries().iter() {
            let (key_text, key_range) = match &entry {
                AnyYamlBlockMapEntry::YamlBlockMapImplicitEntry(implicit) => {
                    if let Some(key) = implicit.key() {
                        if let Some(text) = get_key_text(&key) {
                            (text, key.syntax().text_trimmed_range())
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                AnyYamlBlockMapEntry::YamlBlockMapExplicitEntry(explicit) => {
                    if let Some(key_node) = explicit.key() {
                        let text = key_node.to_string().trim().to_string();
                        (text, key_node.syntax().text_trimmed_range())
                    } else {
                        continue;
                    }
                }
                AnyYamlBlockMapEntry::YamlBogusBlockMapEntry(_) => continue,
            };

            if let Some(ref prev) = prev_key {
                if key_text.to_lowercase() < prev.to_lowercase() {
                    violations.push(OutOfOrderKey {
                        key_text: key_text.clone(),
                        key_range,
                        previous_key: prev.clone(),
                    });
                }
            }

            prev_key = Some(key_text);
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                &state.key_range,
                markup! {
                    "The key "<Emphasis>{&state.key_text}</Emphasis>" should come before "<Emphasis>{&state.previous_key}</Emphasis>" in alphabetical order."
                },
            )
            .note(markup! {
                "Sort mapping keys alphabetically for consistency and easier navigation."
            }),
        )
    }
}
