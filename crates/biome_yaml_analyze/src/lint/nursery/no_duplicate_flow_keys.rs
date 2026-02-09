use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstSeparatedList, TextRange};
use biome_yaml_syntax::{AnyYamlFlowMapEntry, YamlFlowMapping};
use rustc_hash::FxHashMap;

use super::no_duplicate_keys::get_key_text;

declare_lint_rule! {
    /// Disallow duplicate keys in YAML flow mappings.
    ///
    /// When a YAML flow mapping contains duplicate keys, only the last value is used.
    /// This is almost always a mistake and can lead to unexpected behavior.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// {name: John, age: 30, name: Jane}
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// {name: John, age: 30, email: john@example.com}
    /// ```
    pub NoDuplicateFlowKeys {
        version: "next",
        name: "noDuplicateFlowKeys",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct DuplicateFlowKeyState {
    first_range: TextRange,
    key_text: String,
    duplicate_ranges: Vec<TextRange>,
}

impl Rule for NoDuplicateFlowKeys {
    type Query = Ast<YamlFlowMapping>;
    type State = DuplicateFlowKeyState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let mapping = ctx.query();
        let mut keys_seen = FxHashMap::<String, (TextRange, Vec<TextRange>)>::default();

        for entry in mapping.entries().iter().flatten() {
            let (key_text, key_range) = match &entry {
                AnyYamlFlowMapEntry::YamlFlowMapImplicitEntry(implicit) => {
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
                AnyYamlFlowMapEntry::YamlFlowMapExplicitEntry(explicit) => {
                    if let Some(key) = explicit.key() {
                        if let Some(text) = get_key_text(&key) {
                            (text, key.syntax().text_trimmed_range())
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
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
            .map(
                |(key_text, (first_range, duplicate_ranges))| DuplicateFlowKeyState {
                    first_range,
                    key_text,
                    duplicate_ranges,
                },
            )
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
