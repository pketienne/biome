use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize, TriviaPieceKind};
use biome_yaml_syntax::{YamlLanguage, YamlRoot};

declare_lint_rule! {
    /// Disallow more than one consecutive blank line in YAML files.
    ///
    /// Multiple consecutive blank lines add unnecessary vertical space and reduce
    /// readability. This rule enforces a maximum of one blank line between content.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// key: value
    ///
    ///
    /// other: data
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// key: value
    ///
    /// other: data
    /// ```
    pub NoConsecutiveBlankLines {
        version: "next",
        name: "noConsecutiveBlankLines",
        language: "yaml",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

impl Rule for NoConsecutiveBlankLines {
    type Query = Ast<YamlRoot>;
    type State = TextRange;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let text = root.syntax().to_string();
        let mut violations = Vec::new();
        let mut consecutive_newlines = 0u32;
        let mut blank_start = 0u32;
        let mut offset = 0u32;

        for line in text.split('\n') {
            let is_blank = line.trim().is_empty();
            if is_blank {
                consecutive_newlines += 1;
                if consecutive_newlines == 2 {
                    // Mark the start of the extra blank lines
                    blank_start = offset;
                }
            } else {
                if consecutive_newlines > 1 {
                    // We had multiple consecutive blank lines
                    // Report from the second blank line to the current position
                    let start = TextSize::from(blank_start);
                    let end = TextSize::from(offset);
                    violations.push(TextRange::new(start, end));
                }
                consecutive_newlines = 0;
            }
            offset += line.len() as u32 + 1; // +1 for \n
        }

        // Handle trailing consecutive blank lines at end of file
        if consecutive_newlines > 1 {
            let start = TextSize::from(blank_start);
            let end = TextSize::from((text.len() as u32).saturating_sub(1));
            if start < end {
                violations.push(TextRange::new(start, end));
            }
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state,
                markup! {
                    "Multiple consecutive blank lines are not allowed."
                },
            )
            .note(markup! {
                "Remove the extra blank lines. At most one blank line is allowed between content."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        // Find the token that covers the start of the blank line range
        let element = root
            .syntax()
            .covering_element(TextRange::new(state.start(), state.start()));
        let token = match element {
            biome_rowan::NodeOrToken::Token(t) => t,
            biome_rowan::NodeOrToken::Node(n) => n.last_token()?,
        };

        // Rebuild leading trivia of the token after the blank lines,
        // collapsing consecutive newlines to at most 2 (one blank line)
        let mut new_leading_pieces: Vec<(TriviaPieceKind, String)> = Vec::new();
        let mut consecutive_newlines = 0u32;

        for piece in token.leading_trivia().pieces() {
            if piece.kind() == TriviaPieceKind::Newline {
                consecutive_newlines += 1;
                if consecutive_newlines <= 2 {
                    new_leading_pieces.push((piece.kind(), piece.text().to_string()));
                }
            } else if piece.kind() == TriviaPieceKind::Whitespace
                && consecutive_newlines > 1
                && new_leading_pieces
                    .last()
                    .is_some_and(|(k, _)| *k == TriviaPieceKind::Newline)
            {
                // Skip whitespace on extra blank lines
                if consecutive_newlines <= 2 {
                    new_leading_pieces.push((piece.kind(), piece.text().to_string()));
                }
            } else {
                consecutive_newlines = 0;
                new_leading_pieces.push((piece.kind(), piece.text().to_string()));
            }
        }

        let trivia_refs: Vec<(TriviaPieceKind, &str)> = new_leading_pieces
            .iter()
            .map(|(k, s)| (*k, s.as_str()))
            .collect();

        let new_token = token.with_leading_trivia(trivia_refs.iter().copied());
        mutation.replace_token_discard_trivia(token, new_token);

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove extra blank lines." }.to_owned(),
            mutation,
        ))
    }
}
