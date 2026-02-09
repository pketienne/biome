use biome_deserialize_macros::Deserializable;
use serde::{Deserialize, Serialize};
#[derive(Default, Clone, Debug, Deserialize, Deserializable, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoUnusedPrefixOptions {
    /// Prefix namespaces to ignore (e.g., `["owl:", "skos:"]`).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub ignored_prefixes: Option<Box<[Box<str>]>>,

    /// When `true`, unused prefix declarations are not flagged.
    /// Default: `false`.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub keep_unused_prefixes: Option<bool>,
}

impl biome_deserialize::Merge for NoUnusedPrefixOptions {
    fn merge_with(&mut self, other: Self) {
        if let Some(ignored_prefixes) = other.ignored_prefixes {
            self.ignored_prefixes = Some(ignored_prefixes);
        }
        if let Some(keep_unused_prefixes) = other.keep_unused_prefixes {
            self.keep_unused_prefixes = Some(keep_unused_prefixes);
        }
    }
}
