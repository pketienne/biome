use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_ordered_list_marker::UseConsistentOrderedListMarkerOptions;

use crate::MarkdownRuleAction;
use crate::utils::list_utils::{ListMarkerKind, collect_list_items};

declare_lint_rule! {
    /// Enforce consistent ordered list marker delimiter.
    ///
    /// Ordered lists can use `.` or `)` after the number.
    /// This rule enforces consistency.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"."` (default):
    ///
    /// ```md
    /// 1) first
    /// 2) second
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// 1. first
    /// 2. second
    /// ```
    ///
    /// ## Options
    ///
    /// ### `delimiter`
    ///
    /// Which delimiter to enforce. Default: `"."`.
    pub UseConsistentOrderedListMarker {
        version: "next",
        name: "useConsistentOrderedListMarker",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentOrderedMarker {
    range: TextRange,
    expected: &'static str,
    actual: &'static str,
    corrected: String,
}

impl Rule for UseConsistentOrderedListMarker {
    type Query = Ast<MdDocument>;
    type State = InconsistentOrderedMarker;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentOrderedListMarkerOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let delimiter = ctx.options().delimiter();
        let items = collect_list_items(&text);

        let mut signals = Vec::new();

        let expected_kind = if delimiter == "consistent" {
            items
                .iter()
                .find(|i| i.marker_kind.is_ordered())
                .map(|i| i.marker_kind)
        } else if delimiter == ")" {
            Some(ListMarkerKind::OrderedParen)
        } else {
            Some(ListMarkerKind::OrderedDot)
        };

        let expected_kind = match expected_kind {
            Some(k) => k,
            None => return signals,
        };

        for item in &items {
            if item.marker_kind.is_ordered() && item.marker_kind != expected_kind {
                // Compute corrected marker: replace the delimiter character
                let num_part = item.marker.trim_end_matches('.').trim_end_matches(')');
                let new_delim = if expected_kind == ListMarkerKind::OrderedDot {
                    "."
                } else {
                    ")"
                };
                let corrected = format!("{}{}", num_part, new_delim);
                signals.push(InconsistentOrderedMarker {
                    range: TextRange::new(
                        base + TextSize::from(item.byte_offset as u32),
                        base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                    ),
                    expected: expected_kind.marker_char(),
                    actual: item.marker_kind.marker_char(),
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
            markup! { "Use the consistent ordered list delimiter." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected \""{ state.expected }"\" but found \""{ state.actual }"\" as ordered list delimiter."
                },
            )
            .note(markup! {
                "Use a consistent ordered list delimiter."
            }),
        )
    }
}
