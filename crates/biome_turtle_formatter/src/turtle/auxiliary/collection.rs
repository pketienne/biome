use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleCollection;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleCollection;
impl FormatNodeRule<TurtleCollection> for FormatTurtleCollection {
    fn fmt_fields(&self, node: &TurtleCollection, f: &mut TurtleFormatter) -> FormatResult<()> {
        let objects = node.objects();
        if objects.len() == 0 {
            // Empty collection: ()
            write!(
                f,
                [node.l_paren_token()?.format(), node.r_paren_token()?.format()]
            )
        } else {
            write!(
                f,
                [
                    node.l_paren_token()?.format(),
                    group(&soft_block_indent(&objects.format())),
                    node.r_paren_token()?.format(),
                ]
            )
        }
    }
}
