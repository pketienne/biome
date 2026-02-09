use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoInlineHtmlOptions {
    /// List of HTML elements that are allowed in markdown (default: empty).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub allowed_elements: Option<Vec<String>>,
}

impl NoInlineHtmlOptions {
    pub fn allowed_elements(&self) -> &[String] {
        match &self.allowed_elements {
            Some(v) => v.as_slice(),
            None => &[],
        }
    }
}
