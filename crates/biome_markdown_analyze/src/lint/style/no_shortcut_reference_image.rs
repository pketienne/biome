use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdParagraph;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;
use crate::utils::inline_utils::{ReferenceLinkKind, find_code_spans, find_reference_links};

declare_lint_rule! {
    /// Disallow shortcut reference images.
    ///
    /// Shortcut reference images (`![text]`) should use the full or collapsed
    /// form (`![text][label]` or `![text][]`) for clarity.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ![image]
    ///
    /// [image]: image.png
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ![image][]
    ///
    /// [image]: image.png
    /// ```
    pub NoShortcutReferenceImage {
        version: "next",
        name: "noShortcutReferenceImage",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct ShortcutRefImage {
    range: TextRange,
    corrected: String,
}

impl Rule for NoShortcutReferenceImage {
    type Query = Ast<MdParagraph>;
    type State = ShortcutRefImage;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let paragraph = ctx.query();
        let text = paragraph.syntax().text_trimmed().to_string();
        let base = paragraph.syntax().text_trimmed_range().start();
        let mut signals = Vec::new();
        let mut offset = 0usize;

        for line in text.lines() {
            let code_spans = find_code_spans(line);
            let refs = find_reference_links(line, &code_spans);

            for rlink in refs {
                if rlink.is_image && rlink.kind == ReferenceLinkKind::Shortcut {
                    let original = &line[rlink.start..rlink.end];
                    let corrected = format!("{}[]", original);
                    signals.push(ShortcutRefImage {
                        range: TextRange::new(
                            base + TextSize::from((offset + rlink.start) as u32),
                            base + TextSize::from((offset + rlink.end) as u32),
                        ),
                        corrected,
                    });
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, &state.corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Expand to collapsed reference image." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Shortcut reference images should use collapsed or full form."
                },
            )
            .note(markup! {
                "Use "<Emphasis>"![text][]"</Emphasis>" or "<Emphasis>"![text][label]"</Emphasis>" instead."
            }),
        )
    }
}
