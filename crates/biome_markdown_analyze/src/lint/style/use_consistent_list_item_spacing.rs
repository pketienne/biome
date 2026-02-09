use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_list_item_spacing::UseConsistentListItemSpacingOptions;

use crate::MarkdownRuleAction;
use crate::utils::list_utils::collect_list_blocks;

declare_lint_rule! {
    /// Enforce consistent spacing between list items.
    ///
    /// Lists should either be compact (no blank lines) or loose
    /// (blank lines between every item), but not mixed.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// - item one
    ///
    /// - item two
    /// - item three
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - item one
    /// - item two
    /// - item three
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which style to enforce. Default: `"consistent"`.
    pub UseConsistentListItemSpacing {
        version: "next",
        name: "useConsistentListItemSpacing",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentSpacing {
    range: TextRange,
    expected: &'static str,
    actual: &'static str,
    corrected: String,
}

impl Rule for UseConsistentListItemSpacing {
    type Query = Ast<MdDocument>;
    type State = InconsistentSpacing;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentListItemSpacingOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let blocks = collect_list_blocks(&text);
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();

        for block in &blocks {
            if block.items.len() < 2 {
                continue;
            }

            // Determine if gaps between items have blank lines
            let mut has_loose = false;
            let mut has_compact = false;

            for pair in block.items.windows(2) {
                let gap_has_blank = (pair[0].line_index + 1..pair[1].line_index)
                    .any(|l| l < lines.len() && lines[l].trim().is_empty());
                if gap_has_blank {
                    has_loose = true;
                } else if pair[1].line_index == pair[0].line_index + 1 {
                    has_compact = true;
                }
            }

            let expected_style = match style {
                "compact" => "compact",
                "loose" => "loose",
                _ => {
                    // consistent: use the first gap's style
                    if has_loose { "loose" } else { "compact" }
                }
            };

            // Only flag if there's a mix, or if forcing a specific style
            if style == "consistent" && !(has_loose && has_compact) {
                continue;
            }

            for pair in block.items.windows(2) {
                let gap_has_blank = (pair[0].line_index + 1..pair[1].line_index)
                    .any(|l| l < lines.len() && lines[l].trim().is_empty());
                let is_compact = !gap_has_blank && pair[1].line_index == pair[0].line_index + 1;

                let actual = if gap_has_blank { "loose" } else if is_compact { "compact" } else { continue };

                if actual != expected_style {
                    // Compute the range from end of previous item to end of current item
                    let gap_start = pair[0].byte_offset + pair[0].byte_len;
                    let item_end = pair[1].byte_offset + pair[1].byte_len;
                    let item_line = &text[pair[1].byte_offset..item_end];

                    let corrected = if expected_style == "compact" {
                        // Remove blank lines: just newline + item
                        format!("\n{}", item_line)
                    } else {
                        // Add blank line: two newlines + item
                        format!("\n\n{}", item_line)
                    };

                    signals.push(InconsistentSpacing {
                        range: TextRange::new(
                            base + TextSize::from(gap_start as u32),
                            base + TextSize::from(item_end as u32),
                        ),
                        expected: if expected_style == "compact" {
                            "compact"
                        } else {
                            "loose"
                        },
                        actual: if actual == "compact" { "compact" } else { "loose" },
                        corrected,
                    });
                }
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
            markup! { "Normalize list item spacing." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected "{state.expected}" spacing but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use consistent spacing between list items."
            }),
        )
    }
}
