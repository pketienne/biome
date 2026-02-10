use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

use crate::use_consistent_anchor_naming::NamingConvention;

#[derive(Default, Clone, Debug, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseKeyNamingConventionOptions {
    /// The naming convention to enforce for mapping keys.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub convention: Option<NamingConvention>,
}

impl UseKeyNamingConventionOptions {
    /// Returns the configured convention or the default (camelCase).
    pub fn convention(&self) -> &NamingConvention {
        self.convention.as_ref().unwrap_or(&NamingConvention::CamelCase)
    }
}
