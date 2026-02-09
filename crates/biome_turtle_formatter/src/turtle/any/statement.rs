//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

use crate::prelude::*;
use biome_turtle_syntax::AnyTurtleStatement;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyTurtleStatement;
impl FormatRule<AnyTurtleStatement> for FormatAnyTurtleStatement {
    type Context = TurtleFormatContext;
    fn fmt(&self, node: &AnyTurtleStatement, f: &mut TurtleFormatter) -> FormatResult<()> {
        match node {
            AnyTurtleStatement::AnyTurtleDirective(node) => node.format().fmt(f),
            AnyTurtleStatement::TurtleBogusStatement(node) => node.format().fmt(f),
            AnyTurtleStatement::TurtleTriples(node) => node.format().fmt(f),
        }
    }
}
