#![deny(clippy::use_self)]

mod comments;
pub mod context;
mod cst;
mod generated;
mod prelude;
mod trivia;
mod turtle;
pub(crate) mod verbatim;

use crate::comments::TurtleCommentStyle;
pub(crate) use crate::context::TurtleFormatContext;
use crate::context::TurtleFormatOptions;
use crate::cst::FormatTurtleSyntaxNode;
pub(crate) use crate::trivia::*;
pub(crate) use crate::verbatim::{format_bogus_node, format_suppressed_node};
use biome_formatter::comments::Comments;
use biome_formatter::prelude::*;
use biome_formatter::trivia::{FormatToken, format_skipped_token_trivia};
use biome_formatter::{
    CstFormatContext, FormatContext, FormatLanguage, FormatOwnedWithRule, FormatRefWithRule,
    TransformSourceMap, write,
};
use biome_formatter::{Formatted, Printed};
use biome_rowan::{AstNode, SyntaxNode, SyntaxToken, TextRange};
use biome_turtle_syntax::{TurtleLanguage, TurtleSyntaxNode, TurtleSyntaxToken};

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
///
/// The difference to [AsFormat] is that this trait takes ownership of `self`.
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

pub(crate) type TurtleFormatter<'buf> = Formatter<'buf, TurtleFormatContext>;

/// Format a [TurtleSyntaxNode]
pub(crate) trait FormatNodeRule<N>
where
    N: AstNode<Language = TurtleLanguage>,
{
    fn fmt(&self, node: &N, f: &mut TurtleFormatter) -> FormatResult<()> {
        if self.is_suppressed(node, f) {
            return write!(f, [format_suppressed_node(node.syntax())]);
        }

        self.fmt_leading_comments(node, f)?;
        self.fmt_fields(node, f)?;
        self.fmt_dangling_comments(node, f)?;
        self.fmt_trailing_comments(node, f)
    }

    fn fmt_fields(&self, node: &N, f: &mut TurtleFormatter) -> FormatResult<()>;

    fn is_suppressed(&self, node: &N, f: &TurtleFormatter) -> bool {
        f.context().comments().is_suppressed(node.syntax())
    }

    fn fmt_leading_comments(&self, node: &N, f: &mut TurtleFormatter) -> FormatResult<()> {
        format_leading_comments(node.syntax()).fmt(f)
    }

    fn fmt_dangling_comments(&self, node: &N, f: &mut TurtleFormatter) -> FormatResult<()> {
        format_dangling_comments(node.syntax())
            .with_soft_block_indent()
            .fmt(f)
    }

    fn fmt_trailing_comments(&self, node: &N, f: &mut TurtleFormatter) -> FormatResult<()> {
        format_trailing_comments(node.syntax()).fmt(f)
    }
}

/// Rule for formatting bogus nodes.
pub(crate) trait FormatBogusNodeRule<N>
where
    N: AstNode<Language = TurtleLanguage>,
{
    fn fmt(&self, node: &N, f: &mut TurtleFormatter) -> FormatResult<()> {
        format_bogus_node(node.syntax()).fmt(f)
    }
}

/// Format implementation specific to Turtle tokens.
#[derive(Debug, Default)]
pub(crate) struct FormatTurtleSyntaxToken;

impl FormatRule<SyntaxToken<TurtleLanguage>> for FormatTurtleSyntaxToken {
    type Context = TurtleFormatContext;

    fn fmt(
        &self,
        token: &TurtleSyntaxToken,
        f: &mut Formatter<Self::Context>,
    ) -> FormatResult<()> {
        f.state_mut().track_token(token);

        self.format_skipped_token_trivia(token, f)?;
        self.format_trimmed_token_trivia(token, f)?;

        Ok(())
    }
}

impl FormatToken<TurtleLanguage, TurtleFormatContext> for FormatTurtleSyntaxToken {
    fn format_skipped_token_trivia(
        &self,
        token: &TurtleSyntaxToken,
        f: &mut Formatter<TurtleFormatContext>,
    ) -> FormatResult<()> {
        format_skipped_token_trivia(token).fmt(f)
    }
}

impl AsFormat<TurtleFormatContext> for TurtleSyntaxToken {
    type Format<'a> = FormatRefWithRule<'a, Self, FormatTurtleSyntaxToken>;

    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, FormatTurtleSyntaxToken)
    }
}

impl IntoFormat<TurtleFormatContext> for TurtleSyntaxToken {
    type Format = FormatOwnedWithRule<Self, FormatTurtleSyntaxToken>;

    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, FormatTurtleSyntaxToken)
    }
}

#[derive(Debug, Default, Clone)]
pub struct TurtleFormatLanguage {
    options: TurtleFormatOptions,
}

impl TurtleFormatLanguage {
    pub fn new(options: TurtleFormatOptions) -> Self {
        Self { options }
    }
}

impl FormatLanguage for TurtleFormatLanguage {
    type SyntaxLanguage = TurtleLanguage;
    type Context = TurtleFormatContext;
    type FormatRule = FormatTurtleSyntaxNode;

    fn is_range_formatting_node(&self, _node: &SyntaxNode<Self::SyntaxLanguage>) -> bool {
        true
    }

    fn options(&self) -> &<Self::Context as FormatContext>::Options {
        &self.options
    }

    fn create_context(
        self,
        root: &TurtleSyntaxNode,
        source_map: Option<TransformSourceMap>,
        _delegate_fmt_embedded_nodes: bool,
    ) -> Self::Context {
        let comments = Comments::from_node(root, &TurtleCommentStyle, source_map.as_ref());
        TurtleFormatContext::new(self.options, comments).with_source_map(source_map)
    }
}

/// Formats a range within a file, supported by Biome
pub fn format_range(
    options: TurtleFormatOptions,
    root: &TurtleSyntaxNode,
    range: TextRange,
) -> FormatResult<Printed> {
    biome_formatter::format_range(root, range, TurtleFormatLanguage::new(options))
}

/// Formats a Turtle syntax tree.
pub fn format_node(
    options: TurtleFormatOptions,
    root: &TurtleSyntaxNode,
) -> FormatResult<Formatted<TurtleFormatContext>> {
    biome_formatter::format_node(root, TurtleFormatLanguage::new(options), false)
}

/// Formats a single node within a file, supported by Biome.
pub fn format_sub_tree(
    options: TurtleFormatOptions,
    root: &TurtleSyntaxNode,
) -> FormatResult<Printed> {
    biome_formatter::format_sub_tree(root, TurtleFormatLanguage::new(options))
}
