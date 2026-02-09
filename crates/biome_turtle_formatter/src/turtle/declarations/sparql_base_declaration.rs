use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleSparqlBaseDeclaration;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleSparqlBaseDeclaration;
impl FormatNodeRule<TurtleSparqlBaseDeclaration> for FormatTurtleSparqlBaseDeclaration {
    fn fmt_fields(
        &self,
        node: &TurtleSparqlBaseDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        #[allow(non_snake_case)]
        let sparql_base_token = node.SPARQL_BASE_token()?;
        write!(
            f,
            [
                sparql_base_token.format(),
                space(),
                node.iri_token()?.format(),
            ]
        )
    }
}
