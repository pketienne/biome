use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;
use crate::utils::mdx_utils::find_jsx_elements;

declare_lint_rule! {
    /// Enforce self-closing tags for MDX JSX elements without children.
    ///
    /// Components and elements without children should use self-closing tags
    /// for brevity and clarity.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <Component></Component>
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <Component />
    /// ```
    pub UseMdxJsxSelfClosing {
        version: "next",
        name: "useMdxJsxSelfClosing",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct NotSelfClosing {
    range: TextRange,
    tag: String,
    corrected: String,
}

impl Rule for UseMdxJsxSelfClosing {
    type Query = Ast<MdDocument>;
    type State = NotSelfClosing;
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
                let elements = find_jsx_elements(line, byte_offset);
                for elem in &elements {
                    if !elem.self_closing && elem.has_closing_tag {
                        // Check if there's content between opening and closing tag
                        let closing_tag = format!("</{}>", elem.tag);
                        let opening_end = elem.end - byte_offset;
                        if let Some(close_pos) = line[opening_end..].find(&closing_tag) {
                            let between = &line[opening_end..opening_end + close_pos];
                            if between.trim().is_empty() {
                                let full_end = opening_end + close_pos + closing_tag.len();
                                // Build self-closing version from the opening tag
                                let open_start = elem.start - byte_offset;
                                let opening_text = &line[open_start..opening_end];
                                // Replace trailing ">" with " />"
                                let corrected = if opening_text.ends_with('>') {
                                    format!("{} />", &opening_text[..opening_text.len() - 1])
                                } else {
                                    format!("{} />", opening_text)
                                };
                                signals.push(NotSelfClosing {
                                    range: TextRange::new(
                                        base + TextSize::from(elem.start as u32),
                                        base + TextSize::from((byte_offset + full_end) as u32),
                                    ),
                                    tag: elem.tag.clone(),
                                    corrected,
                                });
                            }
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
        // Collect all tokens overlapping the range
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
            markup! { "Convert to self-closing tag." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Element \""{ &state.tag }"\" has no children and should be self-closing."
                },
            )
            .note(markup! {
                "Use a self-closing tag: <"{ &state.tag }" />."
            }),
        )
    }
}
