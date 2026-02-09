use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleString;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleString;
impl FormatNodeRule<TurtleString> for FormatTurtleString {
    fn fmt_fields(&self, node: &TurtleString, f: &mut TurtleFormatter) -> FormatResult<()> {
        write!(f, [node.value()?.format()])
    }
}
