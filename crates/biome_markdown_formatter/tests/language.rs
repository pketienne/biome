use biome_formatter::{IndentStyle, IndentWidth, LineEnding, LineWidth};
use biome_formatter_test::TestFormatLanguage;
use biome_fs::BiomePath;
use biome_markdown_formatter::MarkdownFormatLanguage;
use biome_markdown_formatter::context::{MarkdownFormatContext, MarkdownFormatOptions};
use biome_markdown_parser::parse_markdown;
use biome_markdown_syntax::MarkdownLanguage;
use biome_parser::AnyParse;
use biome_service::settings::{ServiceLanguage, Settings};
use biome_service::workspace::DocumentFileSource;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct MarkdownTestFormatLanguage;

impl TestFormatLanguage for MarkdownTestFormatLanguage {
    type ServiceLanguage = MarkdownLanguage;
    type Context = MarkdownFormatContext;
    type FormatLanguage = MarkdownFormatLanguage;

    fn parse(&self, text: &str) -> AnyParse {
        parse_markdown(text).into()
    }

    fn to_format_language(
        &self,
        settings: &Settings,
        file_source: &DocumentFileSource,
    ) -> Self::FormatLanguage {
        let language_settings = &settings.languages.markdown.formatter;
        let options = <MarkdownLanguage as ServiceLanguage>::resolve_format_options(
            &settings.formatter,
            &settings.override_settings,
            language_settings,
            &BiomePath::new(""),
            file_source,
        );
        MarkdownFormatLanguage::new(options)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum MarkdownSerializableIndentStyle {
    /// Tab
    Tab,
    /// Space
    Space,
}

impl From<MarkdownSerializableIndentStyle> for IndentStyle {
    fn from(test: MarkdownSerializableIndentStyle) -> Self {
        match test {
            MarkdownSerializableIndentStyle::Tab => Self::Tab,
            MarkdownSerializableIndentStyle::Space => Self::Space,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum MarkdownSerializableLineEnding {
    ///  Line Feed only (\n), common on Linux and macOS as well as inside git repos
    Lf,

    /// Carriage Return + Line Feed characters (\r\n), common on Windows
    Crlf,

    /// Carriage Return character only (\r), used very rarely
    Cr,
}

impl From<MarkdownSerializableLineEnding> for LineEnding {
    fn from(test: MarkdownSerializableLineEnding) -> Self {
        match test {
            MarkdownSerializableLineEnding::Lf => Self::Lf,
            MarkdownSerializableLineEnding::Crlf => Self::Crlf,
            MarkdownSerializableLineEnding::Cr => Self::Cr,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct MarkdownSerializableFormatOptions {
    /// The indent style.
    pub indent_style: Option<MarkdownSerializableIndentStyle>,

    /// The indent width.
    pub indent_width: Option<u8>,

    /// The type of line ending.
    pub line_ending: Option<MarkdownSerializableLineEnding>,

    /// What's the max width of a line. Defaults to 80.
    pub line_width: Option<u16>,
}

impl From<MarkdownSerializableFormatOptions> for MarkdownFormatOptions {
    fn from(test: MarkdownSerializableFormatOptions) -> Self {
        Self::default()
            .with_indent_style(test.indent_style.map(Into::into).unwrap_or_default())
            .with_indent_width(
                test.indent_width
                    .and_then(|width| IndentWidth::try_from(width).ok())
                    .unwrap_or_default(),
            )
            .with_line_width(
                test.line_width
                    .and_then(|width| LineWidth::try_from(width).ok())
                    .unwrap_or_default(),
            )
    }
}
