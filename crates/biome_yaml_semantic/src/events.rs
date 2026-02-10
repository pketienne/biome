use biome_rowan::{AstNode, TextRange};
use biome_yaml_syntax::{YamlAliasNode, YamlAnchorProperty, YamlSyntaxKind, YamlSyntaxNode};
use std::collections::VecDeque;

/// Events emitted by the [SemanticEventExtractor].
/// These events are later consumed by the [SemanticModelBuilder].
#[derive(Debug, Eq, PartialEq)]
pub enum SemanticEvent {
    /// An anchor declaration (`&name`) was found.
    AnchorDeclaration {
        /// The bare anchor name (without `&` prefix).
        name: String,
        /// The text range of the anchor property literal token.
        range: TextRange,
        /// Which YAML document this anchor belongs to (0-indexed).
        document_index: usize,
    },
    /// An alias reference (`*name`) was found.
    AliasReference {
        /// The bare alias name (without `*` prefix).
        name: String,
        /// The text range of the alias literal token.
        range: TextRange,
        /// Which YAML document this alias belongs to (0-indexed).
        document_index: usize,
    },
}

impl SemanticEvent {
    pub fn range(&self) -> TextRange {
        match self {
            Self::AnchorDeclaration { range, .. } | Self::AliasReference { range, .. } => *range,
        }
    }
}

/// Extracts [SemanticEvent]s from a YAML syntax tree.
///
/// Push nodes during a preorder walk; pull events with `pop()`.
///
/// ```rust
/// use biome_yaml_parser::parse_yaml;
/// use biome_yaml_semantic::SemanticEventExtractor;
/// use biome_rowan::AstNode;
///
/// let parsed = parse_yaml("anchor: &a value\nalias: *a\n");
/// let mut extractor = SemanticEventExtractor::default();
/// for event in parsed.tree().syntax().preorder() {
///     match event {
///         biome_yaml_syntax::WalkEvent::Enter(node) => extractor.enter(&node),
///         biome_yaml_syntax::WalkEvent::Leave(node) => extractor.leave(&node),
///     }
///     while let Some(e) = extractor.pop() {
///         let _ = e;
///     }
/// }
/// ```
#[derive(Default, Debug)]
pub struct SemanticEventExtractor {
    stash: VecDeque<SemanticEvent>,
    /// Current YAML document index (increments on each YAML_DOCUMENT leave).
    document_count: usize,
    /// Whether we are currently inside a document.
    in_document: bool,
}

impl SemanticEventExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn enter(&mut self, node: &YamlSyntaxNode) {
        match node.kind() {
            YamlSyntaxKind::YAML_DOCUMENT => {
                self.in_document = true;
            }
            YamlSyntaxKind::YAML_ANCHOR_PROPERTY => {
                self.enter_anchor(node);
            }
            YamlSyntaxKind::YAML_ALIAS_NODE => {
                self.enter_alias(node);
            }
            _ => {}
        }
    }

    #[inline]
    pub fn leave(&mut self, node: &YamlSyntaxNode) {
        if node.kind() == YamlSyntaxKind::YAML_DOCUMENT {
            self.document_count += 1;
            self.in_document = false;
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<SemanticEvent> {
        self.stash.pop_front()
    }

    fn enter_anchor(&mut self, node: &YamlSyntaxNode) {
        let Some(anchor) = YamlAnchorProperty::cast(node.clone()) else {
            return;
        };
        let Ok(token) = anchor.value_token() else {
            return;
        };
        let text = token.text_trimmed();
        let name = text.strip_prefix('&').unwrap_or(text).to_string();
        let range = token.text_trimmed_range();
        self.stash.push_back(SemanticEvent::AnchorDeclaration {
            name,
            range,
            document_index: self.current_document_index(),
        });
    }

    fn enter_alias(&mut self, node: &YamlSyntaxNode) {
        let Some(alias) = YamlAliasNode::cast(node.clone()) else {
            return;
        };
        let Ok(token) = alias.value_token() else {
            return;
        };
        let text = token.text_trimmed();
        let name = text.strip_prefix('*').unwrap_or(text).to_string();
        let range = token.text_trimmed_range();
        self.stash.push_back(SemanticEvent::AliasReference {
            name,
            range,
            document_index: self.current_document_index(),
        });
    }

    fn current_document_index(&self) -> usize {
        self.document_count
    }
}
