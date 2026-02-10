use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

/// The naming convention to enforce.
#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum NamingConvention {
    /// camelCase (e.g., `myAnchor`)
    #[default]
    CamelCase,
    /// snake_case (e.g., `my_anchor`)
    SnakeCase,
    /// kebab-case (e.g., `my-anchor`)
    KebabCase,
    /// PascalCase (e.g., `MyAnchor`)
    PascalCase,
}

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseConsistentAnchorNamingOptions {
    /// The naming convention to enforce for anchor names.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub convention: Option<NamingConvention>,
}

impl UseConsistentAnchorNamingOptions {
    /// Returns the configured convention or the default (camelCase).
    pub fn convention(&self) -> &NamingConvention {
        self.convention.as_ref().unwrap_or(&NamingConvention::CamelCase)
    }
}
