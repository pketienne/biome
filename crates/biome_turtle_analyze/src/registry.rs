//! Generated file, do not edit by hand, see `xtask/codegen`

use biome_analyze::RegistryVisitor;
use biome_turtle_syntax::TurtleLanguage;
pub fn visit_registry<V: RegistryVisitor<TurtleLanguage>>(registry: &mut V) {
    registry.record_category::<crate::assist::Assist>();
    registry.record_category::<crate::lint::Lint>();
}
