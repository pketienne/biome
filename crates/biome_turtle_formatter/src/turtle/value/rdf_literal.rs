use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleRdfLiteral;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleRdfLiteral;
impl FormatNodeRule<TurtleRdfLiteral> for FormatTurtleRdfLiteral {
    fn fmt_fields(&self, node: &TurtleRdfLiteral, f: &mut TurtleFormatter) -> FormatResult<()> {
        write!(f, [node.value()?.format()])?;
        if let Some(language_token) = node.language_token() {
            write!(f, [language_token.format()])?;
        }
        if let Some(datatype) = node.datatype() {
            write!(f, [datatype.format()])?;
        }
        Ok(())
    }
}
