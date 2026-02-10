use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{AnyMdBlock, MdDocument, MdLinkBlock};
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt, TextRange};

use crate::MarkdownRuleAction;
use crate::utils::definition_utils::normalize_label;

declare_lint_rule! {
    /// Enforce that link reference definitions are placed at the end.
    ///
    /// Link reference definitions should be grouped at the end of the
    /// document for readability.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// Some text.
    ///
    /// More text.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// Some text.
    ///
    /// More text.
    /// ```
    pub UseDefinitionsAtEnd {
        version: "next",
        name: "useDefinitionsAtEnd",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct DefinitionNotAtEnd {
    range: TextRange,
    label: String,
}

impl Rule for UseDefinitionsAtEnd {
    type Query = Ast<MdDocument>;
    type State = DefinitionNotAtEnd;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let block_list = document.value();
        let blocks: Vec<AnyMdBlock> = block_list.iter().collect();
        let mut signals = Vec::new();

        if blocks.is_empty() {
            return signals;
        }

        // Find index of the last non-definition block
        let mut last_non_def_idx = None;
        for (i, block) in blocks.iter().enumerate() {
            if block.as_any_leaf_block().and_then(|b| b.as_md_link_block()).is_none() {
                last_non_def_idx = Some(i);
            }
        }

        let Some(last_non_def) = last_non_def_idx else {
            // All blocks are definitions — nothing to flag
            return signals;
        };

        // Flag any definitions that appear before the last non-definition block
        for (i, block) in blocks.iter().enumerate() {
            if i < last_non_def {
                if let Some(link_block) = block.as_any_leaf_block().and_then(|b| b.as_md_link_block()) {
                    let label =
                        normalize_label(&link_block.label().syntax().text_trimmed().to_string());
                    signals.push(DefinitionNotAtEnd {
                        range: link_block.syntax().text_trimmed_range(),
                        label,
                    });
                }
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
        let label = &state.label;
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Definition \""{ label }"\" is not at the end of the document."
                },
            )
            .note(markup! {
                "Move link reference definitions to the end of the document."
            }),
        )
    }
}
