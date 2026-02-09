use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList, TextRange};

declare_lint_rule! {
    /// Disallow multiple top-level headings in a document.
    ///
    /// A document should have a single top-level heading (`# heading`) that
    /// serves as the document title. Multiple top-level headings indicate
    /// a structural problem.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// # First Title
    /// # Second Title
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Title
    /// ## Section One
    /// ## Section Two
    /// ```
    pub NoMultipleTopLevelHeadings {
        version: "next",
        name: "noMultipleTopLevelHeadings",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct DuplicateTopLevel {
    range: TextRange,
}

impl Rule for NoMultipleTopLevelHeadings {
    type Query = Ast<MdDocument>;
    type State = DuplicateTopLevel;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let mut signals = Vec::new();
        let mut found_top_level = false;

        for node in document.syntax().descendants() {
            if let Some(header) = MdHeader::cast_ref(&node) {
                let level = header.before().len();
                if level == 1 {
                    if found_top_level {
                        signals.push(DuplicateTopLevel {
                            range: header.syntax().text_trimmed_range(),
                        });
                    }
                    found_top_level = true;
                }
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
                    "Multiple top-level headings found."
                },
            )
            .note(markup! {
                "A document should contain only one top-level heading."
            }),
        )
    }
}
