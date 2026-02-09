use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleObject;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleObject;
impl FormatNodeRule<TurtleObject> for FormatTurtleObject {
    fn fmt_fields(&self, node: &TurtleObject, f: &mut TurtleFormatter) -> FormatResult<()> {
        write!(f, [node.any_turtle_object()?.format()])
    }
}
