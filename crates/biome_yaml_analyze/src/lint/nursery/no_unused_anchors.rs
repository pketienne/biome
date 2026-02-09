use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_yaml_syntax::{YamlAliasNode, YamlAnchorProperty, YamlRoot};
use rustc_hash::{FxHashMap, FxHashSet};

declare_lint_rule! {
    /// Disallow unused anchors in YAML documents.
    ///
    /// Anchors that are defined but never referenced by an alias are likely
    /// unnecessary and may indicate dead code or a typo in the alias name.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// first: &unused value1
    /// second: value2
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// first: &anchor value1
    /// second: *anchor
    /// ```
    pub NoUnusedAnchors {
        version: "next",
        name: "noUnusedAnchors",
        language: "yaml",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct UnusedAnchorState {
    anchor_name: String,
    range: TextRange,
}

impl Rule for NoUnusedAnchors {
    type Query = Ast<YamlRoot>;
    type State = UnusedAnchorState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut anchors = FxHashMap::<String, TextRange>::default();

        for anchor in root.syntax().descendants().filter_map(YamlAnchorProperty::cast) {
            let Ok(token) = anchor.value_token() else {
                continue;
            };
            let text = token.text_trimmed();
            let name = text.strip_prefix('&').unwrap_or(text).to_string();
            anchors.entry(name).or_insert(token.text_trimmed_range());
        }

        let referenced_anchors: FxHashSet<String> = root
            .syntax()
            .descendants()
            .filter_map(YamlAliasNode::cast)
            .filter_map(|alias| {
                let text = alias.value_token().ok()?.text_trimmed().to_string();
                Some(text.strip_prefix('*').unwrap_or(&text).to_string())
            })
            .collect();

        anchors
            .into_iter()
            .filter(|(name, _)| !referenced_anchors.contains(name))
            .map(|(anchor_name, range)| UnusedAnchorState { anchor_name, range })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "The anchor "<Emphasis>{&state.anchor_name}</Emphasis>" is defined but never referenced."
                },
            )
            .note(markup! {
                "Remove the unused anchor or add an alias that references it."
            }),
        )
    }
}
