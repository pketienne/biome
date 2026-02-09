//! Extremely fast, lossless, and error tolerant Turtle (RDF) Parser.

#![deny(clippy::use_self)]

use biome_turtle_factory::TurtleSyntaxFactory;
use biome_turtle_syntax::{TurtleLanguage, TurtleRoot, TurtleSyntaxNode};
pub use biome_parser::prelude::*;
use biome_parser::{AnyParse, NodeParse};
use biome_rowan::{AstNode, NodeCache};
use parser::{TurtleParser, parse_root};

mod lexer;
mod parser;
mod token_source;

pub(crate) type TurtleLosslessTreeSink<'source> =
    LosslessTreeSink<'source, TurtleLanguage, TurtleSyntaxFactory>;

pub fn parse_turtle(source: &str) -> TurtleParse {
    let mut cache = NodeCache::default();
    parse_turtle_with_cache(source, &mut cache)
}

/// Parses the provided string as a Turtle document using the provided node cache.
pub fn parse_turtle_with_cache(source: &str, cache: &mut NodeCache) -> TurtleParse {
    let mut parser = TurtleParser::new(source);

    parse_root(&mut parser);

    let (events, diagnostics, trivia) = parser.finish();

    let mut tree_sink = TurtleLosslessTreeSink::with_cache(source, &trivia, cache);
    biome_parser::event::process(&mut tree_sink, events, diagnostics);
    let (green, diagnostics) = tree_sink.finish();

    TurtleParse::new(green, diagnostics)
}

/// A utility struct for managing the result of a parser job
#[derive(Debug)]
pub struct TurtleParse {
    root: TurtleSyntaxNode,
    diagnostics: Vec<ParseDiagnostic>,
}

impl TurtleParse {
    pub fn new(root: TurtleSyntaxNode, diagnostics: Vec<ParseDiagnostic>) -> Self {
        Self { root, diagnostics }
    }

    /// The syntax node represented by this Parse result
    pub fn syntax(&self) -> TurtleSyntaxNode {
        self.root.clone()
    }

    /// Get the diagnostics which occurred when parsing
    pub fn diagnostics(&self) -> &[ParseDiagnostic] {
        &self.diagnostics
    }

    /// Get the diagnostics which occurred when parsing
    pub fn into_diagnostics(self) -> Vec<ParseDiagnostic> {
        self.diagnostics
    }

    /// Returns [true] if the parser encountered some errors during the parsing.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error())
    }

    /// Convert this parse result into a typed AST node.
    ///
    /// # Panics
    /// Panics if the node represented by this parse result mismatches.
    pub fn tree(&self) -> TurtleRoot {
        TurtleRoot::unwrap_cast(self.syntax())
    }
}

impl From<TurtleParse> for AnyParse {
    fn from(parse: TurtleParse) -> Self {
        let root = parse.syntax();
        let diagnostics = parse.into_diagnostics();
        NodeParse::new(
            // SAFETY: the parser should always return a root node
            root.as_send().unwrap(),
            diagnostics,
        )
        .into()
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_turtle;

    #[test]
    fn parser_smoke_test() {
        let src = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<http://example.org/alice> a foaf:Person ;
    foaf:name "Alice" .
"#;

        let result = parse_turtle(src);
        assert!(!result.has_errors(), "Expected no errors, got: {:?}", result.diagnostics());
    }

    #[test]
    fn parser_a_keyword_test() {
        let src = "ex:bob a ex:Person .";

        let result = parse_turtle(src);
        assert!(!result.has_errors(), "Expected no errors, got: {:?}", result.diagnostics());

        // Verify the tree structure contains TURTLE_TRIPLES (not TURTLE_BOGUS_STATEMENT)
        let tree_str = format!("{:#?}", result.syntax());
        assert!(
            tree_str.contains("TURTLE_TRIPLES"),
            "Expected TURTLE_TRIPLES in AST, got:\n{tree_str}"
        );
        assert!(
            !tree_str.contains("TURTLE_BOGUS"),
            "Expected no TURTLE_BOGUS in AST, got:\n{tree_str}"
        );
    }
}
