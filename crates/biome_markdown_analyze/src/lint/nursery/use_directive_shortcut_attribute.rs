use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::directive_utils::find_directives;
use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Enforce shortcut id attributes in directives.
    ///
    /// Directive syntax supports `#id` as shorthand for `id="id"`.
    /// Using the shorthand form is more concise and idiomatic.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// :::note{id="main"}
    /// :::
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// :::note{#main}
    /// :::
    /// ```
    pub UseDirectiveShortcutAttribute {
        version: "next",
        name: "useDirectiveShortcutAttribute",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct ExpandedId {
    range: TextRange,
    value: String,
}

impl Rule for UseDirectiveShortcutAttribute {
    type Query = Ast<MdDocument>;
    type State = ExpandedId;
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
                    for attr in &dir.attributes {
                        if attr.name == "id"
                            && !attr.is_id_shorthand
                            && attr.value.is_some()
                        {
                            signals.push(ExpandedId {
                                range: TextRange::new(
                                    base + TextSize::from(attr.byte_offset as u32),
                                    base + TextSize::from(
                                        (attr.byte_offset + attr.byte_len) as u32,
                                    ),
                                ),
                                value: attr.value.clone().unwrap_or_default(),
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
                    "Use shortcut id instead of "{ "id=\"" }{ &state.value }{ "\"" }"."
                },
            )
            .note(markup! {
                "Use the shorthand form: #"{ &state.value }" instead."
            }),
        )
    }
}
