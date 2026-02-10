use biome_deserialize_macros::{Deserializable, Merge};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct UseValidSchemaOptions {
    /// Path to a JSON Schema file used to validate YAML documents.
    ///
    /// The path is resolved relative to the working directory (project root).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub schema_path: Option<String>,

    /// A map of glob patterns to schema file paths.
    ///
    /// When a YAML file's path matches a glob pattern, the corresponding
    /// schema is used for validation. Patterns are matched against the
    /// relative file path from the project root.
    ///
    /// Example:
    /// ```json
    /// {
    ///   "schemaAssociations": {
    ///     ".github/workflows/*.yml": "./schemas/github-workflow.json",
    ///     "docker-compose*.yml": "./schemas/docker-compose.json"
    ///   }
    /// }
    /// ```
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub schema_associations: Option<BTreeMap<String, String>>,
}
