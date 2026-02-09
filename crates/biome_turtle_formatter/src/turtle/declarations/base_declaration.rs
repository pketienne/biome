use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleBaseDeclaration;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleBaseDeclaration;
impl FormatNodeRule<TurtleBaseDeclaration> for FormatTurtleBaseDeclaration {
    fn fmt_fields(
        &self,
        node: &TurtleBaseDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        write!(
            f,
            [
                node.base_token()?.format(),
                space(),
                node.iri_token()?.format(),
                space(),
                node.dot_token()?.format(),
            ]
        )
    }
}
