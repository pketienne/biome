use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseFileExtensionOptions {
    /// Expected file extension (default: `"md"`).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub extension: Option<String>,
}

impl UseFileExtensionOptions {
    pub fn extension(&self) -> &str {
        match &self.extension {
            Some(v) => v.as_str(),
            None => "md",
        }
    }
}
