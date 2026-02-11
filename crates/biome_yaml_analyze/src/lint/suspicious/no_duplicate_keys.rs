use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstNodeList};
use biome_yaml_syntax::{AnyYamlBlockMapEntry, AnyYamlMappingImplicitKey, TextRange, YamlBlockMapping};
use rustc_hash::FxHashMap;

declare_lint_rule! {
    /// Disallow two keys with the same name inside YAML block mappings.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// name: John
    /// age: 30
    /// name: Jane
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// name: John
    /// age: 30
    /// city: NYC
    /// ```
    pub NoDuplicateKeys {
        version: "next",
        name: "noDuplicateKeys",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
    }
}

/// Extract the text of a key from a YAML mapping implicit key node.
fn extract_key_text(key: &AnyYamlMappingImplicitKey) -> Option<String> {
    match key {
        AnyYamlMappingImplicitKey::YamlFlowYamlNode(node) => {
            let scalar = node.content()?;
            let token = scalar.value_token().ok()?;
            Some(token.text_trimmed().to_string())
        }
        AnyYamlMappingImplicitKey::YamlFlowJsonNode(_) => {
            // Flow JSON nodes (quoted strings, etc.) are less common as mapping keys
            // but we can get the text from the syntax node
            None
        }
    }
}

impl Rule for NoDuplicateKeys {
    type Query = Ast<YamlBlockMapping>;
    type State = (TextRange, String, Vec<TextRange>);
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let query = ctx.query();
        let mut first_occurrence = FxHashMap::<String, TextRange>::default();
        let mut duplicates = FxHashMap::<String, Vec<TextRange>>::default();

        for entry in query.entries().iter() {
            let key_info = match &entry {
                AnyYamlBlockMapEntry::YamlBlockMapImplicitEntry(implicit) => {
                    implicit.key().as_ref().and_then(|k| {
                        extract_key_text(k).map(|text| (text, k.range()))
                    })
                }
                _ => None,
            };

            if let Some((key_text, key_range)) = key_info {
                if let Some(original_range) = first_occurrence.get(&key_text) {
                    duplicates
                        .entry(key_text)
                        .or_insert_with(|| vec![*original_range])
                        .push(key_range);
                } else {
                    first_occurrence.insert(key_text, key_range);
                }
            }
        }

        duplicates
            .into_iter()
            .map(|(key_text, ranges)| (ranges[0], key_text, ranges[1..].to_vec()))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let (original_range, key_text, duplicate_ranges) = state;
        let mut diagnostic = RuleDiagnostic::new(
            rule_category!(),
            original_range,
            markup! {
                "The key "<Emphasis>{key_text}</Emphasis>" was already declared."
            },
        );
        for range in duplicate_ranges {
            diagnostic = diagnostic.detail(
                range,
                markup! {
                    "This is where a duplicated key was declared again."
                },
            );
        }
        Some(diagnostic.note(
            markup! {
                "If a key is defined multiple times, only the last definition takes effect. Previous definitions are ignored."
            },
        ))
    }
}
