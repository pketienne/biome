use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt};
use biome_rule_options::use_consistent_quote_style::{
    PreferredQuote, UseConsistentQuoteStyleOptions,
};
use biome_yaml_syntax::YamlLanguage;

declare_lint_rule! {
    /// Enforce the consistent use of a preferred quote style for strings.
    ///
    /// Using a consistent quote style across a YAML file improves readability.
    /// By default, double quotes are preferred because they support escape
    /// sequences and are more widely compatible.
    ///
    /// Set the `preferredQuote` option to `"single"` to enforce single quotes
    /// instead.
    ///
    /// Single-quoted strings that contain characters requiring escapes in
    /// double quotes (like backslashes) are not flagged when preferring double
    /// quotes. Double-quoted strings that contain backslash escape sequences
    /// (like `\n`, `\t`) are not flagged when preferring single quotes.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// key: 'single quoted value'
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// key: "double quoted value"
    /// ```
    pub UseConsistentQuoteStyle {
        version: "next",
        name: "useConsistentQuoteStyle",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

impl Rule for UseConsistentQuoteStyle {
    type Query = Ast<biome_yaml_syntax::YamlRoot>;
    type State = biome_rowan::TextRange;
    type Signals = Box<[Self::State]>;
    type Options = UseConsistentQuoteStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let preferred = ctx.options().preferred_quote();
        let mut violations = Vec::new();

        for node in root.syntax().descendants() {
            match preferred {
                PreferredQuote::Double => {
                    if biome_yaml_syntax::YamlSingleQuotedScalar::can_cast(node.kind()) {
                        if let Some(scalar) =
                            biome_yaml_syntax::YamlSingleQuotedScalar::cast(node)
                        {
                            if let Ok(token) = scalar.value_token() {
                                let text = token.text_trimmed();
                                // Don't flag single-quoted strings that contain backslashes,
                                // since switching to double quotes would change their meaning
                                if !text.contains('\\') {
                                    violations.push(scalar.syntax().text_trimmed_range());
                                }
                            }
                        }
                    }
                }
                PreferredQuote::Single => {
                    if biome_yaml_syntax::YamlDoubleQuotedScalar::can_cast(node.kind()) {
                        if let Some(scalar) =
                            biome_yaml_syntax::YamlDoubleQuotedScalar::cast(node)
                        {
                            if let Ok(token) = scalar.value_token() {
                                let text = token.text_trimmed();
                                let inner = &text[1..text.len().saturating_sub(1)];
                                // Skip if content has backslash escape sequences
                                // that cannot be represented in single quotes
                                if !inner.contains('\\') {
                                    violations.push(scalar.syntax().text_trimmed_range());
                                }
                            }
                        }
                    }
                }
            }
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let preferred = ctx.options().preferred_quote();
        let (msg, note) = match preferred {
            PreferredQuote::Double => (
                "Use double quotes instead of single quotes.",
                "Double quotes are preferred for consistency and support escape sequences.",
            ),
            PreferredQuote::Single => (
                "Use single quotes instead of double quotes.",
                "Single quotes are preferred for consistency.",
            ),
        };
        Some(
            RuleDiagnostic::new(rule_category!(), state, markup! { {msg} })
                .note(markup! { {note} }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();
        let preferred = ctx.options().preferred_quote();

        // Use covering_element to find the node at the state range directly
        let element = root.syntax().covering_element(*state);
        let node = match element {
            biome_rowan::NodeOrToken::Node(n) => n,
            biome_rowan::NodeOrToken::Token(t) => t.parent()?,
        };

        // Walk up to find the scalar node if we landed on a child
        let scalar_node = node
            .ancestors()
            .find(|n| {
                matches!(
                    n.kind(),
                    biome_yaml_syntax::YamlSyntaxKind::YAML_SINGLE_QUOTED_SCALAR
                        | biome_yaml_syntax::YamlSyntaxKind::YAML_DOUBLE_QUOTED_SCALAR
                )
            })
            .or_else(|| {
                // The node itself might be the scalar
                if matches!(
                    node.kind(),
                    biome_yaml_syntax::YamlSyntaxKind::YAML_SINGLE_QUOTED_SCALAR
                        | biome_yaml_syntax::YamlSyntaxKind::YAML_DOUBLE_QUOTED_SCALAR
                ) {
                    Some(node.clone())
                } else {
                    None
                }
            })?;

        match preferred {
            PreferredQuote::Double => {
                let scalar = biome_yaml_syntax::YamlSingleQuotedScalar::cast(scalar_node)?;
                let token = scalar.value_token().ok()?;
                let text = token.text_trimmed();

                // Convert 'content' to "content"
                let inner = &text[1..text.len() - 1];
                // In single-quoted YAML, '' is an escaped single quote
                let inner = inner.replace("''", "'");
                // Escape any double quotes in content
                let inner = inner.replace('"', "\\\"");
                let new_text = format!("\"{inner}\"");

                let new_token = biome_yaml_syntax::YamlSyntaxToken::new_detached(
                    token.kind(),
                    &new_text,
                    [],
                    [],
                );
                mutation.replace_token_transfer_trivia(token, new_token);

                Some(RuleAction::new(
                    ctx.metadata().action_category(ctx.category(), ctx.group()),
                    ctx.metadata().applicability(),
                    markup! { "Convert to double quotes." }.to_owned(),
                    mutation,
                ))
            }
            PreferredQuote::Single => {
                let scalar = biome_yaml_syntax::YamlDoubleQuotedScalar::cast(scalar_node)?;
                let token = scalar.value_token().ok()?;
                let text = token.text_trimmed();

                // Convert "content" to 'content'
                let inner = &text[1..text.len() - 1];
                // Unescape double-quote escapes
                let inner = inner.replace("\\\"", "\"");
                // Escape single quotes for single-quoted YAML ('' represents ')
                let inner = inner.replace('\'', "''");
                let new_text = format!("'{inner}'");

                let new_token = biome_yaml_syntax::YamlSyntaxToken::new_detached(
                    token.kind(),
                    &new_text,
                    [],
                    [],
                );
                mutation.replace_token_transfer_trivia(token, new_token);

                Some(RuleAction::new(
                    ctx.metadata().action_category(ctx.category(), ctx.group()),
                    ctx.metadata().applicability(),
                    markup! { "Convert to single quotes." }.to_owned(),
                    mutation,
                ))
            }
        }
    }
}
