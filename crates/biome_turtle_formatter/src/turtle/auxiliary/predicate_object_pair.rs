use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtlePredicateObjectPair;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtlePredicateObjectPair;
impl FormatNodeRule<TurtlePredicateObjectPair> for FormatTurtlePredicateObjectPair {
    fn fmt_fields(
        &self,
        node: &TurtlePredicateObjectPair,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        write!(
            f,
            [node.verb()?.format(), space(), node.objects().format()]
        )
    }
}
