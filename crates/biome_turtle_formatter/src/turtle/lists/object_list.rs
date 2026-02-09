use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleObjectList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleObjectList;
impl FormatRule<TurtleObjectList> for FormatTurtleObjectList {
    type Context = TurtleFormatContext;
    fn fmt(&self, node: &TurtleObjectList, f: &mut TurtleFormatter) -> FormatResult<()> {
        let last_index = node.len().saturating_sub(1);

        for (index, element) in node.elements().enumerate() {
            let object = element.node()?;
            let separator = element.trailing_separator()?;

            write!(f, [object.format()])?;

            if let Some(token) = separator {
                if index == last_index {
                    write!(f, [format_removed(&token)])?;
                } else {
                    // comma after object, then next object on new indented line
                    write!(f, [token.format()])?;
                    write!(f, [indent(&biome_formatter::format_args!(hard_line_break()))])?;
                }
            }
        }

        Ok(())
    }
}
