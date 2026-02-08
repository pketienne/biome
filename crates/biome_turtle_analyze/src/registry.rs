use biome_analyze::RegistryVisitor;
use biome_turtle_syntax::TurtleLanguage;

pub fn visit_registry<V: RegistryVisitor<TurtleLanguage>>(_registry: &mut V) {
    // No rules registered yet
}
