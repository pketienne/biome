use crate::bool::Bool;
use biome_deserialize_macros::{Deserializable, Merge};
use bpaf::Bpaf;
use serde::{Deserialize, Serialize};

/// Options applied to Markdown files
#[derive(
    Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Bpaf, Deserializable, Merge,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MarkdownConfiguration {
    /// Markdown formatter options
    #[bpaf(external(markdown_formatter_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatter: Option<MarkdownFormatterConfiguration>,

    /// Markdown linter options
    #[bpaf(external(markdown_linter_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linter: Option<MarkdownLinterConfiguration>,

    /// Markdown assist options
    #[bpaf(external(markdown_assist_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assist: Option<MarkdownAssistConfiguration>,
}

pub type MarkdownFormatterEnabled = Bool<true>;
pub type MarkdownLinterEnabled = Bool<true>;
pub type MarkdownAssistEnabled = Bool<true>;

/// Options that changes how the Markdown formatter behaves
#[derive(
    Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Bpaf, Deserializable, Merge,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MarkdownFormatterConfiguration {
    /// Control the formatter for Markdown files.
    #[bpaf(long("markdown-formatter-enabled"), argument("true|false"), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<MarkdownFormatterEnabled>,
}

/// Options that changes how the Markdown linter behaves
#[derive(
    Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Bpaf, Deserializable, Merge,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MarkdownLinterConfiguration {
    /// Control the linter for Markdown files.
    #[bpaf(long("markdown-linter-enabled"), argument("true|false"), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<MarkdownLinterEnabled>,
}

/// Options that changes how the Markdown assist behaves
#[derive(
    Bpaf, Clone, Debug, Default, Deserializable, Deserialize, Eq, Merge, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MarkdownAssistConfiguration {
    /// Control the assist for Markdown files.
    #[bpaf(long("markdown-assist-enabled"), argument("true|false"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<MarkdownAssistEnabled>,
}
