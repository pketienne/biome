use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_unordered_list_marker::UseConsistentUnorderedListMarkerOptions;

use crate::MarkdownRuleAction;
use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Enforce consistent unordered list marker style.
    ///
    /// Unordered lists can use `-`, `*`, or `+` as markers.
    /// This rule enforces a consistent marker.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"-"` (default):
    ///
    /// ```md
    /// * item one
    /// * item two
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - item one
    /// - item two
    /// ```
    ///
    /// ## Options
    ///
    /// ### `marker`
    ///
    /// Which marker to enforce. Default: `"-"`.
    pub UseConsistentUnorderedListMarker {
        version: "next",
        name: "useConsistentUnorderedListMarker",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentListMarker {
    range: TextRange,
    expected: String,
    actual: String,
    corrected: String,
}

impl Rule for UseConsistentUnorderedListMarker {
    type Query = Ast<MdDocument>;
    type State = InconsistentListMarker;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentUnorderedListMarkerOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let marker_opt = ctx.options().marker();
        let items = collect_list_items(&text);

        let mut signals = Vec::new();

        // For "consistent" mode, use first unordered item's marker
        let expected_marker = if marker_opt == "consistent" {
            items
                .iter()
                .find(|i| i.marker_kind.is_unordered())
                .map(|i| i.marker.clone())
        } else {
            Some(marker_opt.to_string())
        };

        let expected_marker: String = match expected_marker {
            Some(m) => m,
            None => return signals,
        };

        for item in &items {
            if item.marker_kind.is_unordered() && item.marker != expected_marker {
                let corrected = expected_marker.clone();
                signals.push(InconsistentListMarker {
                    range: TextRange::new(
                        base + TextSize::from(item.byte_offset as u32),
                        base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                    ),
                    expected: expected_marker.clone(),
                    actual: item.marker.clone(),
                    corrected,
                });
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let mut token = root
            .syntax()
            .token_at_offset(state.range.start())
            .right_biased()?;
        let mut tokens = vec![token.clone()];
        while token.text_range().end() < state.range.end() {
            token = token.next_token()?;
            tokens.push(token.clone());
        }
        let first = &tokens[0];
        let last = tokens.last()?;
        let prefix_len = u32::from(state.range.start() - first.text_range().start()) as usize;
        let suffix_start = u32::from(state.range.end() - last.text_range().start()) as usize;
        let prefix = &first.text()[..prefix_len];
        let suffix = &last.text()[suffix_start..];
        let new_text = format!("{}{}{}", prefix, state.corrected, suffix);
        let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            first.kind(),
            &new_text,
            [],
            [],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(first.clone().into(), new_token.into());
        for t in &tokens[1..] {
            let empty =
                biome_markdown_syntax::MarkdownSyntaxToken::new_detached(t.kind(), "", [], []);
            mutation.replace_element_discard_trivia(t.clone().into(), empty.into());
        }
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Use the consistent list marker." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected \""{ &state.expected }"\" but found \""{ &state.actual }"\" as list marker."
                },
            )
            .note(markup! {
                "Use a consistent unordered list marker."
            }),
        )
    }
}
