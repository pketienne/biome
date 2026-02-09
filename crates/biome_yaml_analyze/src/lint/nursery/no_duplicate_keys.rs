use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstNodeList, TextRange};
use biome_yaml_syntax::{AnyYamlBlockMapEntry, AnyYamlMappingImplicitKey, YamlBlockMapping};
use rustc_hash::FxHashMap;

declare_lint_rule! {
    /// Disallow duplicate keys in YAML block mappings.
    ///
    /// When a YAML mapping contains duplicate keys, only the last value is used.
    /// This is almost always a mistake and can lead to unexpected behavior.
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
    /// email: john@example.com
    /// ```
    pub NoDuplicateKeys {
        version: "next",
        name: "noDuplicateKeys",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
    }
}

/// Extract the text content of a mapping key
pub(crate) fn get_key_text(key: &AnyYamlMappingImplicitKey) -> Option<String> {
    match key {
        AnyYamlMappingImplicitKey::YamlFlowJsonNode(node) => {
            node.content().map(|content| {
                let s = content.to_string();
                if s.as_bytes().first().is_some_and(|c| c.is_ascii_whitespace())
                    || s.as_bytes().last().is_some_and(|c| c.is_ascii_whitespace())
                {
                    s.trim().to_string()
                } else {
                    s
                }
            })
        }
        AnyYamlMappingImplicitKey::YamlFlowYamlNode(node) => {
            node.content().map(|scalar| {
                let s = scalar.to_string();
                if s.as_bytes().first().is_some_and(|c| c.is_ascii_whitespace())
                    || s.as_bytes().last().is_some_and(|c| c.is_ascii_whitespace())
                {
                    s.trim().to_string()
                } else {
                    s
                }
            })
        }
    }
}

pub struct DuplicateKeyState {
    first_range: TextRange,
    key_text: String,
    duplicate_ranges: Vec<TextRange>,
}

impl Rule for NoDuplicateKeys {
    type Query = Ast<YamlBlockMapping>;
    type State = DuplicateKeyState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let mapping = ctx.query();
        let mut keys_seen = FxHashMap::<String, (TextRange, Vec<TextRange>)>::default();

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

            // Merge keys (<<) can appear multiple times
            if key_text == "<<" {
                continue;
            }

            if let Some((_, duplicates)) = keys_seen.get_mut(&key_text) {
                duplicates.push(key_range);
            } else {
                keys_seen.insert(key_text, (key_range, Vec::new()));
            }
        }

        keys_seen
            .into_iter()
            .filter(|(_, (_, duplicates))| !duplicates.is_empty())
            .map(|(key_text, (first_range, duplicate_ranges))| DuplicateKeyState {
                first_range,
                key_text,
                duplicate_ranges,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let mut diagnostic = RuleDiagnostic::new(
            rule_category!(),
            state.first_range,
            markup! {
                "The key "<Emphasis>{&state.key_text}</Emphasis>" was already declared."
            },
        );
        for range in &state.duplicate_ranges {
            diagnostic = diagnostic.detail(
                range,
                markup! {
                    "This is where a duplicated key was declared again."
                },
            );
        }
        Some(diagnostic.note(markup! {
            "If a key is defined multiple times, only the last definition takes effect. Previous definitions are ignored."
        }))
    }
}
