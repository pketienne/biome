use biome_rowan::FileSourceError;
use camino::Utf8Path;

/// Represents the type of a Markdown file.
#[derive(
    Debug,
    Clone,
    Default,
    Copy,
    Eq,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MarkdownFileSource {
    variant: MarkdownVariant,
}

#[derive(
    Debug,
    Clone,
    Default,
    Copy,
    Eq,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
enum MarkdownVariant {
    #[default]
    Standard,
    Mdx,
}

impl MarkdownFileSource {
    pub fn markdown() -> Self {
        Self {
            variant: MarkdownVariant::Standard,
        }
    }

    pub fn mdx() -> Self {
        Self {
            variant: MarkdownVariant::Mdx,
        }
    }

    pub fn is_markdown(&self) -> bool {
        matches!(self.variant, MarkdownVariant::Standard)
    }

    pub fn is_mdx(&self) -> bool {
        matches!(self.variant, MarkdownVariant::Mdx)
    }

    /// Try to return the Markdown file source corresponding to this file extension
    pub fn try_from_extension(extension: &str) -> Result<Self, FileSourceError> {
        match extension {
            "md" | "markdown" => Ok(Self::markdown()),
            "mdx" => Ok(Self::mdx()),
            _ => Err(FileSourceError::UnknownExtension),
        }
    }

    /// Try to return the Markdown file source corresponding to this language ID
    ///
    /// See the [LSP spec] and [VS Code spec] for a list of language identifiers
    ///
    /// [LSP spec]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentItem
    /// [VS Code spec]: https://code.visualstudio.com/docs/languages/identifiers
    pub fn try_from_language_id(language_id: &str) -> Result<Self, FileSourceError> {
        match language_id {
            "markdown" => Ok(Self::markdown()),
            "mdx" => Ok(Self::mdx()),
            _ => Err(FileSourceError::UnknownLanguageId),
        }
    }

    /// Try to return the Markdown file source corresponding to this file name from well-known files
    pub fn try_from_well_known(_path: &Utf8Path) -> Result<Self, FileSourceError> {
        // No well-known markdown files to detect by name
        Err(FileSourceError::UnknownFileName)
    }
}
