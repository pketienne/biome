use biome_rowan::{SendNode, TextRange};
use biome_turtle_syntax::TurtleRoot;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;

/// The facade for all semantic information of a Turtle document.
///
/// This struct provides access to prefix declarations, triple indices,
/// IRI references, and related semantic data. It holds a reference-counted
/// pointer to the internal `SemanticModelData`.
#[derive(Clone, Debug)]
pub struct SemanticModel {
    pub(crate) data: Arc<SemanticModelData>,
    root: SendNode,
}

impl SemanticModel {
    pub(crate) fn new(data: SemanticModelData, root: SendNode) -> Self {
        Self {
            data: Arc::new(data),
            root,
        }
    }

    pub fn root(&self) -> TurtleRoot {
        self.root.to_language_root::<TurtleRoot>()
    }

    // --- Prefix resolution ---

    /// Returns the prefix map (namespace → expansion IRI).
    pub fn prefix_map(&self) -> &FxHashMap<String, String> {
        &self.data.prefix_map
    }

    /// Resolve a prefix namespace to its expansion IRI.
    pub fn resolve_prefix(&self, namespace: &str) -> Option<&str> {
        self.data.prefix_map.get(namespace).map(|s| s.as_str())
    }

    /// Try to contract a full IRI to a prefixed name.
    pub fn contract_iri(&self, iri: &str) -> Option<String> {
        for (expansion, namespace) in &self.data.reverse_prefix_map {
            if let Some(local) = iri.strip_prefix(expansion.as_str()) {
                if !local.is_empty() {
                    return Some(format!("{namespace}{local}"));
                }
            }
        }
        None
    }

    /// Expand a prefixed name (e.g., `foaf:name`) to a full IRI.
    pub fn expand_prefixed_name(&self, prefixed: &str) -> Option<String> {
        let colon_pos = prefixed.find(':')?;
        let namespace = &prefixed[..=colon_pos];
        let local = &prefixed[colon_pos + 1..];
        let expansion = self.data.prefix_map.get(namespace)?;
        Some(format!("{expansion}{local}"))
    }

    /// Returns the base URI, if declared.
    pub fn base_uri(&self) -> Option<&str> {
        self.data.base_uri.as_deref()
    }

    // --- Prefix analysis ---

    /// Returns all prefix declarations in document order.
    pub fn prefix_declarations(&self) -> &[PrefixBinding] {
        &self.data.prefix_declarations
    }

    /// Returns an iterator over unused prefix declarations.
    pub fn unused_prefixes(&self) -> impl Iterator<Item = &PrefixBinding> {
        self.data
            .unused_prefixes
            .iter()
            .map(|&i| &self.data.prefix_declarations[i])
    }

    /// Returns an iterator over duplicate prefix declarations.
    pub fn duplicate_prefixes(&self) -> impl Iterator<Item = &PrefixBinding> {
        self.data
            .prefix_declarations
            .iter()
            .filter(|b| b.is_duplicate)
    }

    /// Check if a prefix namespace is used anywhere in the document.
    pub fn is_prefix_used(&self, namespace: &str) -> bool {
        self.data.used_prefixes.contains(namespace)
    }

    // --- Triple access ---

    /// Returns all triples in document order.
    pub fn triples(&self) -> &[TripleInfo] {
        &self.data.triples
    }

    /// Returns indices of triples with the given subject.
    pub fn triples_for_subject(&self, subject: &str) -> &[usize] {
        self.data
            .triples_by_subject
            .get(subject)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns pairs of (first_index, duplicate_index) for duplicate triples.
    pub fn duplicate_triples(&self) -> &[(usize, usize)] {
        &self.data.duplicate_triples
    }

    // --- IRI analysis ---

    /// Returns an iterator over IRI references that could be contracted to prefixed names.
    pub fn expandable_iris(&self) -> impl Iterator<Item = &IriRef> {
        self.data
            .expandable_iris
            .iter()
            .map(|&i| &self.data.iri_references[i])
    }

    /// Returns all prefix references in document order.
    pub fn prefix_references(&self) -> &[PrefixRef] {
        &self.data.prefix_references
    }

    /// Returns all IRI references in document order.
    pub fn iri_references(&self) -> &[IriRef] {
        &self.data.iri_references
    }
}

/// Internal data held by the semantic model.
#[derive(Debug)]
pub(crate) struct SemanticModelData {
    // Prefix resolution
    pub(crate) prefix_declarations: Vec<PrefixBinding>,
    pub(crate) prefix_map: FxHashMap<String, String>,
    pub(crate) reverse_prefix_map: FxHashMap<String, String>,
    pub(crate) base_uri: Option<String>,

    // Prefix usage tracking
    pub(crate) prefix_references: Vec<PrefixRef>,
    pub(crate) used_prefixes: FxHashSet<String>,
    pub(crate) unused_prefixes: Vec<usize>,

    // Triple index
    pub(crate) triples: Vec<TripleInfo>,
    pub(crate) triples_by_subject: FxHashMap<String, Vec<usize>>,
    pub(crate) duplicate_triples: Vec<(usize, usize)>,

    // IRI references
    pub(crate) iri_references: Vec<IriRef>,
    pub(crate) expandable_iris: Vec<usize>,
}

/// A prefix declaration in the document.
#[derive(Debug, Clone)]
pub struct PrefixBinding {
    pub namespace: String,
    pub expansion: String,
    pub range: TextRange,
    pub is_duplicate: bool,
}

/// A triple (subject, predicate, object) extracted from the document.
#[derive(Debug, Clone)]
pub struct TripleInfo {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub statement_range: TextRange,
    pub is_rdf_type: bool,
}

/// A prefixed name reference in the document.
#[derive(Debug, Clone)]
pub struct PrefixRef {
    pub namespace: String,
    pub local_name: String,
    pub range: TextRange,
}

/// A full IRI reference in the document.
#[derive(Debug, Clone)]
pub struct IriRef {
    pub iri: String,
    pub range: TextRange,
    pub suggested_prefixed: Option<String>,
}
