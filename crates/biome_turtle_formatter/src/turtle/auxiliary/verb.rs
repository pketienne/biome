use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleVerb;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleVerb;
impl FormatNodeRule<TurtleVerb> for FormatTurtleVerb {
    fn fmt_fields(&self, node: &TurtleVerb, f: &mut TurtleFormatter) -> FormatResult<()> {
        if let Some(verb) = node.any_turtle_verb() {
            write!(f, [verb.format()])
        } else if let Some(a_token) = node.a_token_token() {
            write!(f, [a_token.format()])
        } else {
            Err(biome_formatter::FormatError::SyntaxError)
        }
    }
}
