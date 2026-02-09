use crate::FormatMarkdownSyntaxToken;
use crate::prelude::*;
use biome_formatter::FormatRefWithRule;
use biome_formatter::separated::{
    FormatSeparatedElementRule, FormatSeparatedIter, TrailingSeparator,
};
use biome_markdown_syntax::{MarkdownLanguage, MarkdownSyntaxToken};
use biome_rowan::{AstNode, AstSeparatedList, AstSeparatedListElementsIterator};
use std::marker::PhantomData;

#[derive(Clone)]
pub(crate) struct MarkdownFormatSeparatedElementRule<N> {
    node: PhantomData<N>,
}

impl<N> FormatSeparatedElementRule<N> for MarkdownFormatSeparatedElementRule<N>
where
    N: AstNode<Language = MarkdownLanguage> + AsFormat<MarkdownFormatContext> + 'static,
{
    type Context = MarkdownFormatContext;
    type FormatNode<'a> = N::Format<'a>;
    type FormatSeparator<'a> =
        FormatRefWithRule<'a, MarkdownSyntaxToken, FormatMarkdownSyntaxToken>;

    fn format_node<'a>(&self, node: &'a N) -> Self::FormatNode<'a> {
        node.format()
    }

    fn format_separator<'a>(
        &self,
        separator: &'a MarkdownSyntaxToken,
    ) -> Self::FormatSeparator<'a> {
        separator.format()
    }
}

type MarkdownFormatSeparatedIter<Node, C> = FormatSeparatedIter<
    AstSeparatedListElementsIterator<MarkdownLanguage, Node>,
    Node,
    MarkdownFormatSeparatedElementRule<Node>,
    C,
>;

pub(crate) trait FormatAstSeparatedListExtension:
    AstSeparatedList<Language = MarkdownLanguage>
{
    fn format_separated(
        &self,
        separator: &'static str,
        trailing_separator: TrailingSeparator,
    ) -> MarkdownFormatSeparatedIter<Self::Node, MarkdownFormatContext> {
        MarkdownFormatSeparatedIter::new(
            self.elements(),
            separator,
            MarkdownFormatSeparatedElementRule { node: PhantomData },
            on_skipped,
            on_removed,
        )
        .with_trailing_separator(trailing_separator)
    }
}

impl<T> FormatAstSeparatedListExtension for T where T: AstSeparatedList<Language = MarkdownLanguage> {}

use crate::trivia::{on_removed, on_skipped};
