use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_turtle_syntax::TurtleRdfLiteral;
use biome_rowan::TextRange;

declare_lint_rule! {
    /// Disallow invalid language tags on RDF literals.
    ///
    /// Language tags in Turtle (e.g. `"hello"@en`) must conform to BCP47.
    /// A valid tag consists of subtags of 1-8 alphanumeric characters
    /// separated by hyphens.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// ex:s ex:label "hello"@toolongsubtag .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    /// ex:s ex:label "hello"@en .
    /// ```
    ///
    pub NoInvalidLanguageTag {
        version: "next",
        name: "noInvalidLanguageTag",
        language: "turtle",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct InvalidTag {
    tag: String,
    range: TextRange,
}

fn is_valid_bcp47(tag: &str) -> bool {
    // BCP47 simplified: subtags of 1-8 alphanumeric chars separated by '-'
    if tag.is_empty() {
        return false;
    }
    tag.split('-')
        .all(|subtag| !subtag.is_empty() && subtag.len() <= 8 && subtag.chars().all(|c| c.is_ascii_alphanumeric()))
}

impl Rule for NoInvalidLanguageTag {
    type Query = Ast<TurtleRdfLiteral>;
    type State = InvalidTag;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        let language_token = node.language_token()?;
        let text = language_token.text_trimmed();
        // Language token includes the '@' prefix
        let tag = text.strip_prefix('@').unwrap_or(text);
        if !is_valid_bcp47(tag) {
            Some(InvalidTag {
                tag: text.to_string(),
                range: language_token.text_trimmed_range(),
            })
        } else {
            None
        }
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Invalid language tag '"{ &state.tag }"'."
                },
            )
            .note(markup! {
                "Language tags must conform to BCP47 (e.g. @en, @en-US)."
            }),
        )
    }
}
