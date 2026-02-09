use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtlePrefixDeclaration;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtlePrefixDeclaration;
impl FormatNodeRule<TurtlePrefixDeclaration> for FormatTurtlePrefixDeclaration {
    fn fmt_fields(
        &self,
        node: &TurtlePrefixDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        write!(
            f,
            [
                node.prefix_token()?.format(),
                space(),
                node.namespace_token()?.format(),
                space(),
                node.iri_token()?.format(),
                space(),
                node.dot_token()?.format(),
            ]
        )
    }
}
