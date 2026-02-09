use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtlePredicateObjectPairList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtlePredicateObjectPairList;
impl FormatRule<TurtlePredicateObjectPairList> for FormatTurtlePredicateObjectPairList {
    type Context = TurtleFormatContext;
    fn fmt(
        &self,
        node: &TurtlePredicateObjectPairList,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        let last_index = node.len().saturating_sub(1);

        for (index, element) in node.elements().enumerate() {
            let pair = element.node()?;
            let separator = element.trailing_separator()?;

            if index > 0 {
                write!(f, [hard_line_break()])?;
            }

            write!(f, [pair.format()])?;

            if let Some(token) = separator {
                if index == last_index {
                    // Remove trailing semicolons after the last pair
                    write!(f, [format_removed(&token)])?;
                } else {
                    // Keep semicolons between pairs (space before `;`)
                    write!(f, [space(), token.format()])?;
                }
            }
        }

        Ok(())
    }
}
