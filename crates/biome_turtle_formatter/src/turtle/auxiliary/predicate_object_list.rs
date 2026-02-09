use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtlePredicateObjectList;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtlePredicateObjectList;
impl FormatNodeRule<TurtlePredicateObjectList> for FormatTurtlePredicateObjectList {
    fn fmt_fields(
        &self,
        node: &TurtlePredicateObjectList,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        write!(f, [node.pairs().format()])
    }
}
