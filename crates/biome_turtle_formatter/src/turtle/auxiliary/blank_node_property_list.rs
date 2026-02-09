use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleBlankNodePropertyList;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleBlankNodePropertyList;
impl FormatNodeRule<TurtleBlankNodePropertyList> for FormatTurtleBlankNodePropertyList {
    fn fmt_fields(
        &self,
        node: &TurtleBlankNodePropertyList,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        let predicates = node.predicates()?;
        write!(
            f,
            [
                node.l_brack_token()?.format(),
                group(&soft_block_indent(&predicates.format())),
                node.r_brack_token()?.format(),
            ]
        )
    }
}
