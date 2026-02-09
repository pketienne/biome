//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

use crate::prelude::*;
use biome_turtle_syntax::AnyTurtleIriValue;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyTurtleIriValue;
impl FormatRule<AnyTurtleIriValue> for FormatAnyTurtleIriValue {
    type Context = TurtleFormatContext;
    fn fmt(&self, node: &AnyTurtleIriValue, f: &mut TurtleFormatter) -> FormatResult<()> {
        match node {
            AnyTurtleIriValue::TurtleBogus(node) => node.format().fmt(f),
            AnyTurtleIriValue::TurtlePrefixedName(node) => node.format().fmt(f),
        }
    }
}
