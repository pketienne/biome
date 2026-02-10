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
    /// Disallow duplicate link reference definitions.
    ///
    /// Each label should only be defined once. Duplicate definitions are
    /// confusing and the second definition is ignored by parsers.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// [foo]: https://other.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// [bar]: https://other.com
    /// ```
    pub NoDuplicateDefinitions {
        version: "next",
        name: "noDuplicateDefinitions",
        language: "md",
        recommended: true,
        severity: Severity::Error,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct DuplicateDefinition {
    range: TextRange,
    label: String,
}

impl Rule for NoDuplicateDefinitions {
    type Query = Ast<MdDocument>;
    type State = DuplicateDefinition;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();

        let mut seen = std::collections::HashSet::new();
        let mut signals = Vec::new();

        for link_block in document
            .syntax()
            .descendants()
            .filter_map(MdLinkBlock::cast)
        {
            let label_text =
                normalize_label(&link_block.label().syntax().text_trimmed().to_string());
            if !seen.insert(label_text.clone()) {
                signals.push(DuplicateDefinition {
                    range: link_block.syntax().text_trimmed_range(),
                    label: label_text,
                });
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        // Find the MdLinkBlock node covering this range
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
            markup! { "Remove the duplicate definition." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Duplicate definition for label \""{ &state.label }"\"."
                },
            )
            .note(markup! {
                "Remove the duplicate definition. Only the first definition is used."
            }),
        )
    }
}
