use std::rc::Rc;

use biome_rowan::TextRange;
use biome_yaml_syntax::YamlSyntaxNode;

use super::binding::Anchor;
use super::model::SemanticModelData;

/// Internal storage for an alias reference.
#[derive(Debug)]
pub(crate) struct AliasRef {
    pub name: String,
    pub range: TextRange,
    pub document_index: usize,
}

/// Public handle to an alias reference. Holds a reference to the model data.
pub struct Alias {
    pub(crate) data: Rc<SemanticModelData>,
    pub(crate) index: usize,
}

impl std::fmt::Debug for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alias_ref = &self.data.aliases[self.index];
        f.debug_struct("Alias")
            .field("name", &alias_ref.name)
            .field("range", &alias_ref.range)
            .field("document_index", &alias_ref.document_index)
            .finish()
    }
}

impl Alias {
    /// The bare alias name (without `*` prefix).
    pub fn name(&self) -> &str {
        &self.data.aliases[self.index].name
    }

    /// The text range of the alias token.
    pub fn range(&self) -> TextRange {
        self.data.aliases[self.index].range
    }

    /// The document index this alias belongs to.
    pub fn document_index(&self) -> usize {
        self.data.aliases[self.index].document_index
    }

    /// The syntax node for this alias, if it was recorded.
    pub fn syntax(&self) -> Option<&YamlSyntaxNode> {
        self.data.node_by_range.get(&self.range())
    }

    /// The anchor this alias references, if resolved.
    pub fn anchor(&self) -> Option<Anchor> {
        self.data.alias_to_anchor[self.index].map(|anchor_idx| Anchor {
            data: self.data.clone(),
            index: anchor_idx,
        })
    }
}

/// An alias that could not be resolved to any anchor.
#[derive(Debug)]
pub(crate) struct UnresolvedAliasData {
    pub name: String,
    pub range: TextRange,
    pub document_index: usize,
}

/// Public handle for an unresolved alias.
pub struct UnresolvedAlias {
    pub(crate) data: Rc<SemanticModelData>,
    pub(crate) id: usize,
}

impl std::fmt::Debug for UnresolvedAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unresolved = &self.data.unresolved_aliases[self.id];
        f.debug_struct("UnresolvedAlias")
            .field("name", &unresolved.name)
            .field("range", &unresolved.range)
            .finish()
    }
}

impl UnresolvedAlias {
    pub fn name(&self) -> &str {
        &self.data.unresolved_aliases[self.id].name
    }

    pub fn range(&self) -> TextRange {
        self.data.unresolved_aliases[self.id].range
    }

    pub fn document_index(&self) -> usize {
        self.data.unresolved_aliases[self.id].document_index
    }
}

/// A duplicate anchor (same name declared multiple times in the same document).
#[derive(Debug)]
pub(crate) struct DuplicateAnchorData {
    pub name: String,
    pub first_range: TextRange,
    pub duplicate_ranges: Vec<TextRange>,
    pub document_index: usize,
}

/// Public handle for a duplicate anchor.
pub struct DuplicateAnchor {
    pub(crate) data: Rc<SemanticModelData>,
    pub(crate) id: usize,
}

impl std::fmt::Debug for DuplicateAnchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dup = &self.data.duplicate_anchors[self.id];
        f.debug_struct("DuplicateAnchor")
            .field("name", &dup.name)
            .field("first_range", &dup.first_range)
            .field("duplicate_ranges", &dup.duplicate_ranges)
            .finish()
    }
}

impl DuplicateAnchor {
    pub fn name(&self) -> &str {
        &self.data.duplicate_anchors[self.id].name
    }

    pub fn first_range(&self) -> TextRange {
        self.data.duplicate_anchors[self.id].first_range
    }

    pub fn duplicate_ranges(&self) -> &[TextRange] {
        &self.data.duplicate_anchors[self.id].duplicate_ranges
    }

    pub fn document_index(&self) -> usize {
        self.data.duplicate_anchors[self.id].document_index
    }
}
