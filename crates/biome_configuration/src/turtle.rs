use crate::bool::Bool;
use biome_deserialize_macros::{Deserializable, Merge};
use biome_formatter::{IndentStyle, IndentWidth, LineEnding, LineWidth};
use bpaf::Bpaf;
use serde::{Deserialize, Serialize};

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
}

#[test]
fn default_turtle_linter() {
    let turtle_configuration = TurtleLinterConfiguration::default();

    assert!(turtle_configuration.is_enabled());
}
