use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseConsistentHorizontalRuleStyleOptions {
    /// The expected horizontal rule style (default: `"---"`).
    /// Common styles: `"---"`, `"***"`, `"___"`.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub style: Option<String>,
}

impl UseConsistentHorizontalRuleStyleOptions {
    pub const DEFAULT_STYLE: &'static str = "---";

    pub fn style(&self) -> &str {
        self.style.as_deref().unwrap_or(Self::DEFAULT_STYLE)
    }
}
