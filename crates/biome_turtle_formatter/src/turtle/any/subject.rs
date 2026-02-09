//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

use crate::prelude::*;
use biome_turtle_syntax::AnyTurtleSubject;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyTurtleSubject;
impl FormatRule<AnyTurtleSubject> for FormatAnyTurtleSubject {
    type Context = TurtleFormatContext;
    fn fmt(&self, node: &AnyTurtleSubject, f: &mut TurtleFormatter) -> FormatResult<()> {
        match node {
            AnyTurtleSubject::TurtleBlankNode(node) => node.format().fmt(f),
            AnyTurtleSubject::TurtleBlankNodePropertyList(node) => node.format().fmt(f),
            AnyTurtleSubject::TurtleCollection(node) => node.format().fmt(f),
            AnyTurtleSubject::TurtleIri(node) => node.format().fmt(f),
        }
    }
}
