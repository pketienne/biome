use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleDatatypeAnnotation;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleDatatypeAnnotation;
impl FormatNodeRule<TurtleDatatypeAnnotation> for FormatTurtleDatatypeAnnotation {
    fn fmt_fields(
        &self,
        node: &TurtleDatatypeAnnotation,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        write!(
            f,
            [node.caret_caret_token()?.format(), node.datatype()?.format()]
        )
    }
}
