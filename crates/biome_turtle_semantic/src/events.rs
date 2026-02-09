use biome_rowan::{AstNode, AstSeparatedList, TextRange};
use biome_turtle_syntax::{
    TurtleBaseDeclaration, TurtlePrefixDeclaration, TurtlePrefixedName,
    TurtleSparqlBaseDeclaration, TurtleSparqlPrefixDeclaration, TurtleSyntaxKind, TurtleSyntaxNode,
    TurtleTriples,
};
use std::collections::VecDeque;

/// A semantic event emitted during AST traversal.
#[derive(Debug, Clone)]
pub enum SemanticEvent {
    /// A `@prefix` or `PREFIX` declaration.
    PrefixDeclaration {
        namespace: String,
        expansion: String,
        range: TextRange,
    },
    /// A `@base` or `BASE` declaration.
    BaseDeclaration {
        iri: String,
        range: TextRange,
    },
    /// A complete (subject, predicate, object) triple.
    Triple {
        subject: String,
        predicate: String,
        object: String,
        statement_range: TextRange,
    },
    /// A prefixed name reference (e.g., `foaf:name`).
    PrefixReference {
        namespace: String,
        local_name: String,
        range: TextRange,
    },
    /// A full IRI reference (e.g., `<http://...>`).
    IriReference {
        iri: String,
        range: TextRange,
    },
}

/// Extracts semantic events from a Turtle AST during a preorder walk.
#[derive(Debug, Default)]
pub struct SemanticEventExtractor {
    stash: VecDeque<SemanticEvent>,
}

impl SemanticEventExtractor {
    pub fn enter(&mut self, node: &TurtleSyntaxNode) {
        match node.kind() {
            TurtleSyntaxKind::TURTLE_PREFIX_DECLARATION => {
                self.enter_prefix_declaration(node);
            }
            TurtleSyntaxKind::TURTLE_SPARQL_PREFIX_DECLARATION => {
                self.enter_sparql_prefix_declaration(node);
            }
            TurtleSyntaxKind::TURTLE_BASE_DECLARATION => {
                self.enter_base_declaration(node);
            }
            TurtleSyntaxKind::TURTLE_SPARQL_BASE_DECLARATION => {
                self.enter_sparql_base_declaration(node);
            }
            TurtleSyntaxKind::TURTLE_TRIPLES => {
                self.enter_triples(node);
            }
            TurtleSyntaxKind::TURTLE_PREFIXED_NAME => {
                self.enter_prefixed_name(node);
            }
            TurtleSyntaxKind::TURTLE_IRI => {
                self.enter_iri(node);
            }
            _ => {}
        }
    }

    pub fn leave(&mut self, _node: &TurtleSyntaxNode) {
        // No cleanup needed for Turtle (unlike CSS which has rule nesting)
    }

    /// Pop the next event from the stash.
    pub fn pop(&mut self) -> Option<SemanticEvent> {
        self.stash.pop_front()
    }

    fn enter_prefix_declaration(&mut self, node: &TurtleSyntaxNode) {
        let Some(decl) = TurtlePrefixDeclaration::cast_ref(node) else {
            return;
        };
        let namespace = match decl.namespace_token() {
            Ok(t) => t.text_trimmed().to_string(),
            Err(_) => return,
        };
        let expansion = match decl.iri_token() {
            Ok(t) => {
                let text = t.text_trimmed();
                // Strip angle brackets from IRIREF
                text.strip_prefix('<')
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or(text)
                    .to_string()
            }
            Err(_) => return,
        };
        self.stash.push_back(SemanticEvent::PrefixDeclaration {
            namespace,
            expansion,
            range: node.text_trimmed_range(),
        });
    }

    fn enter_sparql_prefix_declaration(&mut self, node: &TurtleSyntaxNode) {
        let Some(decl) = TurtleSparqlPrefixDeclaration::cast_ref(node) else {
            return;
        };
        let namespace = match decl.namespace_token() {
            Ok(t) => t.text_trimmed().to_string(),
            Err(_) => return,
        };
        let expansion = match decl.iri_token() {
            Ok(t) => {
                let text = t.text_trimmed();
                text.strip_prefix('<')
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or(text)
                    .to_string()
            }
            Err(_) => return,
        };
        self.stash.push_back(SemanticEvent::PrefixDeclaration {
            namespace,
            expansion,
            range: node.text_trimmed_range(),
        });
    }

    fn enter_base_declaration(&mut self, node: &TurtleSyntaxNode) {
        let Some(decl) = TurtleBaseDeclaration::cast_ref(node) else {
            return;
        };
        let iri = match decl.iri_token() {
            Ok(t) => {
                let text = t.text_trimmed();
                text.strip_prefix('<')
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or(text)
                    .to_string()
            }
            Err(_) => return,
        };
        self.stash.push_back(SemanticEvent::BaseDeclaration {
            iri,
            range: node.text_trimmed_range(),
        });
    }

    fn enter_sparql_base_declaration(&mut self, node: &TurtleSyntaxNode) {
        let Some(decl) = TurtleSparqlBaseDeclaration::cast_ref(node) else {
            return;
        };
        let iri = match decl.iri_token() {
            Ok(t) => {
                let text = t.text_trimmed();
                text.strip_prefix('<')
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or(text)
                    .to_string()
            }
            Err(_) => return,
        };
        self.stash.push_back(SemanticEvent::BaseDeclaration {
            iri,
            range: node.text_trimmed_range(),
        });
    }

    fn enter_triples(&mut self, node: &TurtleSyntaxNode) {
        let Some(triples) = TurtleTriples::cast_ref(node) else {
            return;
        };
        let subject = match triples.subject() {
            Ok(s) => s.syntax().text_trimmed().to_string(),
            Err(_) => return,
        };
        let predicates = match triples.predicates() {
            Ok(p) => p,
            Err(_) => return,
        };
        let statement_range = node.text_trimmed_range();

        for element in predicates.pairs().elements() {
            let pair = match element.node() {
                Ok(p) => p.clone(),
                Err(_) => continue,
            };
            let verb = match pair.verb() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let predicate = verb.syntax().text_trimmed().to_string();

            for obj_element in pair.objects().elements() {
                let obj = match obj_element.node() {
                    Ok(o) => o.clone(),
                    Err(_) => continue,
                };
                let object = obj.syntax().text_trimmed().to_string();

                self.stash.push_back(SemanticEvent::Triple {
                    subject: subject.clone(),
                    predicate: predicate.clone(),
                    object,
                    statement_range,
                });
            }
        }
    }

    fn enter_prefixed_name(&mut self, node: &TurtleSyntaxNode) {
        let Some(prefixed_name) = TurtlePrefixedName::cast_ref(node) else {
            return;
        };
        let token = match prefixed_name.value() {
            Ok(t) => t,
            Err(_) => return,
        };
        let text = token.text_trimmed().to_string();
        let Some(colon_pos) = text.find(':') else {
            return;
        };
        let namespace = text[..=colon_pos].to_string();
        let local_name = text[colon_pos + 1..].to_string();
        self.stash.push_back(SemanticEvent::PrefixReference {
            namespace,
            local_name,
            range: node.text_trimmed_range(),
        });
    }

    fn enter_iri(&mut self, node: &TurtleSyntaxNode) {
        // Only emit IriReference for full IRIs (IRIREF tokens like <http://...>),
        // not for prefixed names (which are handled by enter_prefixed_name).
        for token in node.children_with_tokens() {
            if let Some(token) = token.as_token() {
                if token.kind() == TurtleSyntaxKind::TURTLE_IRIREF_LITERAL {
                    let text = token.text_trimmed();
                    let iri = text
                        .strip_prefix('<')
                        .and_then(|s| s.strip_suffix('>'))
                        .unwrap_or(text)
                        .to_string();
                    self.stash.push_back(SemanticEvent::IriReference {
                        iri,
                        range: node.text_trimmed_range(),
                    });
                    break;
                }
            }
        }
    }
}
