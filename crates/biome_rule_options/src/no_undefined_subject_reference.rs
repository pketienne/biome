use biome_deserialize_macros::Deserializable;
use serde::{Deserialize, Serialize};
#[derive(Default, Clone, Debug, Deserialize, Deserializable, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoUndefinedSubjectReferenceOptions {
    /// Additional prefixes to allow (external vocabularies, e.g., `["ex:", "org:"]`).
    /// Common vocabulary prefixes (rdf:, rdfs:, owl:, xsd:, foaf:, dc:, etc.)
    /// are always allowed by default.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub allowed_prefixes: Option<Box<[Box<str>]>>,
}

impl biome_deserialize::Merge for NoUndefinedSubjectReferenceOptions {
    fn merge_with(&mut self, other: Self) {
        if let Some(allowed_prefixes) = other.allowed_prefixes {
            self.allowed_prefixes = Some(allowed_prefixes);
        }
    }
}
