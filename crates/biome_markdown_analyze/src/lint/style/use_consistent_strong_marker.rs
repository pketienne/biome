use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_strong_marker::UseConsistentStrongMarkerOptions;

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_emphasis_markers};

declare_lint_rule! {
    /// Enforce consistent strong emphasis markers.
    ///
    /// Strong emphasis can use either double asterisks (`**text**`) or
    /// double underscores (`__text__`). This rule enforces a consistent style.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// This is __strong__ text.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// This is **strong** text.
    /// ```
    ///
    /// ## Options
    ///
    /// ### `marker`
    ///
    /// Which strong marker to enforce. Default: `"star"`.
    /// Allowed values: `"star"`, `"underscore"`, `"consistent"`.
    pub UseConsistentStrongMarker {
        version: "next",
        name: "useConsistentStrongMarker",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentStrongMarker {
    range: TextRange,
    expected: char,
    actual: char,
    corrected: String,
}

impl Rule for UseConsistentStrongMarker {
    type Query = Ast<MdDocument>;
    type State = InconsistentStrongMarker;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentStrongMarkerOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let marker_option = ctx.options().marker();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;
        let mut first_seen: Option<char> = None;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                offset += line.len() + 1;
                continue;
            }

            let code_spans = find_code_spans(line);
            let markers = find_emphasis_markers(line, &code_spans);

            for m in &markers {
                // Only check double markers (strong emphasis)
                if m.count != 2 {
                    continue;
                }

                let expected = match marker_option {
                    "star" => '*',
                    "underscore" => '_',
                    "consistent" => {
                        if let Some(first) = first_seen {
                            first
                        } else {
                            first_seen = Some(m.marker_char);
                            continue;
                        }
                    }
                    _ => '*',
                };

                if m.marker_char != expected {
                    let corrected: String = std::iter::repeat(expected).take(2).collect();
                    signals.push(InconsistentStrongMarker {
                        range: TextRange::new(
                            base + TextSize::from((offset + m.start) as u32),
                            base + TextSize::from((offset + m.start + m.count) as u32),
                        ),
                        expected,
                        actual: m.marker_char,
                        corrected,
                    });
                }
            }

            offset += line.len() + 1;
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
            markup! { "Use the consistent strong marker." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let expected = if state.expected == '*' { "**" } else { "__" };
        let actual = if state.actual == '*' { "**" } else { "__" };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected strong marker \""{ expected }"\" but found \""{ actual }"\"."
                },
            )
            .note(markup! {
                "Use consistent strong emphasis markers throughout the document."
            }),
        )
    }
}
