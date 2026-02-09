use crate::prelude::*;
use biome_configuration::turtle::TurtleDirectiveStyle;
use biome_formatter::prelude::syntax_token_cow_slice;
use biome_formatter::write;
use biome_turtle_syntax::TurtlePrefixDeclaration;
use std::borrow::Cow;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtlePrefixDeclaration;

impl FormatNodeRule<TurtlePrefixDeclaration> for FormatTurtlePrefixDeclaration {
    fn fmt_fields(
        &self,
        node: &TurtlePrefixDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        let directive_style = f.options().directive_style();

        match directive_style {
            TurtleDirectiveStyle::Turtle => {
                // Output as-is: @prefix ns: <iri> .
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
            TurtleDirectiveStyle::Sparql => {
                // Convert to SPARQL style: PREFIX ns: <iri>
                let prefix_token = node.prefix_token()?;
                write!(
                    f,
                    [
                        format_replaced(
                            &prefix_token,
                            &syntax_token_cow_slice(
                                Cow::Owned("PREFIX".to_string()),
                                &prefix_token,
                                prefix_token.text_trimmed_range().start(),
                            ),
                        ),
                        space(),
                        node.namespace_token()?.format(),
                        space(),
                        node.iri_token()?.format(),
                    ]
                )
            }
        }
    }
}
