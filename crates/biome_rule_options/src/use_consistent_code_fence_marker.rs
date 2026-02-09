use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseConsistentCodeFenceMarkerOptions {
    /// The marker to use for fenced code blocks (default: "backtick"). Allowed values: "backtick" or "tilde".
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub marker: Option<String>,
}

impl UseConsistentCodeFenceMarkerOptions {
    pub const DEFAULT_MARKER: &'static str = "backtick";

    pub fn marker(&self) -> &str {
        self.marker.as_deref().unwrap_or(Self::DEFAULT_MARKER)
    }
}
