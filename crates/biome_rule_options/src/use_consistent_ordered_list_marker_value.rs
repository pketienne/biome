use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseConsistentOrderedListMarkerValueOptions {
    /// The style to use for ordered list marker values (default: "ordered"). Allowed values: "ordered" or "one".
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub style: Option<String>,
}

impl UseConsistentOrderedListMarkerValueOptions {
    pub const DEFAULT_STYLE: &'static str = "ordered";

    pub fn style(&self) -> &str {
        self.style.as_deref().unwrap_or(Self::DEFAULT_STYLE)
    }
}
