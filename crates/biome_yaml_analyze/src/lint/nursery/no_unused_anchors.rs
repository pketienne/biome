use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, Direction, TextRange, WalkEvent};
use biome_yaml_syntax::{YamlRoot, YamlSyntaxKind};
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
        let mut referenced_anchors = FxHashSet::<String>::default();

        for event in root.syntax().preorder_with_tokens(Direction::Next) {
            if let WalkEvent::Enter(element) = event {
                if let Some(token) = element.as_token() {
                    match token.kind() {
                        YamlSyntaxKind::ANCHOR_PROPERTY_LITERAL => {
                            let text = token.text_trimmed();
                            let name = text.strip_prefix('&').unwrap_or(text).to_string();
                            anchors.entry(name).or_insert(token.text_trimmed_range());
                        }
                        YamlSyntaxKind::ALIAS_LITERAL => {
                            let text = token.text_trimmed();
                            let name = text.strip_prefix('*').unwrap_or(text).to_string();
                            referenced_anchors.insert(name);
                        }
                        _ => {}
                    }
                }
            }
        }

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
