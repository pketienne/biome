use crate::prelude::*;
use biome_configuration::turtle::TurtleDirectiveStyle;
use biome_formatter::prelude::syntax_token_cow_slice;
use biome_formatter::write;
use biome_turtle_syntax::TurtleBaseDeclaration;
use std::borrow::Cow;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleBaseDeclaration;

impl FormatNodeRule<TurtleBaseDeclaration> for FormatTurtleBaseDeclaration {
    fn fmt_fields(
        &self,
        node: &TurtleBaseDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        let directive_style = f.options().directive_style();

        match directive_style {
            TurtleDirectiveStyle::Turtle => {
                // Output as-is: @base <iri> .
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
            TurtleDirectiveStyle::Sparql => {
                // Convert to SPARQL style: BASE <iri>
                let base_token = node.base_token()?;
                write!(
                    f,
                    [
                        format_replaced(
                            &base_token,
                            &syntax_token_cow_slice(
                                Cow::Owned("BASE".to_string()),
                                &base_token,
                                base_token.text_trimmed_range().start(),
                            ),
                        ),
                        space(),
                        node.iri_token()?.format(),
                    ]
                )
            }
        }
    }
}
