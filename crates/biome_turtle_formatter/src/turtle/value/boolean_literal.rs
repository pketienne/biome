use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleBooleanLiteral;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleBooleanLiteral;
impl FormatNodeRule<TurtleBooleanLiteral> for FormatTurtleBooleanLiteral {
    fn fmt_fields(&self, node: &TurtleBooleanLiteral, f: &mut TurtleFormatter) -> FormatResult<()> {
        write!(f, [node.value_token()?.format()])
    }
}
