#![deny(clippy::use_self)]

mod events;
mod semantic_model;

pub use events::*;
pub use semantic_model::*;

use biome_rowan::{AstNode, WalkEvent};
use biome_turtle_syntax::TurtleRoot;

/// Build a semantic model from a parsed Turtle document.
///
/// This walks the AST once, extracting semantic events and processing
/// them into an indexed model that provides efficient queries for
/// prefix declarations, triple indices, IRI references, and more.
pub fn semantic_model(root: &TurtleRoot) -> SemanticModel {
    let mut extractor = SemanticEventExtractor::default();
    let mut builder = SemanticModelBuilder::new(root.clone());

    for node in root.syntax().preorder() {
        match node {
            WalkEvent::Enter(node) => extractor.enter(&node),
            WalkEvent::Leave(node) => extractor.leave(&node),
        }
    }

    while let Some(e) = extractor.pop() {
        builder.push_event(e);
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use biome_turtle_parser::parse_turtle;

    fn model_from(source: &str) -> SemanticModel {
        let parsed = parse_turtle(source);
        semantic_model(&parsed.tree())
    }

    #[test]
    fn collects_prefix_declarations() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix dc: <http://purl.org/dc/elements/1.1/> .
"#,
        );
        assert_eq!(model.prefix_declarations().len(), 2);
        // Namespace tokens include the trailing colon
        assert_eq!(
            model.resolve_prefix("foaf:"),
            Some("http://xmlns.com/foaf/0.1/")
        );
        assert_eq!(
            model.resolve_prefix("dc:"),
            Some("http://purl.org/dc/elements/1.1/")
        );
    }

    #[test]
    fn detects_duplicate_prefixes() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix dc: <http://purl.org/dc/elements/1.1/> .
"#,
        );
        let dups: Vec<_> = model.duplicate_prefixes().collect();
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].namespace, "foaf:");
    }

    #[test]
    fn tracks_prefix_usage() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix dc: <http://purl.org/dc/elements/1.1/> .

<http://example.org/alice> a foaf:Person .
"#,
        );
        assert!(model.is_prefix_used("foaf:"));
        assert!(!model.is_prefix_used("dc:"));

        let unused: Vec<_> = model.unused_prefixes().collect();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].namespace, "dc:");
    }

    #[test]
    fn collects_triples() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<http://example.org/alice> a foaf:Person ;
    foaf:name "Alice" .
"#,
        );
        assert_eq!(model.triples().len(), 2);
    }

    #[test]
    fn detects_duplicate_triples() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<http://example.org/alice> a foaf:Person .
<http://example.org/alice> a foaf:Person .
"#,
        );
        assert!(!model.duplicate_triples().is_empty());
    }

    #[test]
    fn collects_base_declaration() {
        let model = model_from(
            r#"
@base <http://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
"#,
        );
        assert_eq!(model.base_uri(), Some("http://example.org/"));
    }

    #[test]
    fn handles_sparql_style_prefix() {
        let model = model_from(
            r#"
PREFIX foaf: <http://xmlns.com/foaf/0.1/>
BASE <http://example.org/>
"#,
        );
        assert_eq!(
            model.resolve_prefix("foaf:"),
            Some("http://xmlns.com/foaf/0.1/")
        );
        assert_eq!(model.base_uri(), Some("http://example.org/"));
    }

    #[test]
    fn detects_expandable_iris() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<http://example.org/alice> a <http://xmlns.com/foaf/0.1/Person> .
"#,
        );
        let expandable: Vec<_> = model.expandable_iris().collect();
        assert_eq!(expandable.len(), 1);
        assert_eq!(
            expandable[0].suggested_prefixed.as_deref(),
            Some("foaf:Person")
        );
    }

    #[test]
    fn contract_iri() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
"#,
        );
        assert_eq!(
            model.contract_iri("http://xmlns.com/foaf/0.1/Person"),
            Some("foaf:Person".to_string())
        );
        assert_eq!(
            model.contract_iri("http://unknown.org/Thing"),
            None
        );
    }

    #[test]
    fn expand_prefixed_name() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
"#,
        );
        assert_eq!(
            model.expand_prefixed_name("foaf:Person"),
            Some("http://xmlns.com/foaf/0.1/Person".to_string())
        );
        assert_eq!(model.expand_prefixed_name("unknown:Thing"), None);
    }

    #[test]
    fn identifies_rdf_type_triples() {
        let model = model_from(
            r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<http://example.org/alice> rdf:type foaf:Person .
"#,
        );
        let rdf_type_triples: Vec<_> = model
            .triples()
            .iter()
            .filter(|t| t.is_rdf_type)
            .collect();
        assert_eq!(rdf_type_triples.len(), 1);
    }
}
