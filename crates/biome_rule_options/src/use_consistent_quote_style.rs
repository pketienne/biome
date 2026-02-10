use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

/// Options for the `useConsistentQuoteStyle` rule.
#[derive(Default, Clone, Debug, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseConsistentQuoteStyleOptions {
    /// The preferred quote style for strings.
    /// Defaults to `double`.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub preferred_quote: Option<PreferredQuote>,
}

impl UseConsistentQuoteStyleOptions {
    pub fn preferred_quote(&self) -> PreferredQuote {
        self.preferred_quote.unwrap_or_default()
    }
}

/// The preferred quote style.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum PreferredQuote {
    /// Prefer double quotes (`"`).
    #[default]
    Double,
    /// Prefer single quotes (`'`).
    Single,
}
