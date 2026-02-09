use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use biome_rule_options::use_consistent_list_item_indent::UseConsistentListItemIndentOptions;

use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Enforce consistent spacing between list marker and content.
    ///
    /// The space between the marker and content should follow a consistent style.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"one"` (default), extra spaces are flagged:
    ///
    /// ```md
    /// -  item with extra space
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - item with one space
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which indent style to enforce. Default: `"one"`.
    pub UseConsistentListItemIndent {
        version: "next",
        name: "useConsistentListItemIndent",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentItemIndent {
    range: TextRange,
    spaces: usize,
    corrected: String,
}

impl Rule for UseConsistentListItemIndent {
    type Query = Ast<MdDocument>;
    type State = InconsistentItemIndent;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentListItemIndentOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let items = collect_list_items(&text);
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();

        let expected_offset = match style {
            "one" => Some(1usize),
            "tab" => Some(4usize),
            _ => Some(1usize),
        };

        for item in &items {
            if let Some(expected) = expected_offset {
                if item.content_offset != expected {
                    // Build corrected line: indent + marker + expected spaces + content
                    let line = lines[item.line_index];
                    let indent_str = &line[..item.indent];
                    let corrected = format!(
                        "{}{}{}{}",
                        indent_str,
                        item.marker,
                        " ".repeat(expected),
                        item.content
                    );

                    signals.push(InconsistentItemIndent {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        spaces: item.content_offset,
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
            markup! { "Fix spacing between list marker and content." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "List item has "{state.spaces}" space(s) between marker and content."
                },
            )
            .note(markup! {
                "Use consistent spacing between list marker and content."
            }),
        )
    }
}
