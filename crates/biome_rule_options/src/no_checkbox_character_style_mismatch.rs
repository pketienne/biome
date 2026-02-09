use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoCheckboxCharacterStyleMismatchOptions {
    /// The style to use for checked checkbox characters (default: "lowercase"). Allowed values: "lowercase", "uppercase", or "consistent".
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub checked: Option<String>,
}

impl NoCheckboxCharacterStyleMismatchOptions {
    pub const DEFAULT_CHECKED: &'static str = "lowercase";

    pub fn checked(&self) -> &str {
        self.checked.as_deref().unwrap_or(Self::DEFAULT_CHECKED)
    }
}
