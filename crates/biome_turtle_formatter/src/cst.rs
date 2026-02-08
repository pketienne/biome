use crate::prelude::*;
use biome_formatter::{FormatOwnedWithRule, FormatRefWithRule, FormatResult};
use biome_turtle_syntax::{TurtleSyntaxNode, map_syntax_node};

#[derive(Debug, Copy, Clone, Default)]
pub struct FormatTurtleSyntaxNode;

impl FormatRule<TurtleSyntaxNode> for FormatTurtleSyntaxNode {
    type Context = TurtleFormatContext;

    fn fmt(&self, node: &TurtleSyntaxNode, f: &mut TurtleFormatter) -> FormatResult<()> {
        map_syntax_node!(node.clone(), node => node.format().fmt(f))
    }
}

impl AsFormat<TurtleFormatContext> for TurtleSyntaxNode {
    type Format<'a> = FormatRefWithRule<'a, Self, FormatTurtleSyntaxNode>;

    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, FormatTurtleSyntaxNode)
    }
}

impl IntoFormat<TurtleFormatContext> for TurtleSyntaxNode {
    type Format = FormatOwnedWithRule<Self, FormatTurtleSyntaxNode>;

    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, FormatTurtleSyntaxNode)
    }
}
