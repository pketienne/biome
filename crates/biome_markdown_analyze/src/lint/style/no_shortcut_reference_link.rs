use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{ReferenceLinkKind, find_code_spans, find_reference_links};

declare_lint_rule! {
    /// Disallow shortcut reference links.
    ///
    /// Shortcut reference links (`[text]`) should use the full or collapsed
    /// form (`[text][label]` or `[text][]`) for clarity.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// See [link] for details.
    ///
    /// [link]: https://example.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// See [link][] for details.
    ///
    /// [link]: https://example.com
    /// ```
    pub NoShortcutReferenceLink {
        version: "next",
        name: "noShortcutReferenceLink",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct ShortcutRefLink {
    range: TextRange,
    corrected: String,
}

impl Rule for NoShortcutReferenceLink {
    type Query = Ast<MdDocument>;
    type State = ShortcutRefLink;
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

            if !tracker.is_inside_fence() {
                let code_spans = find_code_spans(line);
                let refs = find_reference_links(line, &code_spans);

                for rlink in refs {
                    if !rlink.is_image && rlink.kind == ReferenceLinkKind::Shortcut {
                        let original = &line[rlink.start..rlink.end];
                        let corrected = format!("{}[]", original);
                        signals.push(ShortcutRefLink {
                            range: TextRange::new(
                                base + TextSize::from((offset + rlink.start) as u32),
                                base + TextSize::from((offset + rlink.end) as u32),
                            ),
                            corrected,
                        });
                    }
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
            markup! { "Expand to collapsed reference link." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Shortcut reference links should use collapsed or full form."
                },
            )
            .note(markup! {
                "Use "<Emphasis>"[text][]"</Emphasis>" or "<Emphasis>"[text][label]"</Emphasis>" instead."
            }),
        )
    }
}
