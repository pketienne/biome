use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoLongHeadingsOptions {
    /// The maximum allowed length for headings (default: 60).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
}

impl NoLongHeadingsOptions {
    pub const DEFAULT_MAX_LENGTH: u32 = 60;

    pub fn max_length(&self) -> u32 {
        self.max_length.unwrap_or(Self::DEFAULT_MAX_LENGTH)
    }
}
