use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleSubject;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleSubject;
impl FormatNodeRule<TurtleSubject> for FormatTurtleSubject {
    fn fmt_fields(&self, node: &TurtleSubject, f: &mut TurtleFormatter) -> FormatResult<()> {
        write!(f, [node.any_turtle_subject()?.format()])
    }
}
