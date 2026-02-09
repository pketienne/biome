use crate::prelude::*;
use biome_turtle_syntax::TurtleCollectionObjectList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleCollectionObjectList;
impl FormatRule<TurtleCollectionObjectList> for FormatTurtleCollectionObjectList {
    type Context = TurtleFormatContext;
    fn fmt(&self, node: &TurtleCollectionObjectList, f: &mut TurtleFormatter) -> FormatResult<()> {
        let separator = soft_line_break_or_space();
        let mut join = f.join_with(&separator);

        for object in node {
            join.entry(&format_or_verbatim(object.format()));
        }

        join.finish()
    }
}
