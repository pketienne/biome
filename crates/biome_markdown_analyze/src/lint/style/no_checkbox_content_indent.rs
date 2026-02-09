use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Disallow extra spacing after checkbox brackets.
    ///
    /// There should be exactly one space between the closing `]`
    /// of a checkbox and the content text.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// - [x]  extra space
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - [x] correct
    /// ```
    pub NoCheckboxContentIndent {
        version: "next",
        name: "noCheckboxContentIndent",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct BadCheckboxIndent {
    range: TextRange,
    spaces: usize,
    corrected: String,
}

impl Rule for NoCheckboxContentIndent {
    type Query = Ast<MdDocument>;
    type State = BadCheckboxIndent;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let items = collect_list_items(&text);
        let mut signals = Vec::new();

        for item in &items {
            if let Some(ref cb) = item.checkbox {
                if cb.content_spacing != 1 {
                    let line_text = &text[item.byte_offset..item.byte_offset + item.byte_len];
                    let corrected = if let Some(bracket_pos) = line_text.find(']') {
                        let before = &line_text[..bracket_pos + 1];
                        let after = line_text[bracket_pos + 1..].trim_start();
                        format!("{} {}", before, after)
                    } else {
                        line_text.to_string()
                    };
                    signals.push(BadCheckboxIndent {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        spaces: cb.content_spacing,
                        corrected,
                    });
                }
            }
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
            markup! { "Normalize spacing after checkbox." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Checkbox has "{state.spaces}" space(s) after brackets instead of 1."
                },
            )
            .note(markup! {
                "Use exactly one space between the checkbox brackets and content."
            }),
        )
    }
}
