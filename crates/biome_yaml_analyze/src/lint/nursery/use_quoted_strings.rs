use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::AstNode;
use biome_yaml_syntax::{YamlPlainScalar, YamlSyntaxKind};

declare_lint_rule! {
    /// Require string values to be quoted in YAML.
    ///
    /// Unquoted (plain) strings in YAML can be ambiguous. Values like `yes`,
    /// `no`, `true`, `false`, `null`, or numbers may be interpreted as
    /// non-string types. Quoting all strings eliminates this ambiguity.
    ///
    /// This rule flags plain scalars that appear as mapping values. It does
    /// not flag mapping keys, as unquoted keys are standard YAML practice.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// name: John
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// name: "John"
    /// ```
    pub UseQuotedStrings {
        version: "next",
        name: "useQuotedStrings",
        language: "yaml",
        recommended: false,
        severity: Severity::Information,
    }
}

impl Rule for UseQuotedStrings {
    type Query = Ast<YamlPlainScalar>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let scalar = ctx.query();

        // Only flag plain scalars that are mapping values, not keys.
        // A plain scalar used as a mapping value is typically wrapped in a
        // flow node (YamlFlowYamlNode) which is a child of a block/flow map entry's value slot.
        // We check the grandparent to see if this is in a value position.
        let parent = scalar.syntax().parent()?;
        let grandparent = parent.parent()?;

        // Check if the grandparent is a map entry (block or flow)
        match grandparent.kind() {
            YamlSyntaxKind::YAML_BLOCK_MAP_IMPLICIT_ENTRY
            | YamlSyntaxKind::YAML_BLOCK_MAP_EXPLICIT_ENTRY
            | YamlSyntaxKind::YAML_FLOW_MAP_IMPLICIT_ENTRY
            | YamlSyntaxKind::YAML_FLOW_MAP_EXPLICIT_ENTRY => {
                // The parent (flow node) should be in the value slot (not key slot).
                // In implicit entries, slot 0 is key, slot 2 is value.
                // We check if our parent is NOT the first non-token child (key).
                let first_child = grandparent.first_child()?;
                if first_child == parent {
                    // This is the key position — don't flag
                    return None;
                }
                Some(())
            }
            // Also flag plain scalars in sequence entries
            YamlSyntaxKind::YAML_BLOCK_SEQUENCE_ENTRY => Some(()),
            _ => None,
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let scalar = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                scalar.syntax().text_trimmed_range(),
                markup! {
                    "Unquoted string values are not allowed."
                },
            )
            .note(markup! {
                "Wrap the value in quotes to avoid type ambiguity."
            }),
        )
    }
}
