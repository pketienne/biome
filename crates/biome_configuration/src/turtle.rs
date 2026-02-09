use crate::bool::Bool;
use biome_deserialize_macros::{Deserializable, Merge};
use biome_formatter::{IndentStyle, IndentWidth, LineEnding, LineWidth, QuoteStyle};
use bpaf::Bpaf;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The style of directive declarations in Turtle files.
#[derive(Clone, Copy, Debug, Default, Deserializable, Deserialize, Eq, Hash, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TurtleDirectiveStyle {
    /// Use Turtle-native `@prefix`/`@base` with trailing `.`
    #[default]
    Turtle,
    /// Use SPARQL-compatible `PREFIX`/`BASE` without trailing `.`
    Sparql,
}

impl fmt::Display for TurtleDirectiveStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Turtle => write!(f, "Turtle (@prefix/@base)"),
            Self::Sparql => write!(f, "SPARQL (PREFIX/BASE)"),
        }
    }
}

impl std::str::FromStr for TurtleDirectiveStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "turtle" | "Turtle" => Ok(Self::Turtle),
            "sparql" | "Sparql" | "SPARQL" => Ok(Self::Sparql),
            _ => Err(std::format!(
                "Unknown directive style '{}'. Expected 'turtle' or 'sparql'.",
                s
            )),
        }
    }
}

/// Options applied to Turtle (RDF) files
#[derive(
    Bpaf, Clone, Default, Debug, Deserializable, Deserialize, Eq, Merge, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct TurtleConfiguration {
    /// Turtle formatter options
    #[bpaf(external(turtle_formatter_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatter: Option<TurtleFormatterConfiguration>,

    /// Turtle linter options
    #[bpaf(external(turtle_linter_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linter: Option<TurtleLinterConfiguration>,

    /// Turtle assist options
    #[bpaf(external(turtle_assist_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assist: Option<TurtleAssistConfiguration>,
}

pub type TurtleFormatterEnabled = Bool<true>;

/// Options that changes how the Turtle formatter behaves
#[derive(
    Bpaf, Clone, Debug, Default, Deserializable, Deserialize, Eq, Merge, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct TurtleFormatterConfiguration {
    /// Control the formatter for Turtle files.
    #[bpaf(long("turtle-formatter-enabled"), argument("true|false"))]
    pub enabled: Option<TurtleFormatterEnabled>,

    /// The indent style applied to Turtle files.
    #[bpaf(long("turtle-formatter-indent-style"), argument("tab|space"))]
    pub indent_style: Option<IndentStyle>,

    /// The size of the indentation applied to Turtle files. Default to 2.
    #[bpaf(long("turtle-formatter-indent-width"), argument("NUMBER"))]
    pub indent_width: Option<IndentWidth>,

    /// The type of line ending applied to Turtle files.
    #[bpaf(long("turtle-formatter-line-ending"), argument("lf|crlf|cr|auto"))]
    pub line_ending: Option<LineEnding>,

    /// What's the max width of a line applied to Turtle files. Defaults to 80.
    #[bpaf(long("turtle-formatter-line-width"), argument("NUMBER"))]
    pub line_width: Option<LineWidth>,

    /// The type of quotes used in Turtle string literals. Defaults to double.
    #[bpaf(long("turtle-formatter-quote-style"), argument("double|single"))]
    pub quote_style: Option<QuoteStyle>,

    /// Whether the first predicate should be on a new line after the subject. Defaults to true.
    #[bpaf(long("turtle-formatter-first-predicate-in-new-line"), argument("true|false"))]
    pub first_predicate_in_new_line: Option<bool>,

    /// The directive style for prefix and base declarations. Defaults to "turtle".
    #[bpaf(long("turtle-formatter-directive-style"), argument("turtle|sparql"))]
    pub directive_style: Option<TurtleDirectiveStyle>,
}

impl TurtleFormatterConfiguration {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or_default().into()
    }
}

pub type TurtleLinterEnabled = Bool<true>;

/// Options that change how the Turtle linter behaves.
#[derive(
    Bpaf, Clone, Debug, Default, Deserializable, Deserialize, Eq, Merge, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct TurtleLinterConfiguration {
    /// Control the linter for Turtle files.
    #[bpaf(long("turtle-linter-enabled"), argument("true|false"))]
    pub enabled: Option<TurtleLinterEnabled>,
}

impl TurtleLinterConfiguration {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or_default().into()
    }
}

pub type TurtleAssistEnabled = Bool<false>;

/// Options that change how the Turtle assist behaves
#[derive(
    Bpaf, Clone, Debug, Default, Deserializable, Deserialize, Eq, Merge, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct TurtleAssistConfiguration {
    /// Control the assist for Turtle files.
    #[bpaf(long("turtle-assist-enabled"), argument("true|false"))]
    pub enabled: Option<TurtleAssistEnabled>,
}

#[test]
fn default_turtle_formatter() {
    let turtle_configuration = TurtleFormatterConfiguration::default();

    assert!(turtle_configuration.is_enabled());
    assert_eq!(turtle_configuration.indent_style, None);
    assert_eq!(turtle_configuration.indent_width, None);
    assert_eq!(turtle_configuration.line_ending, None);
    assert_eq!(turtle_configuration.line_width, None);
    assert_eq!(turtle_configuration.quote_style, None);
}

#[test]
fn default_turtle_linter() {
    let turtle_configuration = TurtleLinterConfiguration::default();

    assert!(turtle_configuration.is_enabled());
}
