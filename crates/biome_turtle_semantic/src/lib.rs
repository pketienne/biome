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
    fn blank_node_as_subject() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

_:b1 foaf:name "Bob" .
"#,
        );
        assert_eq!(model.triples().len(), 1);
        assert_eq!(model.triples()[0].subject, "_:b1");
        assert!(!model.triples_for_subject("_:b1").is_empty());
    }

    #[test]
    fn blank_node_property_list() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<http://example.org/alice> foaf:knows [ foaf:name "Bob" ] .
"#,
        );
        // The semantic model extracts the outer triple; the blank node property list
        // is captured as the object text, not decomposed into separate triples.
        assert_eq!(model.triples().len(), 1);
        assert_eq!(
            model.triples()[0].subject,
            "<http://example.org/alice>"
        );
    }

    #[test]
    fn semicolon_notation_multiple_predicates() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix ex: <http://example.org/> .

ex:alice foaf:name "Alice" ;
    foaf:age "30" .
"#,
        );
        let triples = model.triples();
        assert_eq!(triples.len(), 2);
        assert_eq!(triples[0].subject, triples[1].subject);
    }

    #[test]
    fn comma_notation_object_list() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix ex: <http://example.org/> .

ex:alice foaf:knows ex:bob, ex:carol .
"#,
        );
        let triples = model.triples();
        assert_eq!(triples.len(), 2);
        assert_eq!(triples[0].subject, triples[1].subject);
        assert_eq!(triples[0].predicate, triples[1].predicate);
    }

    #[test]
    fn triples_for_subject_index() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix ex: <http://example.org/> .

ex:alice foaf:name "Alice" .
ex:bob foaf:name "Bob" .
ex:alice foaf:age "30" .
"#,
        );
        let alice_indices = model.triples_for_subject("ex:alice");
        assert_eq!(alice_indices.len(), 2);
        let bob_indices = model.triples_for_subject("ex:bob");
        assert_eq!(bob_indices.len(), 1);
        assert!(model.triples_for_subject("ex:unknown").is_empty());
    }

    #[test]
    fn empty_document() {
        let model = model_from("");
        assert!(model.triples().is_empty());
        assert!(model.prefix_declarations().is_empty());
        assert!(model.duplicate_triples().is_empty());
        assert_eq!(model.base_uri(), None);
    }

    #[test]
    fn prefix_references_tracking() {
        let model = model_from(
            r#"
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix ex: <http://example.org/> .

ex:alice a foaf:Person ;
    foaf:name "Alice" .
"#,
        );
        let refs = model.prefix_references();
        // Should have at least references for ex:alice, foaf:Person, foaf:name
        assert!(refs.len() >= 3);
        let foaf_refs: Vec<_> = refs.iter().filter(|r| r.namespace == "foaf:").collect();
        assert!(foaf_refs.len() >= 2); // foaf:Person, foaf:name
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

    // --- Stress tests ---

    #[test]
    fn stress_many_triples() {
        let mut source = String::from("@prefix ex: <http://example.org/> .\n");
        for i in 0..1000 {
            source.push_str(&format!("ex:s{i} ex:p ex:o{i} .\n"));
        }
        let model = model_from(&source);
        assert_eq!(model.triples().len(), 1000);
        // Spot-check triples_for_subject
        assert_eq!(model.triples_for_subject("ex:s0").len(), 1);
        assert_eq!(model.triples_for_subject("ex:s999").len(), 1);
    }

    #[test]
    fn stress_many_prefixes() {
        let mut source = String::new();
        for i in 0..100 {
            source.push_str(&format!(
                "@prefix p{i}: <http://example.org/ns{i}/> .\n"
            ));
        }
        // Use only the first prefix so the rest are unused
        source.push_str("p0:subject p0:predicate p0:object .\n");
        let model = model_from(&source);
        assert_eq!(model.prefix_declarations().len(), 100);
        let unused: Vec<_> = model.unused_prefixes().collect();
        assert_eq!(unused.len(), 99);
    }

    #[test]
    fn stress_deep_nested_blank_nodes() {
        let mut source = String::from("@prefix ex: <http://example.org/> .\n");
        source.push_str("ex:root ex:child ");
        for _ in 0..10 {
            source.push_str("[ ex:child ");
        }
        source.push_str("\"leaf\"");
        for _ in 0..10 {
            source.push_str(" ]");
        }
        source.push_str(" .\n");
        let model = model_from(&source);
        // Should not stack overflow; should produce at least 1 triple
        assert!(!model.triples().is_empty());
    }

    #[test]
    fn stress_large_object_list() {
        let mut source = String::from("@prefix ex: <http://example.org/> .\n");
        source.push_str("ex:s ex:p ");
        for i in 0..500 {
            if i > 0 {
                source.push_str(", ");
            }
            source.push_str(&format!("ex:o{i}"));
        }
        source.push_str(" .\n");
        let model = model_from(&source);
        assert_eq!(model.triples().len(), 500);
        // All triples share same subject and predicate
        assert!(model.triples().iter().all(|t| t.subject == "ex:s"));
        assert!(model.triples().iter().all(|t| t.predicate == "ex:p"));
    }

    #[test]
    fn stress_large_collection() {
        let mut source = String::from("@prefix ex: <http://example.org/> .\n");
        source.push_str("ex:s ex:list (");
        for i in 0..200 {
            source.push_str(&format!(" ex:item{i}"));
        }
        source.push_str(" ) .\n");
        let model = model_from(&source);
        // Should parse without error; at least 1 triple for the outer statement
        assert!(!model.triples().is_empty());
    }

    #[test]
    fn stress_duplicate_detection() {
        let mut source = String::from("@prefix ex: <http://example.org/> .\n");
        // 100 unique triples
        for i in 0..100 {
            source.push_str(&format!("ex:s{i} ex:p ex:o{i} .\n"));
        }
        // 50 exact duplicates of the first 50
        for i in 0..50 {
            source.push_str(&format!("ex:s{i} ex:p ex:o{i} .\n"));
        }
        let model = model_from(&source);
        assert_eq!(model.triples().len(), 150);
        assert_eq!(model.duplicate_triples().len(), 50);
    }
}
