#![deny(clippy::use_self)]

use biome_turtle_syntax::TurtleLanguage;
use biome_rowan::TreeBuilder;

mod generated;
pub mod make;
pub use crate::generated::TurtleSyntaxFactory;

// Re-exported for tests
#[doc(hidden)]
pub use biome_turtle_syntax as syntax;

pub type TurtleSyntaxTreeBuilder = TreeBuilder<'static, TurtleLanguage, TurtleSyntaxFactory>;
