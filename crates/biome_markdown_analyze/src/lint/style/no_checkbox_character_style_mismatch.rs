use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::no_checkbox_character_style_mismatch::NoCheckboxCharacterStyleMismatchOptions;

use crate::MarkdownRuleAction;
use crate::utils::list_utils::collect_list_items;

declare_lint_rule! {
    /// Enforce consistent checkbox character style.
    ///
    /// Checked checkboxes can use `x` or `X`. This rule enforces
    /// a consistent character.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"lowercase"` (default):
    ///
    /// ```md
    /// - [X] done
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - [x] done
    /// ```
    ///
    /// ## Options
    ///
    /// ### `checked`
    ///
    /// Which character to use for checked checkboxes. Default: `"lowercase"`.
    pub NoCheckboxCharacterStyleMismatch {
        version: "next",
        name: "noCheckboxCharacterStyleMismatch",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct MismatchedCheckbox {
    range: TextRange,
    expected: char,
    actual: char,
    corrected: String,
}

impl Rule for NoCheckboxCharacterStyleMismatch {
    type Query = Ast<MdDocument>;
    type State = MismatchedCheckbox;
    type Signals = Vec<Self::State>;
    type Options = NoCheckboxCharacterStyleMismatchOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let checked_style = ctx.options().checked();
        let items = collect_list_items(&text);
        let mut signals = Vec::new();

        let expected_char = match checked_style {
            "uppercase" => 'X',
            "consistent" => {
                // Use the first checked checkbox's character
                items
                    .iter()
                    .filter_map(|i| i.checkbox.as_ref())
                    .find(|c| c.check_char == 'x' || c.check_char == 'X')
                    .map(|c| c.check_char)
                    .unwrap_or('x')
            }
            _ => 'x', // "lowercase" (default)
        };

        for item in &items {
            if let Some(ref cb) = item.checkbox {
                if (cb.check_char == 'x' || cb.check_char == 'X')
                    && cb.check_char != expected_char
                {
                    // Build corrected line by replacing the checkbox character
                    let line = &text[item.byte_offset..item.byte_offset + item.byte_len];
                    let corrected = line.replacen(cb.check_char, &expected_char.to_string(), 1);
                    signals.push(MismatchedCheckbox {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        expected: expected_char,
                        actual: cb.check_char,
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
            markup! { "Normalize the checkbox character." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let expected = state.expected.to_string();
        let actual = state.actual.to_string();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected checkbox character \""{ expected }"\" but found \""{ actual }"\"."
                },
            )
            .note(markup! {
                "Use a consistent checkbox character style."
            }),
        )
    }
}
