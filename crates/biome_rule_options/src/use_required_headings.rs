use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseRequiredHeadingsOptions {
    /// A list of required heading strings (e.g. `["# Title", "## Introduction"]`).
    /// Use `*` as a wildcard to match any heading.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub headings: Option<Vec<String>>,
}

impl UseRequiredHeadingsOptions {
    pub fn headings(&self) -> &[String] {
        match &self.headings {
            Some(v) => v.as_slice(),
            None => &[],
        }
    }
}
