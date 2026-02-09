use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList, TextRange};

declare_lint_rule! {
    /// Heading levels should only increment by one level at a time.
    ///
    /// Headings should increment by one level at a time. Jumping from `h1` to `h3`
    /// makes the document structure harder to follow and hurts accessibility.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// # Heading 1
    /// ### Heading 3
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Heading 1
    /// ## Heading 2
    /// ### Heading 3
    /// ```
    pub UseHeadingIncrement {
        version: "next",
        name: "useHeadingIncrement",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct HeadingLevelJump {
    current_level: usize,
    previous_level: usize,
    range: TextRange,
}

impl Rule for UseHeadingIncrement {
    type Query = Ast<MdDocument>;
    type State = HeadingLevelJump;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let mut signals = Vec::new();
        let mut prev_level: Option<usize> = None;

        for node in document.syntax().descendants() {
            if let Some(header) = MdHeader::cast_ref(&node) {
                let level = header.before().len();
                if let Some(prev) = prev_level {
                    if level > prev + 1 {
                        signals.push(HeadingLevelJump {
                            current_level: level,
                            previous_level: prev,
                            range: header.syntax().text_trimmed_range(),
                        });
                    }
                }
                prev_level = Some(level);
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Heading level jumped from h"{state.previous_level.to_string()}" to h"{state.current_level.to_string()}"."
                },
            )
            .note(markup! {
                "Headings should increment by one level at a time."
            }),
        )
    }
}
