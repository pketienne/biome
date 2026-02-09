use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleNumericLiteral;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleNumericLiteral;
impl FormatNodeRule<TurtleNumericLiteral> for FormatTurtleNumericLiteral {
    fn fmt_fields(&self, node: &TurtleNumericLiteral, f: &mut TurtleFormatter) -> FormatResult<()> {
        write!(f, [node.value()?.format()])
    }
}
