#![deny(clippy::use_self)]

mod comments;
pub mod context;
mod cst;
mod generated;
mod js;
mod md;
mod prelude;

mod trivia;
mod verbatim;

use crate::comments::MarkdownCommentStyle;
pub(crate) use crate::context::MarkdownFormatContext;
use crate::context::MarkdownFormatOptions;
use crate::cst::FormatMarkdownSyntaxNode;

use crate::verbatim::{format_bogus_node, format_suppressed_node};
use biome_formatter::comments::Comments;
use biome_formatter::prelude::*;
use biome_formatter::trivia::{FormatToken, format_skipped_token_trivia};
use biome_formatter::{
    CstFormatContext, FormatContext, FormatLanguage, FormatOwnedWithRule, FormatRefWithRule,
    TransformSourceMap, write,
};
use biome_formatter::{Formatted, Printed};
use biome_markdown_syntax::{
    AnyMdBlock, MarkdownLanguage, MarkdownSyntaxNode, MarkdownSyntaxToken,
};
use biome_rowan::{AstNode, SyntaxNode, TextRange};

/// Used to get an object that knows how to format this object.
pub(crate) trait AsFormat<Context> {
    type Format<'a>: biome_formatter::Format<Context>
    where
        Self: 'a;

    /// Returns an object that is able to format this object.
    fn format(&self) -> Self::Format<'_>;
}

/// Implement [AsFormat] for references to types that implement [AsFormat].
impl<T, C> AsFormat<C> for &T
where
    T: AsFormat<C>,
{
    type Format<'a>
        = T::Format<'a>
    where
        Self: 'a;

    fn format(&self) -> Self::Format<'_> {
        AsFormat::format(&**self)
    }
}

/// Implement [AsFormat] for [SyntaxResult] where `T` implements [AsFormat].
impl<T, C> AsFormat<C> for biome_rowan::SyntaxResult<T>
where
    T: AsFormat<C>,
{
    type Format<'a>
        = biome_rowan::SyntaxResult<T::Format<'a>>
    where
        Self: 'a;

    fn format(&self) -> Self::Format<'_> {
        match self {
            Ok(value) => Ok(value.format()),
            Err(err) => Err(*err),
        }
    }
}

/// Implement [AsFormat] for [Option] when `T` implements [AsFormat]
impl<T, C> AsFormat<C> for Option<T>
where
    T: AsFormat<C>,
{
    type Format<'a>
        = Option<T::Format<'a>>
    where
        Self: 'a;

    fn format(&self) -> Self::Format<'_> {
        self.as_ref().map(|value| value.format())
    }
}

/// Used to convert this object into an object that can be formatted.
pub(crate) trait IntoFormat<Context> {
    type Format: biome_formatter::Format<Context>;

    fn into_format(self) -> Self::Format;
}

impl<T, Context> IntoFormat<Context> for biome_rowan::SyntaxResult<T>
where
    T: IntoFormat<Context>,
{
    type Format = biome_rowan::SyntaxResult<T::Format>;

    fn into_format(self) -> Self::Format {
        self.map(IntoFormat::into_format)
    }
}

impl<T, Context> IntoFormat<Context> for Option<T>
where
    T: IntoFormat<Context>,
{
    type Format = Option<T::Format>;

    fn into_format(self) -> Self::Format {
        self.map(IntoFormat::into_format)
    }
}

/// Formatting specific [Iterator] extensions
pub(crate) trait FormattedIterExt {
    /// Converts every item to an object that knows how to format it.
    fn formatted<Context>(self) -> FormattedIter<Self, Self::Item, Context>
    where
        Self: Iterator + Sized,
        Self::Item: IntoFormat<Context>,
    {
        FormattedIter {
            inner: self,
            options: std::marker::PhantomData,
        }
    }
}

impl<I> FormattedIterExt for I where I: std::iter::Iterator {}

pub(crate) struct FormattedIter<Iter, Item, Context>
where
    Iter: Iterator<Item = Item>,
{
    inner: Iter,
    options: std::marker::PhantomData<Context>,
}

impl<Iter, Item, Context> std::iter::Iterator for FormattedIter<Iter, Item, Context>
where
    Iter: Iterator<Item = Item>,
    Item: IntoFormat<Context>,
{
    type Item = Item::Format;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.into_format())
    }
}

impl<Iter, Item, Context> std::iter::FusedIterator for FormattedIter<Iter, Item, Context>
where
    Iter: std::iter::FusedIterator<Item = Item>,
    Item: IntoFormat<Context>,
{
}

impl<Iter, Item, Context> std::iter::ExactSizeIterator for FormattedIter<Iter, Item, Context>
where
    Iter: Iterator<Item = Item> + std::iter::ExactSizeIterator,
    Item: IntoFormat<Context>,
{
}

pub(crate) type MarkdownFormatter<'buf> = Formatter<'buf, MarkdownFormatContext>;

/// Format a [MarkdownSyntaxNode]
pub(crate) trait FormatNodeRule<N>
where
    N: AstNode<Language = MarkdownLanguage>,
{
    fn fmt(&self, node: &N, f: &mut MarkdownFormatter) -> FormatResult<()> {
        if self.is_suppressed(node, f) {
            return write!(f, [format_suppressed_node(node.syntax())]);
        }

        self.fmt_leading_comments(node, f)?;
        self.fmt_fields(node, f)?;
        self.fmt_dangling_comments(node, f)?;
        self.fmt_trailing_comments(node, f)
    }

    fn fmt_fields(&self, node: &N, f: &mut MarkdownFormatter) -> FormatResult<()>;

    fn is_suppressed(&self, node: &N, f: &MarkdownFormatter) -> bool {
        f.context().comments().is_suppressed(node.syntax())
    }

    fn fmt_leading_comments(&self, node: &N, f: &mut MarkdownFormatter) -> FormatResult<()> {
        format_leading_comments(node.syntax()).fmt(f)
    }

    fn fmt_dangling_comments(&self, node: &N, f: &mut MarkdownFormatter) -> FormatResult<()> {
        format_dangling_comments(node.syntax())
            .with_soft_block_indent()
            .fmt(f)
    }

    fn fmt_trailing_comments(&self, node: &N, f: &mut MarkdownFormatter) -> FormatResult<()> {
        format_trailing_comments(node.syntax()).fmt(f)
    }
}

/// Rule for formatting bogus nodes.
pub(crate) trait FormatBogusNodeRule<N>
where
    N: AstNode<Language = MarkdownLanguage>,
{
    fn fmt(&self, node: &N, f: &mut MarkdownFormatter) -> FormatResult<()> {
        format_bogus_node(node.syntax()).fmt(f)
    }
}

#[derive(Debug, Default, Clone)]
pub struct MarkdownFormatLanguage {
    options: MarkdownFormatOptions,
}

impl MarkdownFormatLanguage {
    pub fn new(options: MarkdownFormatOptions) -> Self {
        Self { options }
    }
}

impl FormatLanguage for MarkdownFormatLanguage {
    type SyntaxLanguage = MarkdownLanguage;
    type Context = MarkdownFormatContext;
    type FormatRule = FormatMarkdownSyntaxNode;

    fn is_range_formatting_node(&self, node: &SyntaxNode<Self::SyntaxLanguage>) -> bool {
        AnyMdBlock::can_cast(node.kind())
    }

    fn options(&self) -> &<Self::Context as FormatContext>::Options {
        &self.options
    }

    fn create_context(
        self,
        root: &MarkdownSyntaxNode,
        source_map: Option<TransformSourceMap>,
        _delegate_fmt_embedded_nodes: bool,
    ) -> Self::Context {
        let comments = Comments::from_node(root, &MarkdownCommentStyle, source_map.as_ref());
        MarkdownFormatContext::new(self.options, comments).with_source_map(source_map)
    }
}

/// Format implementation specific to Markdown tokens.
#[derive(Debug, Default)]
pub(crate) struct FormatMarkdownSyntaxToken;

impl FormatRule<MarkdownSyntaxToken> for FormatMarkdownSyntaxToken {
    type Context = MarkdownFormatContext;

    fn fmt(
        &self,
        token: &MarkdownSyntaxToken,
        f: &mut Formatter<Self::Context>,
    ) -> FormatResult<()> {
        f.state_mut().track_token(token);

        self.format_skipped_token_trivia(token, f)?;
        self.format_trimmed_token_trivia(token, f)?;

        Ok(())
    }
}

impl FormatToken<MarkdownLanguage, MarkdownFormatContext> for FormatMarkdownSyntaxToken {
    fn format_skipped_token_trivia(
        &self,
        token: &MarkdownSyntaxToken,
        f: &mut Formatter<MarkdownFormatContext>,
    ) -> FormatResult<()> {
        format_skipped_token_trivia(token).fmt(f)
    }
}

impl AsFormat<MarkdownFormatContext> for MarkdownSyntaxToken {
    type Format<'a> = FormatRefWithRule<'a, Self, FormatMarkdownSyntaxToken>;

    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, FormatMarkdownSyntaxToken)
    }
}

impl IntoFormat<MarkdownFormatContext> for MarkdownSyntaxToken {
    type Format = FormatOwnedWithRule<Self, FormatMarkdownSyntaxToken>;

    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, FormatMarkdownSyntaxToken)
    }
}

/// Formats a Markdown syntax tree.
pub fn format_node(
    options: MarkdownFormatOptions,
    root: &MarkdownSyntaxNode,
) -> FormatResult<Formatted<MarkdownFormatContext>> {
    biome_formatter::format_node(root, MarkdownFormatLanguage::new(options), false)
}

/// Formats a range within a Markdown file.
pub fn format_range(
    options: MarkdownFormatOptions,
    root: &MarkdownSyntaxNode,
    range: TextRange,
) -> FormatResult<Printed> {
    biome_formatter::format_range(root, range, MarkdownFormatLanguage::new(options))
}

#[cfg(test)]
mod tests {
    use crate::context::MarkdownFormatOptions;
    use crate::format_node;
    use biome_markdown_parser::parse_markdown;


    #[test]
    fn smoke_test() {
        let src = "# Hello\n\nWorld\n";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), src);
    }

    #[test]
    fn ensures_final_newline() {
        let src = "# Hello\n\nWorld";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "# Hello\n\nWorld\n");
    }

    #[test]
    fn preserves_blank_lines() {
        let src = "# Title\n\nParagraph one.\n\nParagraph two.\n";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), src);
    }

    #[test]
    fn handles_empty_document() {
        let src = "";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        // Empty documents stay empty
        assert_eq!(result.as_code(), "");
    }

    #[test]
    fn handles_heading_only() {
        let src = "# Hello\n";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), src);
    }

    #[test]
    fn preserves_thematic_break() {
        let src = "# Title\n\n---\n\nContent\n";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), src);
    }

    #[test]
    fn normalizes_thematic_break_stars() {
        let src = "***\n";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "---\n");
    }

    #[test]
    fn normalizes_thematic_break_underscores() {
        let src = "___\n";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "---\n");
    }

    #[test]
    fn normalizes_heading_space() {
        let src = "#Hello\n";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "# Hello\n");
    }

    #[test]
    fn normalizes_heading_extra_spaces() {
        let src = "##  Title\n";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "## Title\n");
    }

    #[test]
    fn trailing_hashes_preserved_by_parser() {
        // Parser doesn't separate trailing hashes from content,
        // so they stay as-is. Removal requires parser changes.
        let src = "### Heading ###\n";
        let parse = parse_markdown(src);
        let options = MarkdownFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "### Heading ###\n");
    }
}
