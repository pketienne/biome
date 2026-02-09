use crate::FormatBogusNodeRule;
use biome_turtle_syntax::TurtleBogusStatement;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleBogusStatement;
impl FormatBogusNodeRule<TurtleBogusStatement> for FormatTurtleBogusStatement {}
