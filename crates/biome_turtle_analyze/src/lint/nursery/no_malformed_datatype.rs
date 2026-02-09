use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_turtle_syntax::TurtleRdfLiteral;

declare_lint_rule! {
    /// Disallow literal values that don't conform to their declared XSD datatype.
    ///
    /// When a literal has a datatype annotation like `^^xsd:integer`, the value
    /// should conform to the expected format for that datatype. Malformed values
    /// indicate data quality issues.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    /// @prefix ex: <http://example.org/> .
    /// ex:s ex:count "abc"^^xsd:integer .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    /// @prefix ex: <http://example.org/> .
    /// ex:s ex:count "42"^^xsd:integer .
    /// ```
    ///
    pub NoMalformedDatatype {
        version: "next",
        name: "noMalformedDatatype",
        language: "turtle",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct MalformedDatatype {
    range: TextRange,
    value: String,
    datatype: String,
    expected: &'static str,
}

const XSD_INTEGER: &str = "xsd:integer";
const XSD_INTEGER_FULL: &str = "<http://www.w3.org/2001/XMLSchema#integer>";
const XSD_BOOLEAN: &str = "xsd:boolean";
const XSD_BOOLEAN_FULL: &str = "<http://www.w3.org/2001/XMLSchema#boolean>";
const XSD_DECIMAL: &str = "xsd:decimal";
const XSD_DECIMAL_FULL: &str = "<http://www.w3.org/2001/XMLSchema#decimal>";

impl Rule for NoMalformedDatatype {
    type Query = Ast<TurtleRdfLiteral>;
    type State = MalformedDatatype;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        let datatype = node.datatype()?;
        let datatype_text = datatype.syntax().text_trimmed().to_string();

        // Extract the datatype name after ^^
        let dt_name = datatype_text.strip_prefix("^^")?;

        let string_node = node.value().ok()?;
        let token = string_node.value().ok()?;
        let text = token.text_trimmed();

        // Extract inner value from quotes
        let inner = if text.starts_with('"') && text.ends_with('"') && !text.starts_with("\"\"\"") {
            &text[1..text.len() - 1]
        } else if text.starts_with('\'') && text.ends_with('\'') && !text.starts_with("'''") {
            &text[1..text.len() - 1]
        } else {
            return None;
        };

        match dt_name {
            XSD_INTEGER | XSD_INTEGER_FULL => {
                if !is_valid_integer(inner) {
                    Some(MalformedDatatype {
                        range: node.syntax().text_trimmed_range(),
                        value: inner.to_string(),
                        datatype: dt_name.to_string(),
                        expected: "an integer value (e.g. 42, -7, +0)",
                    })
                } else {
                    None
                }
            }
            XSD_BOOLEAN | XSD_BOOLEAN_FULL => {
                if !is_valid_boolean(inner) {
                    Some(MalformedDatatype {
                        range: node.syntax().text_trimmed_range(),
                        value: inner.to_string(),
                        datatype: dt_name.to_string(),
                        expected: "a boolean value (true, false, 0, or 1)",
                    })
                } else {
                    None
                }
            }
            XSD_DECIMAL | XSD_DECIMAL_FULL => {
                if !is_valid_decimal(inner) {
                    Some(MalformedDatatype {
                        range: node.syntax().text_trimmed_range(),
                        value: inner.to_string(),
                        datatype: dt_name.to_string(),
                        expected: "a decimal value (e.g. 3.14, -0.5, +100.0)",
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Value \""{ &state.value }"\" does not conform to datatype "{ &state.datatype }"."
                },
            )
            .note(markup! {
                "Expected "{ state.expected }"."
            }),
        )
    }
}

fn is_valid_integer(s: &str) -> bool {
    let s = s.strip_prefix(['+', '-']).unwrap_or(s);
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

fn is_valid_boolean(s: &str) -> bool {
    matches!(s, "true" | "false" | "0" | "1")
}

fn is_valid_decimal(s: &str) -> bool {
    let s = s.strip_prefix(['+', '-']).unwrap_or(s);
    if s.is_empty() {
        return false;
    }
    // Must contain a decimal point
    let parts: Vec<&str> = s.splitn(2, '.').collect();
    match parts.len() {
        1 => parts[0].chars().all(|c| c.is_ascii_digit()) && !parts[0].is_empty(),
        2 => {
            let int_part = parts[0];
            let frac_part = parts[1];
            // At least one part must be non-empty
            (!int_part.is_empty() || !frac_part.is_empty())
                && (int_part.is_empty() || int_part.chars().all(|c| c.is_ascii_digit()))
                && (frac_part.is_empty() || frac_part.chars().all(|c| c.is_ascii_digit()))
        }
        _ => false,
    }
}
