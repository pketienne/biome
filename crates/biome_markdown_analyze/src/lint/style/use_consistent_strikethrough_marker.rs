use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_strikethrough_marker::UseConsistentStrikethroughMarkerOptions;

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, is_in_code_span};

declare_lint_rule! {
    /// Enforce consistent strikethrough marker style.
    ///
    /// Strikethrough can use `~~text~~` (double tilde) or `~text~`
    /// (single tilde). This rule enforces consistency.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"double-tilde"` (default):
    ///
    /// ```md
    /// ~single tilde~
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ~~double tilde~~
    /// ```
    ///
    /// ## Options
    ///
    /// ### `marker`
    ///
    /// Which marker style to enforce. Default: `"consistent"`.
    pub UseConsistentStrikethroughMarker {
        version: "next",
        name: "useConsistentStrikethroughMarker",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentStrikethrough {
    range: TextRange,
    expected: &'static str,
    actual: &'static str,
    corrected: String,
}

impl Rule for UseConsistentStrikethroughMarker {
    type Query = Ast<MdDocument>;
    type State = InconsistentStrikethrough;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentStrikethroughMarkerOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let marker_style = ctx.options().marker();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        // Find all strikethrough markers and their styles
        let mut found_markers: Vec<(&'static str, usize, usize)> = Vec::new(); // (style, line_offset+start, end)

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                continue;
            }

            let line_offset: usize = text.lines().take(line_idx).map(|l| l.len() + 1).sum();
            let code_spans = find_code_spans(line);
            let bytes = line.as_bytes();
            let mut i = 0;

            while i < bytes.len() {
                if is_in_code_span(i, &code_spans) {
                    i += 1;
                    continue;
                }

                if bytes[i] == b'~' {
                    let start = i;
                    let mut count = 0;
                    while i < bytes.len() && bytes[i] == b'~' {
                        count += 1;
                        i += 1;
                    }

                    let style = if count == 2 { "double-tilde" } else if count == 1 { "tilde" } else { i += 1; continue; };
                    found_markers.push((style, line_offset + start, line_offset + start + count));
                } else {
                    i += 1;
                }
            }
        }

        if found_markers.is_empty() {
            return signals;
        }

        let expected = match marker_style {
            "tilde" => "tilde",
            "double-tilde" => "double-tilde",
            _ => {
                // consistent: use first marker style
                found_markers[0].0
            }
        };

        for &(style, start, end) in &found_markers {
            if style != expected {
                let corrected = if expected == "double-tilde" {
                    "~~".to_string()
                } else {
                    "~".to_string()
                };
                signals.push(InconsistentStrikethrough {
                    range: TextRange::new(
                        base + TextSize::from(start as u32),
                        base + TextSize::from(end as u32),
                    ),
                    expected,
                    actual: style,
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
            markup! { "Use the consistent strikethrough marker." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected "{state.expected}" strikethrough marker but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use a consistent strikethrough marker style."
            }),
        )
    }
}
