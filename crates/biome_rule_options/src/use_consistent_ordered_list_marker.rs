use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseConsistentOrderedListMarkerOptions {
    /// Which delimiter to enforce: ".", ")", or "consistent".
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    delimiter: Option<String>,
}

impl UseConsistentOrderedListMarkerOptions {
    pub fn delimiter(&self) -> &str {
        self.delimiter.as_deref().unwrap_or(".")
    }
}
