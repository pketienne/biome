use std::rc::Rc;

use biome_rowan::{AstNode, TextRange, TextSize};
use biome_yaml_syntax::{YamlAnchorProperty, YamlRoot, YamlSyntaxNode};
use rustc_hash::FxHashMap;

use super::binding::{Anchor, AnchorBinding};
use super::reference::{
    Alias, AliasRef, DuplicateAnchor, DuplicateAnchorData, UnresolvedAlias, UnresolvedAliasData,
};

/// All semantic data for a YAML file. Lives behind `Rc` so handles can outlive the model.
#[derive(Debug)]
pub(crate) struct SemanticModelData {
    pub root: YamlRoot,
    pub node_by_range: FxHashMap<TextRange, YamlSyntaxNode>,

    // Anchors
    pub anchors: Vec<AnchorBinding>,
    pub anchors_by_start: FxHashMap<TextSize, usize>,

    // Aliases
    pub aliases: Vec<AliasRef>,

    // Anchor → aliases mapping
    pub anchor_to_aliases: Vec<Vec<usize>>,
    // Alias → anchor mapping (None if unresolved)
    pub alias_to_anchor: Vec<Option<usize>>,

    // Pre-computed diagnostics
    pub unresolved_aliases: Vec<UnresolvedAliasData>,
    pub duplicate_anchors: Vec<DuplicateAnchorData>,
}

impl PartialEq for SemanticModelData {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}

impl Eq for SemanticModelData {}

/// The facade for all YAML semantic information: anchors, aliases, and their relationships.
#[derive(Clone, Debug)]
pub struct SemanticModel {
    pub(crate) data: Rc<SemanticModelData>,
}

impl SemanticModel {
    pub(crate) fn new(data: SemanticModelData) -> Self {
        Self {
            data: Rc::new(data),
        }
    }

    /// Returns an iterator over all anchor bindings.
    pub fn all_anchors(&self) -> impl Iterator<Item = Anchor> + '_ {
        (0..self.data.anchors.len()).map(move |i| Anchor {
            data: self.data.clone(),
            index: i,
        })
    }

    /// Returns an iterator over all alias references.
    pub fn all_aliases(&self) -> impl Iterator<Item = Alias> + '_ {
        (0..self.data.aliases.len()).map(move |i| Alias {
            data: self.data.clone(),
            index: i,
        })
    }

    /// Look up an anchor by its AST node.
    pub fn as_anchor(&self, node: &YamlAnchorProperty) -> Option<Anchor> {
        let range = node.syntax().text_trimmed_range();
        let &idx = self.data.anchors_by_start.get(&range.start())?;
        Some(Anchor {
            data: self.data.clone(),
            index: idx,
        })
    }

    /// All aliases that could not be resolved to an anchor.
    pub fn all_unresolved_aliases(&self) -> impl Iterator<Item = UnresolvedAlias> + '_ {
        (0..self.data.unresolved_aliases.len()).map(move |i| UnresolvedAlias {
            data: self.data.clone(),
            id: i,
        })
    }

    /// All anchors that have duplicate declarations in the same document.
    pub fn all_duplicate_anchors(&self) -> impl Iterator<Item = DuplicateAnchor> + '_ {
        (0..self.data.duplicate_anchors.len()).map(move |i| DuplicateAnchor {
            data: self.data.clone(),
            id: i,
        })
    }
}
