use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtlePrefixedName;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtlePrefixedName;
impl FormatNodeRule<TurtlePrefixedName> for FormatTurtlePrefixedName {
    fn fmt_fields(&self, node: &TurtlePrefixedName, f: &mut TurtleFormatter) -> FormatResult<()> {
        write!(f, [node.value()?.format()])
    }
}
