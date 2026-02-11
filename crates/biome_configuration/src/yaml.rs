use crate::bool::Bool;
use biome_deserialize_macros::{Deserializable, Merge};
use biome_formatter::{IndentStyle, IndentWidth, LineEnding, LineWidth};
use bpaf::Bpaf;
use serde::{Deserialize, Serialize};

/// Options applied to YAML files
#[derive(
    Bpaf, Clone, Debug, Default, Deserializable, Deserialize, Eq, Merge, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct YamlConfiguration {
    /// Parsing options
    #[bpaf(external(yaml_parser_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parser: Option<YamlParserConfiguration>,

    /// Formatting options
    #[bpaf(external(yaml_formatter_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatter: Option<YamlFormatterConfiguration>,

    /// Linting options
    #[bpaf(external(yaml_linter_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linter: Option<YamlLinterConfiguration>,

    /// Assist options
    #[bpaf(external(yaml_assist_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assist: Option<YamlAssistConfiguration>,
}

/// Options that changes how the YAML parser behaves
#[derive(
    Bpaf, Clone, Debug, Default, Deserializable, Deserialize, Eq, Merge, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct YamlParserConfiguration {}

pub type YamlFormatterEnabled = Bool<true>;

/// Options that changes how the YAML formatter behaves
#[derive(
    Bpaf, Clone, Debug, Default, Deserializable, Deserialize, Eq, Merge, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct YamlFormatterConfiguration {
    /// Control the formatter for YAML files.
    #[bpaf(long("yaml-formatter-enabled"), argument("true|false"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<YamlFormatterEnabled>,

    /// The indent style applied to YAML files.
    #[bpaf(long("yaml-formatter-indent-style"), argument("tab|space"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_style: Option<IndentStyle>,

    /// The size of the indentation applied to YAML files. Default to 2.
    #[bpaf(long("yaml-formatter-indent-width"), argument("NUMBER"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_width: Option<IndentWidth>,

    /// The type of line ending applied to YAML files. `auto` uses CRLF on Windows and LF on other platforms.
    #[bpaf(long("yaml-formatter-line-ending"), argument("lf|crlf|cr|auto"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_ending: Option<LineEnding>,

    /// What's the max width of a line applied to YAML files. Defaults to 80.
    #[bpaf(long("yaml-formatter-line-width"), argument("NUMBER"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_width: Option<LineWidth>,
}

impl YamlFormatterConfiguration {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or_default().into()
    }
}

pub type YamlLinterEnabled = Bool<true>;

/// Linter options specific to the YAML linter
#[derive(
    Bpaf, Clone, Debug, Default, Deserializable, Deserialize, Eq, Merge, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct YamlLinterConfiguration {
    /// Control the linter for YAML files.
    #[bpaf(long("yaml-linter-enabled"), argument("true|false"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<YamlLinterEnabled>,
}

impl YamlLinterConfiguration {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or_default().into()
    }
}

pub type YamlAssistEnabled = Bool<true>;

/// Assist options specific to the YAML assist
#[derive(
    Bpaf, Clone, Debug, Default, Deserializable, Deserialize, Eq, Merge, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct YamlAssistConfiguration {
    /// Control the assist for YAML files.
    #[bpaf(long("yaml-assist-enabled"), argument("true|false"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<YamlAssistEnabled>,
}
