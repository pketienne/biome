use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoConsecutiveBlankLinesOptions {
    /// Maximum number of consecutive blank lines allowed (default: 1).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub max_consecutive: Option<u32>,
}

impl NoConsecutiveBlankLinesOptions {
    pub const DEFAULT_MAX_CONSECUTIVE: u32 = 1;

    pub fn max_consecutive(&self) -> u32 {
        self.max_consecutive
            .unwrap_or(Self::DEFAULT_MAX_CONSECUTIVE)
    }
}
