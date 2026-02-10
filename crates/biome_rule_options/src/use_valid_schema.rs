use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseValidSchemaOptions {
    /// Path to a JSON Schema file used to validate YAML documents.
    ///
    /// The path is resolved relative to the working directory (project root).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub schema_path: Option<String>,
}
