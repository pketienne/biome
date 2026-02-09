use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleBlankNode;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleBlankNode;
impl FormatNodeRule<TurtleBlankNode> for FormatTurtleBlankNode {
    fn fmt_fields(&self, node: &TurtleBlankNode, f: &mut TurtleFormatter) -> FormatResult<()> {
        write!(f, [node.value()?.format()])
    }
}
