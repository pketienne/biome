use crate::prelude::*;
use biome_configuration::turtle::TurtleDirectiveStyle;
use biome_formatter::prelude::syntax_token_cow_slice;
use biome_formatter::write;
use biome_turtle_syntax::{TextSize, TurtleSparqlPrefixDeclaration};
use std::borrow::Cow;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleSparqlPrefixDeclaration;

impl FormatNodeRule<TurtleSparqlPrefixDeclaration> for FormatTurtleSparqlPrefixDeclaration {
    fn fmt_fields(
        &self,
        node: &TurtleSparqlPrefixDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        let directive_style = f.options().directive_style();

        #[allow(non_snake_case)]
        let sparql_prefix_token = node.SPARQL_PREFIX_token()?;

        match directive_style {
            TurtleDirectiveStyle::Sparql => {
                // Output as-is: PREFIX ns: <iri>
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
            TurtleDirectiveStyle::Turtle => {
                // Convert to Turtle style: @prefix ns: <iri> .
                write!(
                    f,
                    [
                        format_replaced(
                            &sparql_prefix_token,
                            &syntax_token_cow_slice(
                                Cow::Owned("@prefix".to_string()),
                                &sparql_prefix_token,
                                sparql_prefix_token.text_trimmed_range().start(),
                            ),
                        ),
                        space(),
                        node.namespace_token()?.format(),
                        space(),
                        node.iri_token()?.format(),
                        space(),
                        text(".", TextSize::default()),
                    ]
                )
            }
        }
    }
}
