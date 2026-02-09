use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::no_inline_html::NoInlineHtmlOptions;

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_html_tags};

declare_lint_rule! {
    /// Disallow inline HTML in markdown.
    ///
    /// Inline HTML can reduce the portability of markdown documents.
    /// Use markdown syntax instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// This has <em>inline HTML</em>.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// This has *emphasis* instead.
    /// ```
    ///
    /// ## Options
    ///
    /// ### `allowedElements`
    ///
    /// HTML elements that are allowed. Default: empty (no elements allowed).
    pub NoInlineHtml {
        version: "next",
        name: "noInlineHtml",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct InlineHtml {
    range: TextRange,
    tag_name: String,
}

impl Rule for NoInlineHtml {
    type Query = Ast<MdDocument>;
    type State = InlineHtml;
    type Signals = Vec<Self::State>;
    type Options = NoInlineHtmlOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let allowed = ctx.options().allowed_elements();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                offset += line.len() + 1;
                continue;
            }

            let code_spans = find_code_spans(line);
            let tags = find_html_tags(line, &code_spans);

            for tag in &tags {
                // Skip allowed elements
                if allowed.iter().any(|a: &String| a.to_lowercase() == tag.tag_name) {
                    continue;
                }

                signals.push(InlineHtml {
                    range: TextRange::new(
                        base + TextSize::from((offset + tag.start) as u32),
                        base + TextSize::from((offset + tag.end) as u32),
                    ),
                    tag_name: tag.tag_name.clone(),
                });
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
        // Remove the HTML tag entirely
        let new_text = format!("{}{}", prefix, suffix);
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
            markup! { "Remove the HTML tag." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Inline HTML element \""{ &state.tag_name }"\" is not allowed."
                },
            )
            .note(markup! {
                "Use markdown syntax instead of inline HTML."
            }),
        )
    }
}
