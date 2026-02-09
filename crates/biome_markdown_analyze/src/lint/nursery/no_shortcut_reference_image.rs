use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;
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
    }
}

pub struct ShortcutRefImage {
    range: TextRange,
}

impl Rule for NoShortcutReferenceImage {
    type Query = Ast<MdDocument>;
    type State = ShortcutRefImage;
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
                    if rlink.is_image && rlink.kind == ReferenceLinkKind::Shortcut {
                        signals.push(ShortcutRefImage {
                            range: TextRange::new(
                                base + TextSize::from((offset + rlink.start) as u32),
                                base + TextSize::from((offset + rlink.end) as u32),
                            ),
                        });
                    }
                }
            }

            offset += line.len() + 1;
        }

        signals
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
