use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize, TriviaPieceKind};
use biome_yaml_syntax::{YamlLanguage, YamlRoot};

declare_lint_rule! {
    /// Disallow trailing whitespace at the end of lines.
    ///
    /// Trailing whitespace is invisible and serves no purpose. It can cause
    /// noise in version control diffs and is considered bad practice.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// key: value
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// key: value
    /// ```
    pub NoTrailingSpaces {
        version: "next",
        name: "noTrailingSpaces",
        language: "yaml",
        recommended: true,
        severity: Severity::Error,
        fix_kind: FixKind::Safe,
    }
}

impl Rule for NoTrailingSpaces {
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
            let trimmed_len = line.trim_end_matches(|c| c == ' ' || c == '\t').len();
            let line_len = line.len();
            if trimmed_len < line_len {
                let start = TextSize::from(offset + trimmed_len as u32);
                let end = TextSize::from(offset + line_len as u32);
                violations.push(TextRange::new(start, end));
            }
            offset += line_len as u32 + 1; // +1 for the \n
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state,
                markup! {
                    "Trailing whitespace is not allowed."
                },
            )
            .note(markup! {
                "Remove the trailing whitespace at the end of the line."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        // Find the token that contains or is adjacent to the trailing whitespace range
        let token = root
            .syntax()
            .covering_element(TextRange::new(state.start(), state.start()));
        let token = match token {
            biome_rowan::NodeOrToken::Token(t) => t,
            biome_rowan::NodeOrToken::Node(n) => n.last_token()?,
        };

        // Rebuild the token's trailing trivia without whitespace before newlines
        let mut new_trailing_pieces: Vec<(TriviaPieceKind, String)> = Vec::new();
        let mut skip_whitespace = true;
        // Process trailing trivia in reverse to strip trailing whitespace before newlines
        let pieces: Vec<_> = token.trailing_trivia().pieces().collect();
        for piece in pieces.iter().rev() {
            if piece.kind() == TriviaPieceKind::Newline {
                skip_whitespace = true;
                new_trailing_pieces.push((piece.kind(), piece.text().to_string()));
            } else if piece.kind() == TriviaPieceKind::Whitespace && skip_whitespace {
                // Skip trailing whitespace before newline
                continue;
            } else {
                skip_whitespace = false;
                new_trailing_pieces.push((piece.kind(), piece.text().to_string()));
            }
        }
        new_trailing_pieces.reverse();

        let trivia_refs: Vec<(TriviaPieceKind, &str)> = new_trailing_pieces
            .iter()
            .map(|(k, s)| (*k, s.as_str()))
            .collect();

        let new_token = token.with_trailing_trivia(trivia_refs.iter().copied());
        mutation.replace_token_discard_trivia(token, new_token);

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove trailing whitespace." }.to_owned(),
            mutation,
        ))
    }
}
