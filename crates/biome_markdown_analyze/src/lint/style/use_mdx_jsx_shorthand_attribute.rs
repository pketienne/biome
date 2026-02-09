use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

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
    }
}

pub struct LonghandBoolean {
    range: TextRange,
    name: String,
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
