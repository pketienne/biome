use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseConsistentMdxJsxQuoteStyleOptions {
    /// Expected quote style: `"double"`, `"single"`, or `"consistent"` (default).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub quote: Option<String>,
}

impl UseConsistentMdxJsxQuoteStyleOptions {
    pub fn quote(&self) -> &str {
        match &self.quote {
            Some(v) => v.as_str(),
            None => "consistent",
        }
    }
}
