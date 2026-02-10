use std::rc::Rc;

use biome_rowan::TextRange;
use biome_yaml_syntax::YamlSyntaxNode;

use super::model::SemanticModelData;
use super::reference::Alias;

/// Internal storage for an anchor binding.
#[derive(Debug)]
pub(crate) struct AnchorBinding {
    pub name: String,
    pub range: TextRange,
    pub document_index: usize,
}

/// Public handle to an anchor binding. Holds a reference to the model data.
pub struct Anchor {
    pub(crate) data: Rc<SemanticModelData>,
    pub(crate) index: usize,
}

impl std::fmt::Debug for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let binding = &self.data.anchors[self.index];
        f.debug_struct("Anchor")
            .field("name", &binding.name)
            .field("range", &binding.range)
            .field("document_index", &binding.document_index)
            .finish()
    }
}

impl Anchor {
    /// The bare anchor name (without `&` prefix).
    pub fn name(&self) -> &str {
        &self.data.anchors[self.index].name
    }

    /// The text range of the anchor token.
    pub fn range(&self) -> TextRange {
        self.data.anchors[self.index].range
    }

    /// The document index this anchor belongs to.
    pub fn document_index(&self) -> usize {
        self.data.anchors[self.index].document_index
    }

    /// The syntax node for this anchor, if it was recorded.
    pub fn syntax(&self) -> Option<&YamlSyntaxNode> {
        self.data.node_by_range.get(&self.range())
    }

    /// All aliases that reference this anchor.
    pub fn all_aliases(&self) -> Vec<Alias> {
        self.data.anchor_to_aliases[self.index]
            .iter()
            .map(|&alias_idx| Alias {
                data: self.data.clone(),
                index: alias_idx,
            })
            .collect()
    }
}
