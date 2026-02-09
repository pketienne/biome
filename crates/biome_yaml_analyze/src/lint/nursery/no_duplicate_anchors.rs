use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_yaml_syntax::{YamlAnchorProperty, YamlRoot};
use rustc_hash::FxHashMap;

declare_lint_rule! {
    /// Disallow duplicate anchor names in a YAML document.
    ///
    /// Anchor names must be unique within a YAML document. When the same anchor
    /// name is defined multiple times, aliases referencing that name may resolve
    /// to unexpected values.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// first: &anchor value1
    /// second: &anchor value2
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// first: &anchor1 value1
    /// second: &anchor2 value2
    /// ```
    pub NoDuplicateAnchors {
        version: "next",
        name: "noDuplicateAnchors",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct DuplicateAnchorState {
    anchor_name: String,
    first_range: TextRange,
    duplicate_ranges: Vec<TextRange>,
}

impl Rule for NoDuplicateAnchors {
    type Query = Ast<YamlRoot>;
    type State = DuplicateAnchorState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut anchors = FxHashMap::<String, (TextRange, Vec<TextRange>)>::default();

        for anchor in root.syntax().descendants().filter_map(YamlAnchorProperty::cast) {
            let Ok(token) = anchor.value_token() else {
                continue;
            };
            let text = token.text_trimmed();
            let name = text.strip_prefix('&').unwrap_or(text).to_string();

            if let Some((_, duplicates)) = anchors.get_mut(&name) {
                duplicates.push(token.text_trimmed_range());
            } else {
                anchors.insert(name, (token.text_trimmed_range(), Vec::new()));
            }
        }

        anchors
            .into_iter()
            .filter(|(_, (_, duplicates))| !duplicates.is_empty())
            .map(
                |(anchor_name, (first_range, duplicate_ranges))| DuplicateAnchorState {
                    anchor_name,
                    first_range,
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
                "The anchor "<Emphasis>{&state.anchor_name}</Emphasis>" is defined multiple times."
            },
        );
        for range in &state.duplicate_ranges {
            diagnostic = diagnostic.detail(
                range,
                markup! {
                    "This is where a duplicated anchor was declared again."
                },
            );
        }
        Some(diagnostic.note(markup! {
            "Anchor names must be unique within a document. Duplicate anchors may cause aliases to resolve to unexpected values."
        }))
    }
}
