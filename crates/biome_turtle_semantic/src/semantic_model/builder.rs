use crate::events::SemanticEvent;
use crate::semantic_model::model::{
    IriRef, PrefixBinding, PrefixRef, SemanticModel, SemanticModelData, TripleInfo,
};
use biome_rowan::AstNode;
use biome_turtle_syntax::TurtleRoot;
use rustc_hash::{FxHashMap, FxHashSet};

/// Builds a `SemanticModel` from a sequence of `SemanticEvent`s.
pub struct SemanticModelBuilder {
    root: TurtleRoot,

    // Prefix data
    prefix_declarations: Vec<PrefixBinding>,
    prefix_map: FxHashMap<String, String>,
    reverse_prefix_map: FxHashMap<String, String>,
    seen_namespaces: FxHashSet<String>,
    base_uri: Option<String>,

    // Prefix usage
    prefix_references: Vec<PrefixRef>,
    used_prefixes: FxHashSet<String>,

    // Triple data
    triples: Vec<TripleInfo>,
    triples_by_subject: FxHashMap<String, Vec<usize>>,
    seen_triples: FxHashSet<(String, String, String)>,
    duplicate_triples: Vec<(usize, usize)>,

    // IRI data
    iri_references: Vec<IriRef>,
}

impl SemanticModelBuilder {
    pub fn new(root: TurtleRoot) -> Self {
        Self {
            root,
            prefix_declarations: Vec::new(),
            prefix_map: FxHashMap::default(),
            reverse_prefix_map: FxHashMap::default(),
            seen_namespaces: FxHashSet::default(),
            base_uri: None,
            prefix_references: Vec::new(),
            used_prefixes: FxHashSet::default(),
            triples: Vec::new(),
            triples_by_subject: FxHashMap::default(),
            seen_triples: FxHashSet::default(),
            duplicate_triples: Vec::new(),
            iri_references: Vec::new(),
        }
    }

    /// Process a single semantic event.
    pub fn push_event(&mut self, event: SemanticEvent) {
        match event {
            SemanticEvent::PrefixDeclaration {
                namespace,
                expansion,
                range,
            } => {
                let is_duplicate = self.seen_namespaces.contains(&namespace);
                self.prefix_declarations.push(PrefixBinding {
                    namespace: namespace.clone(),
                    expansion: expansion.clone(),
                    range,
                    is_duplicate,
                });
                if !is_duplicate {
                    self.seen_namespaces.insert(namespace.clone());
                    self.prefix_map
                        .insert(namespace.clone(), expansion.clone());
                    self.reverse_prefix_map.insert(expansion, namespace);
                }
            }
            SemanticEvent::BaseDeclaration { iri, range: _ } => {
                self.base_uri = Some(iri);
            }
            SemanticEvent::Triple {
                subject,
                predicate,
                object,
                statement_range,
            } => {
                let idx = self.triples.len();
                let key = (subject.clone(), predicate.clone(), object.clone());

                // Check for duplicates
                if !self.seen_triples.insert(key.clone()) {
                    // Find the first occurrence index
                    if let Some(first_idx) = self.triples.iter().position(|t| {
                        t.subject == subject && t.predicate == predicate && t.object == object
                    }) {
                        self.duplicate_triples.push((first_idx, idx));
                    }
                }

                let is_rdf_type = predicate == "a"
                    || predicate == "rdf:type"
                    || predicate == "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>";

                self.triples.push(TripleInfo {
                    subject: subject.clone(),
                    predicate,
                    object,
                    statement_range,
                    is_rdf_type,
                });

                self.triples_by_subject
                    .entry(subject)
                    .or_default()
                    .push(idx);
            }
            SemanticEvent::PrefixReference {
                namespace,
                local_name,
                range,
            } => {
                self.used_prefixes.insert(namespace.clone());
                self.prefix_references.push(PrefixRef {
                    namespace,
                    local_name,
                    range,
                });
            }
            SemanticEvent::IriReference { iri, range } => {
                // Check if this IRI could be contracted to a prefixed name
                let suggested_prefixed = self.try_contract_iri(&iri);
                self.iri_references.push(IriRef {
                    iri,
                    range,
                    suggested_prefixed,
                });
            }
        }
    }

    /// Try to contract an IRI to a prefixed name using the known prefix map.
    fn try_contract_iri(&self, iri: &str) -> Option<String> {
        for (expansion, namespace) in &self.reverse_prefix_map {
            if let Some(local) = iri.strip_prefix(expansion.as_str()) {
                // Only contract if the local part is a valid local name
                // (no special characters that would break prefixed name syntax)
                if !local.is_empty() && is_valid_local_name(local) {
                    return Some(format!("{namespace}{local}"));
                }
            }
        }
        None
    }

    /// Build the final `SemanticModel`.
    pub fn build(self) -> SemanticModel {
        // Compute unused prefix indices
        let unused_prefixes: Vec<usize> = self
            .prefix_declarations
            .iter()
            .enumerate()
            .filter(|(_, binding)| {
                !binding.is_duplicate && !self.used_prefixes.contains(&binding.namespace)
            })
            .map(|(i, _)| i)
            .collect();

        // Compute expandable IRI indices
        let expandable_iris: Vec<usize> = self
            .iri_references
            .iter()
            .enumerate()
            .filter(|(_, iri_ref)| iri_ref.suggested_prefixed.is_some())
            .map(|(i, _)| i)
            .collect();

        let data = SemanticModelData {
            prefix_declarations: self.prefix_declarations,
            prefix_map: self.prefix_map,
            reverse_prefix_map: self.reverse_prefix_map,
            base_uri: self.base_uri,
            prefix_references: self.prefix_references,
            used_prefixes: self.used_prefixes,
            unused_prefixes,
            triples: self.triples,
            triples_by_subject: self.triples_by_subject,
            duplicate_triples: self.duplicate_triples,
            iri_references: self.iri_references,
            expandable_iris,
        };

        let root = self.root.syntax().as_send().expect("TurtleRoot should be a root node");
        SemanticModel::new(data, root)
    }
}

/// Check if a string is a valid Turtle local name (simplified check).
fn is_valid_local_name(s: &str) -> bool {
    // A local name shouldn't contain characters that need escaping
    // This is a simplified check — the full PN_LOCAL production is complex
    !s.contains(' ')
        && !s.contains('<')
        && !s.contains('>')
        && !s.contains('"')
        && !s.contains('{')
        && !s.contains('}')
        && !s.contains('|')
        && !s.contains('^')
        && !s.contains('`')
        && !s.contains('\\')
}
