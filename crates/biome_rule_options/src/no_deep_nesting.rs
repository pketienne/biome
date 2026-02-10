use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoDeepNestingOptions {
    /// The maximum allowed nesting depth for mappings and sequences.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub max_depth: Option<u16>,
}

impl NoDeepNestingOptions {
    pub const DEFAULT_MAX_DEPTH: u16 = 4;

    /// Returns [`Self::max_depth`] if set, otherwise [`Self::DEFAULT_MAX_DEPTH`].
    pub fn max_depth(&self) -> u16 {
        self.max_depth.unwrap_or(Self::DEFAULT_MAX_DEPTH)
    }
}
