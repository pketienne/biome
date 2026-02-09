use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseProperNamesOptions {
    /// A list of proper names with their correct capitalization
    /// (e.g. `["JavaScript", "TypeScript", "GitHub"]`).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub names: Option<Vec<String>>,
}

impl UseProperNamesOptions {
    pub fn names(&self) -> &[String] {
        match &self.names {
            Some(v) => v.as_slice(),
            None => &[],
        }
    }
}
