use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};
use biome_yaml_syntax::{YamlLanguage, YamlRoot, YamlSyntaxToken};

declare_lint_rule! {
    /// Require a newline at the end of the file.
    ///
    /// POSIX defines a line as a sequence of characters ending with a newline.
    /// Files without a final newline can cause issues with some tools and
    /// create noisy diffs when content is later appended.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// key: value
    /// other: data
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// key: value
    /// ```
    pub UseFinalNewline {
        version: "next",
        name: "useFinalNewline",
        language: "yaml",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

impl Rule for UseFinalNewline {
    type Query = Ast<YamlRoot>;
    type State = TextRange;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let text = root.syntax().to_string();

        // Empty files are fine
        if text.is_empty() {
            return None;
        }

        if !text.ends_with('\n') {
            let len = text.len() as u32;
            // Point at the last character
            let start = TextSize::from(len.saturating_sub(1));
            let end = TextSize::from(len);
            return Some(TextRange::new(start, end));
        }

        None
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state,
                markup! {
                    "Missing newline at the end of the file."
                },
            )
            .note(markup! {
                "Add a newline at the end of the file."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        // Find the last real token and append a newline to its trailing content
        let eof = root.eof_token().ok()?;
        let new_eof = YamlSyntaxToken::new_detached(
            eof.kind(),
            "\n",
            [biome_rowan::TriviaPiece::new(
                biome_rowan::TriviaPieceKind::Newline,
                1,
            )],
            [],
        );
        mutation.replace_token(eof, new_eof);

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Add a trailing newline." }.to_owned(),
            mutation,
        ))
    }
}
