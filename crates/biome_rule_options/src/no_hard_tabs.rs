use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoHardTabsOptions {
    /// Whether to allow hard tabs inside fenced code blocks (default: false).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub allow_in_code_blocks: Option<bool>,
}

impl NoHardTabsOptions {
    pub const DEFAULT_ALLOW_IN_CODE_BLOCKS: bool = false;

    pub fn allow_in_code_blocks(&self) -> bool {
        self.allow_in_code_blocks
            .unwrap_or(Self::DEFAULT_ALLOW_IN_CODE_BLOCKS)
    }
}
