use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseConsistentListItemIndentOptions {
    /// Which indent style to enforce: "one", "tab", or "space".
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    style: Option<String>,
}

impl UseConsistentListItemIndentOptions {
    pub fn style(&self) -> &str {
        self.style.as_deref().unwrap_or("one")
    }
}
