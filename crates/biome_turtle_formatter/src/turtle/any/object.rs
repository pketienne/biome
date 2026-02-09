//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

use crate::prelude::*;
use biome_turtle_syntax::AnyTurtleObject;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyTurtleObject;
impl FormatRule<AnyTurtleObject> for FormatAnyTurtleObject {
    type Context = TurtleFormatContext;
    fn fmt(&self, node: &AnyTurtleObject, f: &mut TurtleFormatter) -> FormatResult<()> {
        match node {
            AnyTurtleObject::TurtleBlankNode(node) => node.format().fmt(f),
            AnyTurtleObject::TurtleBlankNodePropertyList(node) => node.format().fmt(f),
            AnyTurtleObject::TurtleBogus(node) => node.format().fmt(f),
            AnyTurtleObject::TurtleBooleanLiteral(node) => node.format().fmt(f),
            AnyTurtleObject::TurtleCollection(node) => node.format().fmt(f),
            AnyTurtleObject::TurtleIri(node) => node.format().fmt(f),
            AnyTurtleObject::TurtleNumericLiteral(node) => node.format().fmt(f),
            AnyTurtleObject::TurtleRdfLiteral(node) => node.format().fmt(f),
        }
    }
}
