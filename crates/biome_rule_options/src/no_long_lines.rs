use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoLongLinesOptions {
    /// Maximum line length (default: 80).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub max_length: Option<u32>,

    /// Whether to allow long lines inside fenced code blocks (default: true).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub allow_in_code_blocks: Option<bool>,

    /// Whether to allow long lines that contain URLs (default: true).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub allow_urls: Option<bool>,
}

impl NoLongLinesOptions {
    pub const DEFAULT_MAX_LENGTH: u32 = 80;
    pub const DEFAULT_ALLOW_IN_CODE_BLOCKS: bool = true;
    pub const DEFAULT_ALLOW_URLS: bool = true;

    pub fn max_length(&self) -> u32 {
        self.max_length.unwrap_or(Self::DEFAULT_MAX_LENGTH)
    }

    pub fn allow_in_code_blocks(&self) -> bool {
        self.allow_in_code_blocks
            .unwrap_or(Self::DEFAULT_ALLOW_IN_CODE_BLOCKS)
    }

    pub fn allow_urls(&self) -> bool {
        self.allow_urls.unwrap_or(Self::DEFAULT_ALLOW_URLS)
    }
}
