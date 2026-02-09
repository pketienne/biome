#![deny(clippy::use_self)]

mod comments;
pub mod context;
mod cst;
mod generated;
mod yaml;
mod separated;
mod trivia;
mod prelude;
mod verbatim;

use crate::comments::YamlCommentStyle;
pub(crate) use crate::context::YamlFormatContext;
use crate::context::YamlFormatOptions;
use crate::cst::FormatYamlSyntaxNode;
pub(crate) use crate::trivia::*;
use crate::verbatim::{format_bogus_node, format_suppressed_node};
use biome_formatter::comments::Comments;
use biome_formatter::prelude::*;
use biome_formatter::trivia::{FormatToken, format_skipped_token_trivia};
use biome_formatter::{
    CstFormatContext, FormatContext, FormatLanguage, FormatOwnedWithRule, FormatRefWithRule,
    TransformSourceMap, write,
};
use biome_formatter::{Formatted, Printed};
use biome_rowan::{AstNode, SyntaxNode, TextRange};
use biome_yaml_syntax::{AnyYamlDocument, YamlLanguage, YamlSyntaxNode, YamlSyntaxToken};

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

pub(crate) type YamlFormatter<'buf> = Formatter<'buf, YamlFormatContext>;

/// Format a [YamlSyntaxNode]
pub(crate) trait FormatNodeRule<N>
where
    N: AstNode<Language = YamlLanguage>,
{
    fn fmt(&self, node: &N, f: &mut YamlFormatter) -> FormatResult<()> {
        if self.is_suppressed(node, f) {
            return write!(f, [format_suppressed_node(node.syntax())]);
        }

        self.fmt_leading_comments(node, f)?;
        self.fmt_fields(node, f)?;
        self.fmt_dangling_comments(node, f)?;
        self.fmt_trailing_comments(node, f)
    }

    fn fmt_fields(&self, node: &N, f: &mut YamlFormatter) -> FormatResult<()>;

    fn is_suppressed(&self, node: &N, f: &YamlFormatter) -> bool {
        f.context().comments().is_suppressed(node.syntax())
    }

    fn fmt_leading_comments(&self, node: &N, f: &mut YamlFormatter) -> FormatResult<()> {
        format_leading_comments(node.syntax()).fmt(f)
    }

    fn fmt_dangling_comments(&self, node: &N, f: &mut YamlFormatter) -> FormatResult<()> {
        format_dangling_comments(node.syntax())
            .with_soft_block_indent()
            .fmt(f)
    }

    fn fmt_trailing_comments(&self, node: &N, f: &mut YamlFormatter) -> FormatResult<()> {
        format_trailing_comments(node.syntax()).fmt(f)
    }
}

/// Rule for formatting bogus nodes.
pub(crate) trait FormatBogusNodeRule<N>
where
    N: AstNode<Language = YamlLanguage>,
{
    fn fmt(&self, node: &N, f: &mut YamlFormatter) -> FormatResult<()> {
        format_bogus_node(node.syntax()).fmt(f)
    }
}

#[derive(Debug, Default, Clone)]
pub struct YamlFormatLanguage {
    options: YamlFormatOptions,
}

impl YamlFormatLanguage {
    pub fn new(options: YamlFormatOptions) -> Self {
        Self { options }
    }
}

impl FormatLanguage for YamlFormatLanguage {
    type SyntaxLanguage = YamlLanguage;
    type Context = YamlFormatContext;
    type FormatRule = FormatYamlSyntaxNode;

    fn is_range_formatting_node(&self, node: &SyntaxNode<Self::SyntaxLanguage>) -> bool {
        use biome_yaml_syntax::YamlSyntaxKind::*;
        AnyYamlDocument::can_cast(node.kind())
            || matches!(
                node.kind(),
                YAML_BLOCK_MAPPING
                    | YAML_BLOCK_SEQUENCE
                    | YAML_BLOCK_MAP_EXPLICIT_ENTRY
                    | YAML_BLOCK_MAP_IMPLICIT_ENTRY
                    | YAML_BLOCK_SEQUENCE_ENTRY
            )
    }

    fn options(&self) -> &<Self::Context as FormatContext>::Options {
        &self.options
    }

    fn create_context(
        self,
        root: &YamlSyntaxNode,
        source_map: Option<TransformSourceMap>,
        _delegate_fmt_embedded_nodes: bool,
    ) -> Self::Context {
        let comments = Comments::from_node(root, &YamlCommentStyle, source_map.as_ref());
        YamlFormatContext::new(self.options, comments).with_source_map(source_map)
    }
}

/// Format implementation specific to YAML tokens.
#[derive(Debug, Default)]
pub(crate) struct FormatYamlSyntaxToken;

impl FormatRule<YamlSyntaxToken> for FormatYamlSyntaxToken {
    type Context = YamlFormatContext;

    fn fmt(
        &self,
        token: &YamlSyntaxToken,
        f: &mut Formatter<Self::Context>,
    ) -> FormatResult<()> {
        f.state_mut().track_token(token);

        self.format_skipped_token_trivia(token, f)?;
        self.format_trimmed_token_trivia(token, f)?;

        Ok(())
    }
}

impl FormatToken<YamlLanguage, YamlFormatContext> for FormatYamlSyntaxToken {
    fn format_skipped_token_trivia(
        &self,
        token: &YamlSyntaxToken,
        f: &mut Formatter<YamlFormatContext>,
    ) -> FormatResult<()> {
        format_skipped_token_trivia(token).fmt(f)
    }
}

impl AsFormat<YamlFormatContext> for YamlSyntaxToken {
    type Format<'a> = FormatRefWithRule<'a, Self, FormatYamlSyntaxToken>;

    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, FormatYamlSyntaxToken)
    }
}

impl IntoFormat<YamlFormatContext> for YamlSyntaxToken {
    type Format = FormatOwnedWithRule<Self, FormatYamlSyntaxToken>;

    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, FormatYamlSyntaxToken)
    }
}

/// Formats a YAML syntax tree.
pub fn format_node(
    options: YamlFormatOptions,
    root: &YamlSyntaxNode,
) -> FormatResult<Formatted<YamlFormatContext>> {
    biome_formatter::format_node(root, YamlFormatLanguage::new(options), false)
}

/// Formats a range within a YAML file.
pub fn format_range(
    options: YamlFormatOptions,
    root: &YamlSyntaxNode,
    range: TextRange,
) -> FormatResult<Printed> {
    biome_formatter::format_range(root, range, YamlFormatLanguage::new(options))
}

/// Formats a YAML sub-tree that does not encompass the root of the tree.
pub fn format_sub_tree(
    options: YamlFormatOptions,
    root: &YamlSyntaxNode,
) -> FormatResult<Printed> {
    biome_formatter::format_sub_tree(root, YamlFormatLanguage::new(options))
}

#[cfg(test)]
mod tests {
    use crate::context::YamlFormatOptions;
    use crate::format_node;
    use biome_yaml_parser::parse_yaml;

    #[test]
    fn smoke_test() {
        let src = "";
        let parse = parse_yaml(src);
        let options = YamlFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        // Empty input should produce empty output
        assert_eq!(result.as_code(), src);
    }

    #[test]
    fn simple_mapping() {
        let src = "key: value\n";
        let parse = parse_yaml(src);
        let options = YamlFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "key: value\n");
    }

    #[test]
    fn nested_mapping() {
        let src = "parent:\n  child: value\n";
        let parse = parse_yaml(src);
        let options = YamlFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "parent:\n  child: value\n");
    }

    #[test]
    fn block_sequence() {
        let src = "- item1\n- item2\n";
        let parse = parse_yaml(src);
        let options = YamlFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "- item1\n- item2\n");
    }

    #[test]
    fn multiple_keys() {
        let src = "key1: value1\nkey2: value2\n";
        let parse = parse_yaml(src);
        let options = YamlFormatOptions::default();
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "key1: value1\nkey2: value2\n");
    }

    #[test]
    fn quote_style_double_to_single() {
        use biome_formatter::QuoteStyle;
        let src = "key: \"hello world\"\n";
        let parse = parse_yaml(src);
        let options = YamlFormatOptions::default().with_quote_style(QuoteStyle::Single);
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "key: 'hello world'\n");
    }

    #[test]
    fn quote_style_single_to_double() {
        use biome_formatter::QuoteStyle;
        let src = "key: 'hello world'\n";
        let parse = parse_yaml(src);
        let options = YamlFormatOptions::default().with_quote_style(QuoteStyle::Double);
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "key: \"hello world\"\n");
    }

    #[test]
    fn quote_style_preserves_when_unsafe() {
        use biome_formatter::QuoteStyle;
        // Double-quoted with escape sequence — can't convert to single
        let src = "key: \"line1\\nline2\"\n";
        let parse = parse_yaml(src);
        let options = YamlFormatOptions::default().with_quote_style(QuoteStyle::Single);
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "key: \"line1\\nline2\"\n");
    }

    #[test]
    fn quote_style_preserves_when_target_quote_in_content() {
        use biome_formatter::QuoteStyle;
        // Double-quoted with single quote in content — can't convert to single
        let src = "key: \"it's here\"\n";
        let parse = parse_yaml(src);
        let options = YamlFormatOptions::default().with_quote_style(QuoteStyle::Single);
        let formatted = format_node(options, &parse.syntax()).unwrap();
        let result = formatted.print().unwrap();
        assert_eq!(result.as_code(), "key: \"it's here\"\n");
    }
}
