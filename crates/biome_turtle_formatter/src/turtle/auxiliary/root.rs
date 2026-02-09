use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::{TurtleRoot, TurtleRootFields};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleRoot;
impl FormatNodeRule<TurtleRoot> for FormatTurtleRoot {
    fn fmt_fields(&self, node: &TurtleRoot, f: &mut TurtleFormatter) -> FormatResult<()> {
        let TurtleRootFields {
            bom_token,
            statements,
            eof_token,
        } = node.as_fields();

        write!(
            f,
            [
                bom_token.format(),
                statements.format(),
                hard_line_break(),
                format_removed(&eof_token?),
            ]
        )
    }
}
