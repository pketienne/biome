use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdMdxJsxElement;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

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
    type Query = Ast<MdMdxJsxElement>;
    type State = NotSelfClosing;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();

        // Only interested in non-self-closing tags (no `/` before `>`)
        if node.slash_token().is_some() {
            return Vec::new();
        }

        let tag = node.name().syntax().text_trimmed().to_string();
        let elem_end = node.syntax().text_range_with_trivia().end();

        // Look at the text after this element in the parent for `</tagname>`
        let parent = match node.syntax().parent() {
            Some(p) => p,
            None => return Vec::new(),
        };
        let parent_text = parent.text_with_trivia().to_string();
        let parent_start = parent.text_range_with_trivia().start();
        let offset = u32::from(elem_end - parent_start) as usize;
        let after = &parent_text[offset..];

        let closing_tag = format!("</{}>", tag);
        if let Some(close_pos) = after.find(&closing_tag) {
            let between = &after[..close_pos];
            if between.trim().is_empty() {
                // Build the self-closing version
                let elem_text = node.syntax().text_trimmed().to_string();
                let corrected = if elem_text.ends_with('>') {
                    format!("{} />", &elem_text[..elem_text.len() - 1])
                } else {
                    format!("{} />", elem_text)
                };

                let full_end = elem_end + TextSize::from((close_pos + closing_tag.len()) as u32);
                return vec![NotSelfClosing {
                    range: TextRange::new(node.syntax().text_trimmed_range().start(), full_end),
                    tag,
                    corrected,
                }];
            }
        }

        Vec::new()
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, &state.corrected)?;
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
