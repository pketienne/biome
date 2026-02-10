use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};
use biome_yaml_syntax::{YamlLanguage, YamlRoot, YamlSyntaxToken};

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

        // Find the token covering the comment
        let token = root
            .syntax()
            .covering_element(TextRange::new(state.start(), state.start()))
            .into_token()?;

        let text = token.text().to_string();
        // Insert a space after each '#' that is immediately followed by non-space
        let mut result = String::with_capacity(text.len() + 1);
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            result.push(c);
            if c == '#' {
                if let Some(&next) = chars.peek() {
                    if next != ' ' && next != '\n' && next != '!' {
                        result.push(' ');
                    }
                }
            }
        }

        if result == text {
            return None;
        }

        let new_token = YamlSyntaxToken::new_detached(token.kind(), &result, [], []);
        mutation.replace_token_transfer_trivia(token, new_token);

        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Add space after '#'." }.to_owned(),
            mutation,
        ))
    }
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
