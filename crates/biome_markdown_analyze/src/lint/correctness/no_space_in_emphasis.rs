use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, is_in_code_span};

declare_lint_rule! {
    /// Disallow spaces inside emphasis markers.
    ///
    /// Spaces immediately after opening or before closing emphasis markers
    /// can prevent proper rendering in some parsers.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// This is * not emphasized * text.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// This is *emphasized* text.
    /// ```
    pub NoSpaceInEmphasis {
        version: "next",
        name: "noSpaceInEmphasis",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct SpaceInEmphasis {
    range: TextRange,
    is_opening: bool,
    space_pos: TextSize,
}

impl Rule for NoSpaceInEmphasis {
    type Query = Ast<MdDocument>;
    type State = SpaceInEmphasis;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                offset += line.len() + 1;
                continue;
            }

            let bytes = line.as_bytes();
            let code_spans = find_code_spans(line);

            // Collect marker runs: (start, count, marker_byte)
            let mut runs: Vec<(usize, usize, u8)> = Vec::new();
            let mut i = 0;
            while i < bytes.len() {
                if is_in_code_span(i, &code_spans) {
                    i += 1;
                    continue;
                }
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'*' || bytes[i] == b'_' {
                    let marker_byte = bytes[i];
                    let start = i;
                    while i < bytes.len() && bytes[i] == marker_byte {
                        i += 1;
                    }
                    let count = i - start;
                    if count <= 3 {
                        runs.push((start, count, marker_byte));
                    }
                } else {
                    i += 1;
                }
            }

            // Find matched pairs: opening marker with trailing space,
            // closing marker with leading space (same char and count).
            let mut used = vec![false; runs.len()];
            for idx in 0..runs.len() {
                if used[idx] {
                    continue;
                }
                let (start, count, marker) = runs[idx];
                let after = start + count;
                // Opening marker must be followed by a space
                if after >= bytes.len() || bytes[after] != b' ' {
                    continue;
                }

                for jdx in (idx + 1)..runs.len() {
                    if used[jdx] {
                        continue;
                    }
                    let (close_start, close_count, close_marker) = runs[jdx];
                    if close_marker != marker || close_count != count {
                        continue;
                    }
                    // Closing marker must be preceded by a space
                    if close_start == 0 || bytes[close_start - 1] != b' ' {
                        continue;
                    }

                    used[idx] = true;
                    used[jdx] = true;

                    // Report opening space (after opening markers)
                    signals.push(SpaceInEmphasis {
                        range: TextRange::new(
                            base + TextSize::from((offset + start) as u32),
                            base + TextSize::from((offset + after + 1) as u32),
                        ),
                        is_opening: true,
                        space_pos: base + TextSize::from((offset + after) as u32),
                    });

                    // Report closing space (before closing markers)
                    let close_end = close_start + close_count;
                    signals.push(SpaceInEmphasis {
                        range: TextRange::new(
                            base + TextSize::from((offset + close_start - 1) as u32),
                            base + TextSize::from((offset + close_end) as u32),
                        ),
                        is_opening: false,
                        space_pos: base + TextSize::from((offset + close_start - 1) as u32),
                    });

                    break;
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        // Find the token containing the space character
        let token = root
            .syntax()
            .token_at_offset(state.space_pos)
            .right_biased()?;
        let token_text = token.text().to_string();
        let token_start = token.text_range().start();
        let rel = u32::from(state.space_pos - token_start) as usize;

        if rel >= token_text.len() {
            return None;
        }

        // Remove the space character at the relative position
        let new_text = format!("{}{}", &token_text[..rel], &token_text[rel + 1..]);

        let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            token.kind(),
            &new_text,
            [],
            [],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(token.into(), new_token.into());
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove space from emphasis marker." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let position = if state.is_opening { "after opening" } else { "before closing" };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Unexpected space "{position}" emphasis marker."
                },
            )
            .note(markup! {
                "Remove the space to ensure emphasis renders correctly."
            }),
        )
    }
}
