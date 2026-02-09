use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;
use crate::utils::mdx_utils::find_jsx_elements;

declare_lint_rule! {
    /// Enforce sorted attributes on MDX JSX elements.
    ///
    /// Keeping attributes in alphabetical order improves readability and
    /// makes diffs cleaner.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <Component zebra="1" alpha="2" />
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <Component alpha="2" zebra="1" />
    /// ```
    pub UseSortedMdxJsxAttributes {
        version: "next",
        name: "useSortedMdxJsxAttributes",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct UnsortedAttribute {
    range: TextRange,
    first_unsorted: String,
    previous: String,
}

impl Rule for UseSortedMdxJsxAttributes {
    type Query = Ast<MdDocument>;
    type State = UnsortedAttribute;
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
                    if elem.attributes.len() >= 2 {
                        for i in 1..elem.attributes.len() {
                            let prev = &elem.attributes[i - 1].name;
                            let curr = &elem.attributes[i].name;
                            if curr.to_lowercase() < prev.to_lowercase() {
                                signals.push(UnsortedAttribute {
                                    range: TextRange::new(
                                        base + TextSize::from(elem.attributes[i].byte_offset as u32),
                                        base + TextSize::from(
                                            (elem.attributes[i].byte_offset
                                                + elem.attributes[i].byte_len)
                                                as u32,
                                        ),
                                    ),
                                    first_unsorted: curr.clone(),
                                    previous: prev.clone(),
                                });
                                break;
                            }
                        }
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
                    "Attribute \""{ &state.first_unsorted }"\" should come before \""{ &state.previous }"\"."
                },
            )
            .note(markup! {
                "Sort JSX attributes alphabetically."
            }),
        )
    }
}
