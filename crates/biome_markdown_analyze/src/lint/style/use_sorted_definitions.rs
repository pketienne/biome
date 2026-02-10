use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdLinkBlock};
use biome_rowan::{AstNode, BatchMutationExt, TextRange};

use crate::MarkdownRuleAction;
use crate::utils::definition_utils::normalize_label;

declare_lint_rule! {
    /// Enforce alphabetically sorted link reference definitions.
    ///
    /// Definitions should be sorted by their label for easier navigation
    /// and to avoid duplicates.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [beta]: https://beta.com
    /// [alpha]: https://alpha.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [alpha]: https://alpha.com
    /// [beta]: https://beta.com
    /// ```
    pub UseSortedDefinitions {
        version: "next",
        name: "useSortedDefinitions",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct UnsortedDefinition {
    range: TextRange,
    label: String,
    expected_after: String,
}

impl Rule for UseSortedDefinitions {
    type Query = Ast<MdDocument>;
    type State = UnsortedDefinition;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let definitions: Vec<_> = document
            .syntax()
            .descendants()
            .filter_map(MdLinkBlock::cast)
            .collect();

        let mut signals = Vec::new();

        for window in definitions.windows(2) {
            let prev_label =
                normalize_label(&window[0].label().syntax().text_trimmed().to_string());
            let curr_label =
                normalize_label(&window[1].label().syntax().text_trimmed().to_string());

            if curr_label < prev_label {
                signals.push(UnsortedDefinition {
                    range: window[1].syntax().text_trimmed_range(),
                    label: curr_label,
                    expected_after: prev_label,
                });
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let node = root
            .syntax()
            .descendants()
            .filter_map(MdLinkBlock::cast)
            .find(|n| n.syntax().text_trimmed_range() == state.range)?;
        let mut mutation = root.begin();
        mutation.remove_node(node);
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove definition from current position." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Definition \""{ &state.label }"\" should come before \""{ &state.expected_after }"\"."
                },
            )
            .note(markup! {
                "Sort definitions alphabetically by their label."
            }),
        )
    }
}
