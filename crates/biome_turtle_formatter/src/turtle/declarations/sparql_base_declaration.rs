use crate::prelude::*;
use biome_configuration::turtle::TurtleDirectiveStyle;
use biome_formatter::prelude::syntax_token_cow_slice;
use biome_formatter::write;
use biome_turtle_syntax::{TextSize, TurtleSparqlBaseDeclaration};
use std::borrow::Cow;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleSparqlBaseDeclaration;

impl FormatNodeRule<TurtleSparqlBaseDeclaration> for FormatTurtleSparqlBaseDeclaration {
    fn fmt_fields(
        &self,
        node: &TurtleSparqlBaseDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        let directive_style = f.options().directive_style();

        #[allow(non_snake_case)]
        let sparql_base_token = node.SPARQL_BASE_token()?;

        match directive_style {
            TurtleDirectiveStyle::Sparql => {
                // Output as-is: BASE <iri>
                write!(
                    f,
                    [
                        sparql_base_token.format(),
                        space(),
                        node.iri_token()?.format(),
                    ]
                )
            }
            TurtleDirectiveStyle::Turtle => {
                // Convert to Turtle style: @base <iri> .
                write!(
                    f,
                    [
                        format_replaced(
                            &sparql_base_token,
                            &syntax_token_cow_slice(
                                Cow::Owned("@base".to_string()),
                                &sparql_base_token,
                                sparql_base_token.text_trimmed_range().start(),
                            ),
                        ),
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
