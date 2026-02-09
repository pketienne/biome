use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;
use crate::utils::mdx_utils::{find_jsx_elements, is_void_element};

declare_lint_rule! {
    /// Disallow children for void HTML elements in MDX.
    ///
    /// Void HTML elements like `<br>`, `<hr>`, and `<img>` cannot have children.
    /// Using them with content is always an error.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <hr>content</hr>
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <hr />
    /// ```
    pub NoMdxJsxVoidChildren {
        version: "next",
        name: "noMdxJsxVoidChildren",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct VoidWithChildren {
    range: TextRange,
    tag: String,
}

impl Rule for NoMdxJsxVoidChildren {
    type Query = Ast<MdDocument>;
    type State = VoidWithChildren;
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
                    if is_void_element(&elem.tag)
                        && !elem.self_closing
                        && elem.has_closing_tag
                    {
                        signals.push(VoidWithChildren {
                            range: TextRange::new(
                                base + TextSize::from(elem.start as u32),
                                base + TextSize::from(elem.end as u32),
                            ),
                            tag: elem.tag.clone(),
                        });
                    }
                }
            }
            byte_offset += line.len() + 1;
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Void element \""{ &state.tag }"\" cannot have children."
                },
            )
            .note(markup! {
                "Use a self-closing tag instead."
            }),
        )
    }
}
