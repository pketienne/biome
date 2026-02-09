use crate::FormatBogusNodeRule;
use biome_turtle_syntax::TurtleBogus;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleBogus;
impl FormatBogusNodeRule<TurtleBogus> for FormatTurtleBogus {}
