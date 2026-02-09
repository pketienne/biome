//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

use crate::prelude::*;
use biome_markdown_syntax::{AnyCodeBlock, AnyContainerBlock, AnyLeafBlock};
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyCodeBlock;
impl FormatRule<AnyCodeBlock> for FormatAnyCodeBlock {
    type Context = MarkdownFormatContext;
    fn fmt(&self, node: &AnyCodeBlock, f: &mut MarkdownFormatter) -> FormatResult<()> {
        match node {
            AnyCodeBlock::MdFencedCodeBlock(node) => node.format().fmt(f),
            AnyCodeBlock::MdIndentCodeBlock(node) => node.format().fmt(f),
        }
    }
}
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyContainerBlock;
impl FormatRule<AnyContainerBlock> for FormatAnyContainerBlock {
    type Context = MarkdownFormatContext;
    fn fmt(&self, node: &AnyContainerBlock, f: &mut MarkdownFormatter) -> FormatResult<()> {
        match node {
            AnyContainerBlock::MdBulletListItem(node) => node.format().fmt(f),
            AnyContainerBlock::MdOrderListItem(node) => node.format().fmt(f),
            AnyContainerBlock::MdQuote(node) => node.format().fmt(f),
        }
    }
}
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyLeafBlock;
impl FormatRule<AnyLeafBlock> for FormatAnyLeafBlock {
    type Context = MarkdownFormatContext;
    fn fmt(&self, node: &AnyLeafBlock, f: &mut MarkdownFormatter) -> FormatResult<()> {
        match node {
            AnyLeafBlock::AnyCodeBlock(node) => node.format().fmt(f),
            AnyLeafBlock::MdHeader(node) => node.format().fmt(f),
            AnyLeafBlock::MdHtmlBlock(node) => node.format().fmt(f),
            AnyLeafBlock::MdLinkBlock(node) => node.format().fmt(f),
            AnyLeafBlock::MdParagraph(node) => node.format().fmt(f),
            AnyLeafBlock::MdSetextHeader(node) => node.format().fmt(f),
            AnyLeafBlock::MdThematicBreakBlock(node) => node.format().fmt(f),
        }
    }
}
