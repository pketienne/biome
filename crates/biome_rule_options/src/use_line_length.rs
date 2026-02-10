use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseLineLengthOptions {
    /// The maximum allowed line length.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub max_length: Option<u16>,
}

impl UseLineLengthOptions {
    pub const DEFAULT_MAX_LENGTH: u16 = 120;

    /// Returns [`Self::max_length`] if set, otherwise [`Self::DEFAULT_MAX_LENGTH`].
    pub fn max_length(&self) -> u16 {
        self.max_length.unwrap_or(Self::DEFAULT_MAX_LENGTH)
    }
}
