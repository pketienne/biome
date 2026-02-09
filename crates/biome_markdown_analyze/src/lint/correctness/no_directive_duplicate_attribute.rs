use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};
use std::collections::HashSet;

use crate::utils::directive_utils::find_directives;
use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Disallow duplicate attributes on markdown directives.
    ///
    /// Duplicate attributes on directives lead to unpredictable behavior
    /// and are always a mistake.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ::video{src="a.mp4" src="b.mp4"}
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ::video{src="a.mp4" title="My Video"}
    /// ```
    pub NoDirectiveDuplicateAttribute {
        version: "next",
        name: "noDirectiveDuplicateAttribute",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct DuplicateDirAttribute {
    range: TextRange,
    name: String,
}

impl Rule for NoDirectiveDuplicateAttribute {
    type Query = Ast<MdDocument>;
    type State = DuplicateDirAttribute;
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
                let directives = find_directives(line, byte_offset);
                for dir in &directives {
                    let mut seen = HashSet::new();
                    for attr in &dir.attributes {
                        if !seen.insert(&attr.name) {
                            signals.push(DuplicateDirAttribute {
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
                    "Duplicate attribute \""{ &state.name }"\" on directive."
                },
            )
            .note(markup! {
                "Remove the duplicate attribute."
            }),
        )
    }
}
