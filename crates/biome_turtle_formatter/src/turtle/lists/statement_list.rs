use crate::prelude::*;
use biome_turtle_syntax::TurtleStatementList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleStatementList;
impl FormatRule<TurtleStatementList> for FormatTurtleStatementList {
    type Context = TurtleFormatContext;
    fn fmt(&self, node: &TurtleStatementList, f: &mut TurtleFormatter) -> FormatResult<()> {
        let mut join = f.join_nodes_with_hardline();

        for statement in node {
            // format each statement with verbatim fallback
            join.entry(
                statement.syntax(),
                &format_or_verbatim(statement.format()),
            );
        }

        join.finish()
    }
}
