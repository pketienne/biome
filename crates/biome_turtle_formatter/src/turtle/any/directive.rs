//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

use crate::prelude::*;
use biome_turtle_syntax::AnyTurtleDirective;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyTurtleDirective;
impl FormatRule<AnyTurtleDirective> for FormatAnyTurtleDirective {
    type Context = TurtleFormatContext;
    fn fmt(&self, node: &AnyTurtleDirective, f: &mut TurtleFormatter) -> FormatResult<()> {
        match node {
            AnyTurtleDirective::TurtleBaseDeclaration(node) => node.format().fmt(f),
            AnyTurtleDirective::TurtlePrefixDeclaration(node) => node.format().fmt(f),
            AnyTurtleDirective::TurtleSparqlBaseDeclaration(node) => node.format().fmt(f),
            AnyTurtleDirective::TurtleSparqlPrefixDeclaration(node) => node.format().fmt(f),
        }
    }
}
