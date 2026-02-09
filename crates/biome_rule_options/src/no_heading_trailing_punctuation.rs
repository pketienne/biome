use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoHeadingTrailingPunctuationOptions {
    /// Characters considered trailing punctuation (default: `".,;:!?"`).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub punctuation: Option<String>,
}

impl NoHeadingTrailingPunctuationOptions {
    pub const DEFAULT_PUNCTUATION: &'static str = ".,;:!?";

    pub fn punctuation(&self) -> &str {
        self.punctuation
            .as_deref()
            .unwrap_or(Self::DEFAULT_PUNCTUATION)
    }
}
