use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, Direction, TextRange, TextSize, TriviaPieceKind};
use biome_yaml_syntax::{YamlLanguage, YamlRoot};

declare_lint_rule! {
    /// Enforce a space after the `#` character in comments.
    ///
    /// Comments that start immediately after `#` without a space are harder to
    /// read. This rule requires at least one space between `#` and the comment
    /// text.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// #bad comment
    /// key: value
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// # good comment
    /// key: value
    /// ```
    pub UseCommentSpacing {
        version: "next",
        name: "useCommentSpacing",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

impl Rule for UseCommentSpacing {
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
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix('#') {
                // Allow empty comments (#\n) and shebangs (#!)
                // Flag comments where # is not followed by a space
                if !rest.is_empty() && !rest.starts_with(' ') && !rest.starts_with('!') {
                    // Find the position of # in the original line
                    let leading_spaces = line.len() - trimmed.len();
                    let hash_pos = offset + leading_spaces as u32;
                    let start = TextSize::from(hash_pos);
                    let end = TextSize::from(hash_pos + 1 + rest.len() as u32);
                    violations.push(TextRange::new(start, end));
                }
            }
            // Also check for inline comments
            if !trimmed.starts_with('#') {
                // Look for inline comments: content followed by # comment
                if let Some(hash_idx) = find_inline_comment(line) {
                    let rest = &line[hash_idx + 1..];
                    if !rest.is_empty() && !rest.starts_with(' ') {
                        let start = TextSize::from(offset + hash_idx as u32);
                        let end = TextSize::from(offset + hash_idx as u32 + 1 + rest.len() as u32);
                        violations.push(TextRange::new(start, end));
                    }
                }
            }
            offset += line.len() as u32 + 1; // +1 for \n
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state,
                markup! {
                    "Comment should have a space after the '#' character."
                },
            )
            .note(markup! {
                "Add a space between '#' and the comment text for readability."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        // Comments are trivia in YAML, so covering_element() won't return them.
        // We need to find the token whose trivia contains the comment at `state`.
        for token in root.syntax().descendants_tokens(Direction::Next) {
            // Check leading trivia
            let mut found_in_leading = false;
            for piece in token.leading_trivia().pieces() {
                if piece.kind() == TriviaPieceKind::SingleLineComment
                    && piece.text_range().start() <= state.start()
                    && piece.text_range().end() >= state.end()
                {
                    found_in_leading = true;
                    break;
                }
            }

            if found_in_leading {
                let new_leading: Vec<(TriviaPieceKind, String)> = token
                    .leading_trivia()
                    .pieces()
                    .map(|piece| {
                        if piece.kind() == TriviaPieceKind::SingleLineComment
                            && piece.text_range().start() <= state.start()
                            && piece.text_range().end() >= state.end()
                        {
                            (piece.kind(), fix_comment_spacing(piece.text()))
                        } else {
                            (piece.kind(), piece.text().to_string())
                        }
                    })
                    .collect();

                let trivia_refs: Vec<(TriviaPieceKind, &str)> = new_leading
                    .iter()
                    .map(|(k, s)| (*k, s.as_str()))
                    .collect();
                let new_token = token.with_leading_trivia(trivia_refs.iter().copied());
                mutation.replace_token_discard_trivia(token, new_token);

                return Some(RuleAction::new(
                    ctx.metadata().action_category(ctx.category(), ctx.group()),
                    ctx.metadata().applicability(),
                    markup! { "Add space after '#'." }.to_owned(),
                    mutation,
                ));
            }

            // Check trailing trivia
            let mut found_in_trailing = false;
            for piece in token.trailing_trivia().pieces() {
                if piece.kind() == TriviaPieceKind::SingleLineComment
                    && piece.text_range().start() <= state.start()
                    && piece.text_range().end() >= state.end()
                {
                    found_in_trailing = true;
                    break;
                }
            }

            if found_in_trailing {
                let new_trailing: Vec<(TriviaPieceKind, String)> = token
                    .trailing_trivia()
                    .pieces()
                    .map(|piece| {
                        if piece.kind() == TriviaPieceKind::SingleLineComment
                            && piece.text_range().start() <= state.start()
                            && piece.text_range().end() >= state.end()
                        {
                            (piece.kind(), fix_comment_spacing(piece.text()))
                        } else {
                            (piece.kind(), piece.text().to_string())
                        }
                    })
                    .collect();

                let trivia_refs: Vec<(TriviaPieceKind, &str)> = new_trailing
                    .iter()
                    .map(|(k, s)| (*k, s.as_str()))
                    .collect();
                let new_token = token.with_trailing_trivia(trivia_refs.iter().copied());
                mutation.replace_token_discard_trivia(token, new_token);

                return Some(RuleAction::new(
                    ctx.metadata().action_category(ctx.category(), ctx.group()),
                    ctx.metadata().applicability(),
                    markup! { "Add space after '#'." }.to_owned(),
                    mutation,
                ));
            }
        }

        None
    }
}

/// Insert a space after `#` in a comment trivia piece if missing.
fn fix_comment_spacing(text: &str) -> String {
    if let Some(rest) = text.strip_prefix('#') {
        if !rest.is_empty() && !rest.starts_with(' ') && !rest.starts_with('!') {
            return format!("# {rest}");
        }
    }
    text.to_string()
}

/// Find the position of an inline comment `#` that is not inside a string.
/// Returns None if no inline comment is found.
fn find_inline_comment(line: &str) -> Option<usize> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for (i, c) in line.char_indices() {
        match c {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '#' if !in_single_quote && !in_double_quote => {
                // Must be preceded by whitespace to be a comment
                if i > 0 && line.as_bytes()[i - 1] == b' ' {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}
