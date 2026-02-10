use biome_rowan::{TextRange, TextSize};
use biome_yaml_syntax::{YamlRoot, YamlSyntaxKind, YamlSyntaxNode};
use rustc_hash::FxHashMap;

use crate::SemanticEvent;

use super::binding::AnchorBinding;
use super::model::{SemanticModel, SemanticModelData};
use super::reference::{AliasRef, DuplicateAnchorData, UnresolvedAliasData};

/// Builds the [SemanticModel] by consuming [SemanticEvent]s and [YamlSyntaxNode]s.
pub struct SemanticModelBuilder {
    root: YamlRoot,
    node_by_range: FxHashMap<TextRange, YamlSyntaxNode>,

    // Collected during event processing
    anchors: Vec<AnchorBinding>,
    anchors_by_start: FxHashMap<TextSize, usize>,
    /// Map (name, document_index) → first anchor index for that name in that document
    anchors_by_name_doc: FxHashMap<(String, usize), usize>,
    /// Track duplicate anchor ranges per (name, document_index)
    duplicate_ranges: FxHashMap<(String, usize), Vec<TextRange>>,

    aliases: Vec<AliasRef>,

    // Built during `build()`
    anchor_to_aliases: Vec<Vec<usize>>,
    alias_to_anchor: Vec<Option<usize>>,
}

impl SemanticModelBuilder {
    pub fn new(root: YamlRoot) -> Self {
        Self {
            root,
            node_by_range: FxHashMap::default(),
            anchors: Vec::new(),
            anchors_by_start: FxHashMap::default(),
            anchors_by_name_doc: FxHashMap::default(),
            duplicate_ranges: FxHashMap::default(),
            aliases: Vec::new(),
            anchor_to_aliases: Vec::new(),
            alias_to_anchor: Vec::new(),
        }
    }

    /// Record a syntax node for later lookup by range.
    #[inline]
    pub fn push_node(&mut self, node: &YamlSyntaxNode) {
        if matches!(
            node.kind(),
            YamlSyntaxKind::YAML_ANCHOR_PROPERTY | YamlSyntaxKind::YAML_ALIAS_NODE
        ) {
            self.node_by_range
                .insert(node.text_trimmed_range(), node.clone());
        }
    }

    /// Process a semantic event.
    #[inline]
    pub fn push_event(&mut self, event: SemanticEvent) {
        match event {
            SemanticEvent::AnchorDeclaration {
                name,
                range,
                document_index,
            } => {
                let anchor_id = self.anchors.len();
                let key = (name.clone(), document_index);

                if self.anchors_by_name_doc.contains_key(&key) {
                    // Duplicate anchor — record the duplicate range
                    self.duplicate_ranges
                        .entry(key)
                        .or_default()
                        .push(range);
                } else {
                    self.anchors_by_name_doc.insert(key, anchor_id);
                }

                self.anchors.push(AnchorBinding {
                    name,
                    range,
                    document_index,
                });
                self.anchors_by_start.insert(range.start(), anchor_id);
                self.anchor_to_aliases.push(Vec::new());
            }
            SemanticEvent::AliasReference {
                name,
                range,
                document_index,
            } => {
                self.aliases.push(AliasRef {
                    name,
                    range,
                    document_index,
                });
                self.alias_to_anchor.push(None);
            }
        }
    }

    /// Resolve all alias → anchor relationships and build the final model.
    pub fn build(mut self) -> SemanticModel {
        let mut unresolved_aliases = Vec::new();

        // Resolve each alias to its anchor within the same document
        for alias_idx in 0..self.aliases.len() {
            let alias = &self.aliases[alias_idx];
            let key = (alias.name.clone(), alias.document_index);

            if let Some(&anchor_idx) = self.anchors_by_name_doc.get(&key) {
                self.alias_to_anchor[alias_idx] = Some(anchor_idx);
                self.anchor_to_aliases[anchor_idx].push(alias_idx);
            } else {
                unresolved_aliases.push(UnresolvedAliasData {
                    name: alias.name.clone(),
                    range: alias.range,
                    document_index: alias.document_index,
                });
            }
        }

        // Build duplicate anchor data
        let mut duplicate_anchors = Vec::new();
        for ((name, document_index), dup_ranges) in &self.duplicate_ranges {
            let key = (name.clone(), *document_index);
            if let Some(&first_anchor_idx) = self.anchors_by_name_doc.get(&key) {
                let first_range = self.anchors[first_anchor_idx].range;
                duplicate_anchors.push(DuplicateAnchorData {
                    name: name.clone(),
                    first_range,
                    duplicate_ranges: dup_ranges.clone(),
                    document_index: *document_index,
                });
            }
        }

        let data = SemanticModelData {
            root: self.root,
            node_by_range: self.node_by_range,
            anchors: self.anchors,
            anchors_by_start: self.anchors_by_start,
            aliases: self.aliases,
            anchor_to_aliases: self.anchor_to_aliases,
            alias_to_anchor: self.alias_to_anchor,
            unresolved_aliases,
            duplicate_anchors,
        };
        SemanticModel::new(data)
    }
}
