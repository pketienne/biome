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
    /// Enforce shorthand boolean attributes in MDX JSX elements.
    ///
    /// In JSX, `prop={true}` can be written as just `prop`. The shorthand
    /// form is more idiomatic and concise.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <Component disabled={true} />
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <Component disabled />
    /// ```
    pub UseMdxJsxShorthandAttribute {
        version: "next",
        name: "useMdxJsxShorthandAttribute",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct LonghandBoolean {
    range: TextRange,
    name: String,
    corrected: String,
}

impl Rule for UseMdxJsxShorthandAttribute {
    type Query = Ast<MdDocument>;
    type State = LonghandBoolean;
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
                    for attr in &elem.attributes {
                        if let Some(val) = &attr.value {
                            if val == "{true}" {
                                signals.push(LonghandBoolean {
                                    range: TextRange::new(
                                        base + TextSize::from(attr.byte_offset as u32),
                                        base + TextSize::from(
                                            (attr.byte_offset + attr.byte_len) as u32,
                                        ),
                                    ),
                                    name: attr.name.clone(),
                                    corrected: attr.name.clone(),
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
            markup! { "Use shorthand boolean attribute." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Attribute \""{ &state.name }"\" uses longhand \"{true}\" instead of shorthand."
                },
            )
            .note(markup! {
                "Use the shorthand form: just \""{ &state.name }"\" without a value."
            }),
        )
    }
}
