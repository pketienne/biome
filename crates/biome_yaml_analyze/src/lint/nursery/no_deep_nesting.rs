use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_rule_options::no_deep_nesting::NoDeepNestingOptions;
use biome_yaml_syntax::{YamlRoot, YamlSyntaxKind, YamlSyntaxNode};

declare_lint_rule! {
    /// Disallow deeply nested YAML structures.
    ///
    /// Deeply nested YAML documents are hard to read and maintain.
    /// This rule flags any mapping or sequence that exceeds a nesting
    /// depth of 4 levels (configurable via the `maxDepth` option).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// a:
    ///   b:
    ///     c:
    ///       d:
    ///         e: too deep
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// a:
    ///   b:
    ///     c:
    ///       d: still ok
    /// ```
    pub NoDeepNesting {
        version: "next",
        name: "noDeepNesting",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct DeepNestingState {
    range: TextRange,
    depth: u16,
}

fn is_nesting_node(kind: YamlSyntaxKind) -> bool {
    matches!(
        kind,
        YamlSyntaxKind::YAML_BLOCK_MAPPING
            | YamlSyntaxKind::YAML_BLOCK_SEQUENCE
            | YamlSyntaxKind::YAML_FLOW_MAPPING
            | YamlSyntaxKind::YAML_FLOW_SEQUENCE
    )
}

fn nesting_depth(node: &YamlSyntaxNode) -> u16 {
    let mut depth = 0u16;
    let mut current = node.parent();
    while let Some(parent) = current {
        if is_nesting_node(parent.kind()) {
            depth += 1;
        }
        current = parent.parent();
    }
    depth
}

impl Rule for NoDeepNesting {
    type Query = Ast<YamlRoot>;
    type State = DeepNestingState;
    type Signals = Box<[Self::State]>;
    type Options = NoDeepNestingOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let max_depth = ctx.options().max_depth();
        let mut violations = Vec::new();

        for node in root.syntax().descendants() {
            if is_nesting_node(node.kind()) {
                let depth = nesting_depth(&node);
                if depth > max_depth {
                    violations.push(DeepNestingState {
                        range: node.text_trimmed_range(),
                        depth,
                    });
                }
            }
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Nesting depth of "{state.depth.to_string()}" exceeds the maximum allowed depth."
                },
            )
            .note(markup! {
                "Deeply nested structures are hard to read. Consider flattening or using anchors/aliases."
            }),
        )
    }
}
