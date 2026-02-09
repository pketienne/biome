use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};
use std::collections::HashSet;

use crate::MarkdownRuleAction;
use crate::utils::directive_utils::find_directives;
use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Disallow duplicate attributes on markdown directives.
    ///
    /// Duplicate attributes on directives lead to unpredictable behavior
    /// and are always a mistake.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ::video{src="a.mp4" src="b.mp4"}
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ::video{src="a.mp4" title="My Video"}
    /// ```
    pub NoDirectiveDuplicateAttribute {
        version: "next",
        name: "noDirectiveDuplicateAttribute",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct DuplicateDirAttribute {
    range: TextRange,
    corrected: String,
    name: String,
}

impl Rule for NoDirectiveDuplicateAttribute {
    type Query = Ast<MdDocument>;
    type State = DuplicateDirAttribute;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut byte_offset: usize = 0;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if !tracker.is_inside_fence() {
                let directives = find_directives(line, byte_offset);
                for dir in &directives {
                    let mut seen = HashSet::new();
                    for attr in &dir.attributes {
                        if !seen.insert(&attr.name) {
                            // Build range that includes any leading whitespace
                            let attr_start = attr.byte_offset;
                            let attr_end = attr.byte_offset + attr.byte_len;
                            // Look back to consume leading whitespace
                            let mut start = attr_start;
                            let text_bytes = text.as_bytes();
                            while start > 0 && text_bytes[start - 1] == b' ' {
                                start -= 1;
                            }
                            signals.push(DuplicateDirAttribute {
                                range: TextRange::new(
                                    base + TextSize::from(start as u32),
                                    base + TextSize::from(attr_end as u32),
                                ),
                                corrected: String::new(),
                                name: attr.name.clone(),
                            });
                        }
                    }
                }
            }
            byte_offset += line.len() + 1;
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
            let empty = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
                t.kind(),
                "",
                [],
                [],
            );
            mutation.replace_element_discard_trivia(t.clone().into(), empty.into());
        }
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove the duplicate attribute." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Duplicate attribute \""{ &state.name }"\" on directive."
                },
            )
            .note(markup! {
                "Remove the duplicate attribute."
            }),
        )
    }
}
