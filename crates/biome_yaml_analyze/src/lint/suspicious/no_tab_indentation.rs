use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};
use biome_yaml_syntax::{YamlLanguage, YamlRoot, YamlSyntaxToken};

declare_lint_rule! {
    /// Disallow tabs for indentation in YAML files.
    ///
    /// The YAML specification forbids tabs for indentation. While some parsers
    /// may accept tabs, they can lead to inconsistent behavior across different
    /// YAML processors.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// key:
    /// 	value: indented with tab
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// key:
    ///   value: indented with spaces
    /// ```
    pub NoTabIndentation {
        version: "next",
        name: "noTabIndentation",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
        fix_kind: FixKind::Safe,
    }
}

impl Rule for NoTabIndentation {
    type Query = Ast<YamlRoot>;
    type State = TextRange;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let text = root.syntax().to_string();
        let mut violations = Vec::new();
        let mut offset = 0u32;

        for line in text.split('\n') {
            // Check for leading tabs (indentation)
            let mut col = 0usize;
            for ch in line.chars() {
                if ch == '\t' {
                    let start = TextSize::from(offset + col as u32);
                    violations.push(TextRange::new(start, start + TextSize::from(1)));
                } else if ch == ' ' {
                    // Still in indentation
                } else {
                    break;
                }
                col += ch.len_utf8();
            }
            offset += line.len() as u32 + 1; // +1 for the \n
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state,
                markup! {
                    "Tabs are not allowed for indentation in YAML."
                },
            )
            .note(markup! {
                "The YAML specification forbids tabs for indentation. Use spaces instead."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        // Find the token covering this tab
        let token = root
            .syntax()
            .covering_element(TextRange::new(state.start(), state.start()))
            .into_token()?;

        let text = token.text().to_string();
        let new_text = text.replace('\t', "  ");
        if new_text == text {
            return None;
        }

        let new_token = YamlSyntaxToken::new_detached(token.kind(), &new_text, [], []);
        mutation.replace_token_transfer_trivia(token, new_token);

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Replace tab with spaces." }.to_owned(),
            mutation,
        ))
    }
}
