use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdHeader;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Require a space before the closing hashes in closed ATX headings.
    ///
    /// If a heading uses closing hashes (e.g., `## Heading ##`), there
    /// should be a space before the closing hashes.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ## Heading##
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ## Heading ##
    /// ```
    pub NoMissingSpaceClosedAtxHeading {
        version: "next",
        name: "noMissingSpaceClosedAtxHeading",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct MissingSpaceClosedAtx {
    range: TextRange,
    closing_hashes: String,
}

impl Rule for NoMissingSpaceClosedAtxHeading {
    type Query = Ast<MdHeader>;
    type State = MissingSpaceClosedAtx;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let header = ctx.query();
        let text = header.syntax().text_trimmed().to_string();
        let trimmed = text.trim_end();

        if !trimmed.starts_with('#') || !trimmed.ends_with('#') {
            return None;
        }

        let content = trimmed.trim_end_matches('#');
        if content.is_empty() {
            return None;
        }

        if content.ends_with(' ') {
            return None;
        }

        let closing_hash_count = trimmed.len() - content.len();
        let base = header.syntax().text_trimmed_range().start();
        let range = TextRange::new(
            base + TextSize::from(content.len() as u32),
            base + TextSize::from(trimmed.len() as u32),
        );

        Some(MissingSpaceClosedAtx {
            range,
            closing_hashes: "#".repeat(closing_hash_count),
        })
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let replacement = format!(" {}", state.closing_hashes);
        let mutation = make_text_replacement(&ctx.root(), state.range, &replacement)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Add a space before the closing hashes." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "Missing space before closing hashes in ATX heading."
                },
            )
            .note(markup! {
                "Add a space before the closing hash characters."
            }),
        )
    }
}
