use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseFirstLineHeadingOptions {
    /// The required heading level for the first heading (default: 1).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub level: Option<u8>,
}

impl UseFirstLineHeadingOptions {
    pub const DEFAULT_LEVEL: u8 = 1;

    pub fn level(&self) -> u8 {
        self.level.unwrap_or(Self::DEFAULT_LEVEL)
    }
}
