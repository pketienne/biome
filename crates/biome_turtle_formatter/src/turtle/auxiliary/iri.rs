use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleIri;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleIri;
impl FormatNodeRule<TurtleIri> for FormatTurtleIri {
    fn fmt_fields(&self, node: &TurtleIri, f: &mut TurtleFormatter) -> FormatResult<()> {
        if let Some(value) = node.value() {
            write!(f, [value.format()])
        } else if let Some(iriref) = node.iriref_token() {
            write!(f, [iriref.format()])
        } else {
            Err(biome_formatter::FormatError::SyntaxError)
        }
    }
}
