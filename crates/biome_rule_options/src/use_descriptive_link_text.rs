use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseDescriptiveLinkTextOptions {
    /// The minimum length of descriptive link text (default: 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_length: Option<u32>,
}

impl UseDescriptiveLinkTextOptions {
    pub const DEFAULT_MINIMUM_LENGTH: u32 = 1;

    pub fn minimum_length(&self) -> u32 {
        self.minimum_length.unwrap_or(Self::DEFAULT_MINIMUM_LENGTH)
    }
}
