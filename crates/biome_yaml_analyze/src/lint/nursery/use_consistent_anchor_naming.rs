use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::TextRange;
use biome_rule_options::use_consistent_anchor_naming::{NamingConvention, UseConsistentAnchorNamingOptions};
use biome_yaml_semantic::semantic_model;
use biome_yaml_syntax::YamlRoot;

declare_lint_rule! {
    /// Enforce a consistent naming convention for YAML anchors.
    ///
    /// By default, this rule enforces camelCase naming for anchor names.
    /// Consistent anchor naming improves readability and makes it easier
    /// to identify and trace anchor/alias relationships.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// defaults: &my_defaults
    ///   timeout: 30
    /// ```
    ///
    /// ```yaml,expect_diagnostic
    /// defaults: &MY-DEFAULTS
    ///   timeout: 30
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// defaults: &myDefaults
    ///   timeout: 30
    /// production:
    ///   <<: *myDefaults
    /// ```
    pub UseConsistentAnchorNaming {
        version: "next",
        name: "useConsistentAnchorNaming",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

fn matches_convention(s: &str, convention: &NamingConvention) -> bool {
    if s.is_empty() {
        return true;
    }
    match convention {
        NamingConvention::CamelCase => {
            let first = s.chars().next().unwrap();
            (first.is_ascii_lowercase() || first.is_ascii_digit())
                && !s.contains('_')
                && !s.contains('-')
        }
        NamingConvention::SnakeCase => {
            s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        }
        NamingConvention::KebabCase => {
            s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        }
        NamingConvention::PascalCase => {
            let first = s.chars().next().unwrap();
            first.is_ascii_uppercase() && !s.contains('_') && !s.contains('-')
        }
    }
}

pub struct BadAnchorNameState {
    name: String,
    range: TextRange,
}

impl Rule for UseConsistentAnchorNaming {
    type Query = Ast<YamlRoot>;
    type State = BadAnchorNameState;
    type Signals = Box<[Self::State]>;
    type Options = UseConsistentAnchorNamingOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let convention = ctx.options().convention();
        let model = semantic_model(root);

        model
            .all_anchors()
            .filter(|anchor| !matches_convention(anchor.name(), convention))
            .map(|anchor| BadAnchorNameState {
                name: anchor.name().to_string(),
                range: anchor.range(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "The anchor name "<Emphasis>{&state.name}</Emphasis>" does not follow the expected naming convention."
                },
            )
            .note(markup! {
                "Use the configured naming convention for anchor names."
            }),
        )
    }
}
