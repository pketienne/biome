use crate::bool::Bool;
use biome_deserialize_macros::{Deserializable, Merge};
use bpaf::Bpaf;
use serde::{Deserialize, Serialize};

/// Options applied to YAML files
#[derive(
    Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Bpaf, Deserializable, Merge,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct YamlConfiguration {
    /// YAML formatter options
    #[bpaf(external(yaml_formatter_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatter: Option<YamlFormatterConfiguration>,

    /// YAML linter options
    #[bpaf(external(yaml_linter_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linter: Option<YamlLinterConfiguration>,

    /// YAML assist options
    #[bpaf(external(yaml_assist_configuration), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assist: Option<YamlAssistConfiguration>,
}

pub type YamlFormatterEnabled = Bool<true>;
pub type YamlLinterEnabled = Bool<true>;
pub type YamlAssistEnabled = Bool<true>;

/// Options that changes how the YAML formatter behaves
#[derive(
    Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Bpaf, Deserializable, Merge,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct YamlFormatterConfiguration {
    /// Control the formatter for YAML files.
    #[bpaf(long("yaml-formatter-enabled"), argument("true|false"), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<YamlFormatterEnabled>,
}

/// Options that changes how the YAML linter behaves
#[derive(
    Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Bpaf, Deserializable, Merge,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct YamlLinterConfiguration {
    /// Control the linter for YAML files.
    #[bpaf(long("yaml-linter-enabled"), argument("true|false"), optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<YamlLinterEnabled>,
}

/// Options that changes how the YAML assist behaves
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
