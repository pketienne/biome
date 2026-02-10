mod binding;
mod builder;
mod model;
mod reference;

use biome_rowan::AstNode;
use biome_yaml_syntax::YamlRoot;

pub use binding::*;
pub use builder::*;
pub use model::*;
pub use reference::*;

use crate::SemanticEventExtractor;

/// Build the complete [SemanticModel] of a parsed YAML file.
///
/// Performs a single preorder traversal of the syntax tree to extract all
/// anchor declarations and alias references, then resolves their relationships.
///
/// ```rust
/// use biome_yaml_parser::parse_yaml;
/// use biome_yaml_semantic::semantic_model;
///
/// let parsed = parse_yaml("defaults: &defaults\n  timeout: 30\nserver:\n  <<: *defaults\n");
/// let model = semantic_model(&parsed.tree());
///
/// assert_eq!(model.all_anchors().count(), 1);
/// assert_eq!(model.all_aliases().count(), 1);
/// assert_eq!(model.all_unresolved_aliases().count(), 0);
/// ```
pub fn semantic_model(root: &YamlRoot) -> SemanticModel {
    let mut extractor = SemanticEventExtractor::default();
    let mut builder = SemanticModelBuilder::new(root.clone());

    let syntax = root.syntax();
    for node in syntax.preorder() {
        match node {
            biome_yaml_syntax::WalkEvent::Enter(node) => {
                builder.push_node(&node);
                extractor.enter(&node);
            }
            biome_yaml_syntax::WalkEvent::Leave(node) => extractor.leave(&node),
        }
    }

    while let Some(e) = extractor.pop() {
        builder.push_event(e);
    }

    builder.build()
}
