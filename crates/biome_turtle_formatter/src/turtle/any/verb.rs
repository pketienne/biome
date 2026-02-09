//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

use crate::prelude::*;
use biome_turtle_syntax::AnyTurtleVerb;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyTurtleVerb;
impl FormatRule<AnyTurtleVerb> for FormatAnyTurtleVerb {
    type Context = TurtleFormatContext;
    fn fmt(&self, node: &AnyTurtleVerb, f: &mut TurtleFormatter) -> FormatResult<()> {
        match node {
            AnyTurtleVerb::TurtleBogus(node) => node.format().fmt(f),
            AnyTurtleVerb::TurtleIri(node) => node.format().fmt(f),
        }
    }
}
