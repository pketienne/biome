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
    /// Enforce consistent indentation for nested unordered list items.
    ///
    /// Nested unordered list items should use a consistent number of
    /// spaces for indentation (typically 2 or 4).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// - item
    ///    - odd indent
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - item
    ///   - nested item
    /// ```
    pub UseConsistentUnorderedListIndent {
        version: "next",
        name: "useConsistentUnorderedListIndent",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct OddIndentation {
    range: TextRange,
    indent: usize,
    corrected: String,
}

impl Rule for UseConsistentUnorderedListIndent {
    type Query = Ast<MdDocument>;
    type State = OddIndentation;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let items = collect_list_items(&text);
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();

        for item in &items {
            if item.marker_kind.is_unordered() && item.indent > 0 {
                // Indent should be a multiple of 2
                if item.indent % 2 != 0 {
                    // Round up to nearest even number
                    let target_indent = (item.indent + 1) / 2 * 2;
                    let line = lines[item.line_index];
                    let trimmed = line.trim_start();
                    let corrected = format!("{}{}", " ".repeat(target_indent), trimmed);

                    signals.push(OddIndentation {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        indent: item.indent,
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
            markup! { "Fix list item indentation to nearest even number." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Unordered list item has odd indentation of "{state.indent}" spaces."
                },
            )
            .note(markup! {
                "Use a consistent multiple of 2 spaces for nested list indentation."
            }),
        )
    }
}
