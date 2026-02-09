use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleSparqlPrefixDeclaration;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleSparqlPrefixDeclaration;
impl FormatNodeRule<TurtleSparqlPrefixDeclaration> for FormatTurtleSparqlPrefixDeclaration {
    fn fmt_fields(
        &self,
        node: &TurtleSparqlPrefixDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        #[allow(non_snake_case)]
        let sparql_prefix_token = node.SPARQL_PREFIX_token()?;
        write!(
            f,
            [
                sparql_prefix_token.format(),
                space(),
                node.namespace_token()?.format(),
                space(),
                node.iri_token()?.format(),
            ]
        )
    }
}
