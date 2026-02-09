use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use crate::MarkdownRuleAction;
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_linebreak_style::UseConsistentLinebreakStyleOptions;

declare_lint_rule! {
    /// Enforce consistent line endings.
    ///
    /// Line endings can be either LF (`\n`) or CRLF (`\r\n`). This rule
    /// enforces consistent usage.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// Files with mixed or wrong line endings.
    ///
    /// ### Valid
    ///
    /// Files with consistent line endings matching the configured style.
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which line ending style to enforce. Default: `"lf"`.
    /// Allowed values: `"lf"`, `"crlf"`.
    pub UseConsistentLinebreakStyle {
        version: "next",
        name: "useConsistentLinebreakStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentLinebreak {
    range: TextRange,
    expected: &'static str,
}

impl Rule for UseConsistentLinebreakStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentLinebreak;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentLinebreakStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let expect_crlf = style == "crlf";
        let mut signals = Vec::new();

        let bytes = text.as_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            if byte == b'\n' {
                let has_cr = i > 0 && bytes[i - 1] == b'\r';

                if expect_crlf && !has_cr {
                    // Expected CRLF but found LF
                    signals.push(InconsistentLinebreak {
                        range: TextRange::new(
                            base + TextSize::from(i as u32),
                            base + TextSize::from((i + 1) as u32),
                        ),
                        expected: "crlf",
                    });
                } else if !expect_crlf && has_cr {
                    // Expected LF but found CRLF
                    signals.push(InconsistentLinebreak {
                        range: TextRange::new(
                            base + TextSize::from((i - 1) as u32),
                            base + TextSize::from((i + 1) as u32),
                        ),
                        expected: "lf",
                    });
                }
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let corrected = if state.expected == "lf" { "\n" } else { "\r\n" };
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
        let new_text = format!("{}{}{}", prefix, corrected, suffix);
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
            markup! { "Fix the line ending." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let expected = if state.expected == "lf" {
            "LF (\\n)"
        } else {
            "CRLF (\\r\\n)"
        };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected "{expected}" line endings."
                },
            )
            .note(markup! {
                "Use consistent line endings throughout the document."
            }),
        )
    }
}
