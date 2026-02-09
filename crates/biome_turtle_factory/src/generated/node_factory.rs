//! Generated file, do not edit by hand, see `xtask/codegen`

#![allow(clippy::redundant_closure)]
use biome_rowan::AstNode;
use biome_turtle_syntax::{
    TurtleSyntaxElement as SyntaxElement, TurtleSyntaxNode as SyntaxNode,
    TurtleSyntaxToken as SyntaxToken, *,
};
pub fn turtle_base_declaration(
    base_token: SyntaxToken,
    iri_token: SyntaxToken,
    dot_token: SyntaxToken,
) -> TurtleBaseDeclaration {
    TurtleBaseDeclaration::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_BASE_DECLARATION,
        [
            Some(SyntaxElement::Token(base_token)),
            Some(SyntaxElement::Token(iri_token)),
            Some(SyntaxElement::Token(dot_token)),
        ],
    ))
}
pub fn turtle_blank_node(value_token: SyntaxToken) -> TurtleBlankNode {
    TurtleBlankNode::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_BLANK_NODE,
        [Some(SyntaxElement::Token(value_token))],
    ))
}
pub fn turtle_blank_node_property_list(
    l_brack_token: SyntaxToken,
    predicates: TurtlePredicateObjectList,
    r_brack_token: SyntaxToken,
) -> TurtleBlankNodePropertyList {
    TurtleBlankNodePropertyList::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_BLANK_NODE_PROPERTY_LIST,
        [
            Some(SyntaxElement::Token(l_brack_token)),
            Some(SyntaxElement::Node(predicates.into_syntax())),
            Some(SyntaxElement::Token(r_brack_token)),
        ],
    ))
}
pub fn turtle_boolean_literal(value_token_token: SyntaxToken) -> TurtleBooleanLiteral {
    TurtleBooleanLiteral::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_BOOLEAN_LITERAL,
        [Some(SyntaxElement::Token(value_token_token))],
    ))
}
pub fn turtle_collection(
    l_paren_token: SyntaxToken,
    objects: TurtleCollectionObjectList,
    r_paren_token: SyntaxToken,
) -> TurtleCollection {
    TurtleCollection::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_COLLECTION,
        [
            Some(SyntaxElement::Token(l_paren_token)),
            Some(SyntaxElement::Node(objects.into_syntax())),
            Some(SyntaxElement::Token(r_paren_token)),
        ],
    ))
}
pub fn turtle_datatype_annotation(
    caret_caret_token: SyntaxToken,
    datatype: TurtleIri,
) -> TurtleDatatypeAnnotation {
    TurtleDatatypeAnnotation::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_DATATYPE_ANNOTATION,
        [
            Some(SyntaxElement::Token(caret_caret_token)),
            Some(SyntaxElement::Node(datatype.into_syntax())),
        ],
    ))
}
pub fn turtle_iri() -> TurtleIriBuilder {
    TurtleIriBuilder {
        value: None,
        iriref_token: None,
    }
}
pub struct TurtleIriBuilder {
    value: Option<AnyTurtleIriValue>,
    iriref_token: Option<SyntaxToken>,
}
impl TurtleIriBuilder {
    pub fn with_value(mut self, value: AnyTurtleIriValue) -> Self {
        self.value = Some(value);
        self
    }
    pub fn with_iriref_token(mut self, iriref_token: SyntaxToken) -> Self {
        self.iriref_token = Some(iriref_token);
        self
    }
    pub fn build(self) -> TurtleIri {
        TurtleIri::unwrap_cast(SyntaxNode::new_detached(
            TurtleSyntaxKind::TURTLE_IRI,
            [
                self.value
                    .map(|token| SyntaxElement::Node(token.into_syntax())),
                self.iriref_token.map(|token| SyntaxElement::Token(token)),
            ],
        ))
    }
}
pub fn turtle_numeric_literal(value_token: SyntaxToken) -> TurtleNumericLiteral {
    TurtleNumericLiteral::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_NUMERIC_LITERAL,
        [Some(SyntaxElement::Token(value_token))],
    ))
}
pub fn turtle_object(any_turtle_object: AnyTurtleObject) -> TurtleObject {
    TurtleObject::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_OBJECT,
        [Some(SyntaxElement::Node(any_turtle_object.into_syntax()))],
    ))
}
pub fn turtle_predicate_object_list(
    pairs: TurtlePredicateObjectPairList,
) -> TurtlePredicateObjectList {
    TurtlePredicateObjectList::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_PREDICATE_OBJECT_LIST,
        [Some(SyntaxElement::Node(pairs.into_syntax()))],
    ))
}
pub fn turtle_predicate_object_pair(
    verb: TurtleVerb,
    objects: TurtleObjectList,
) -> TurtlePredicateObjectPair {
    TurtlePredicateObjectPair::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_PREDICATE_OBJECT_PAIR,
        [
            Some(SyntaxElement::Node(verb.into_syntax())),
            Some(SyntaxElement::Node(objects.into_syntax())),
        ],
    ))
}
pub fn turtle_prefix_declaration(
    prefix_token: SyntaxToken,
    namespace_token: SyntaxToken,
    iri_token: SyntaxToken,
    dot_token: SyntaxToken,
) -> TurtlePrefixDeclaration {
    TurtlePrefixDeclaration::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_PREFIX_DECLARATION,
        [
            Some(SyntaxElement::Token(prefix_token)),
            Some(SyntaxElement::Token(namespace_token)),
            Some(SyntaxElement::Token(iri_token)),
            Some(SyntaxElement::Token(dot_token)),
        ],
    ))
}
pub fn turtle_prefixed_name(value_token: SyntaxToken) -> TurtlePrefixedName {
    TurtlePrefixedName::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_PREFIXED_NAME,
        [Some(SyntaxElement::Token(value_token))],
    ))
}
pub fn turtle_rdf_literal(value: TurtleString) -> TurtleRdfLiteralBuilder {
    TurtleRdfLiteralBuilder {
        value,
        language_token: None,
        datatype: None,
    }
}
pub struct TurtleRdfLiteralBuilder {
    value: TurtleString,
    language_token: Option<SyntaxToken>,
    datatype: Option<TurtleDatatypeAnnotation>,
}
impl TurtleRdfLiteralBuilder {
    pub fn with_language_token(mut self, language_token: SyntaxToken) -> Self {
        self.language_token = Some(language_token);
        self
    }
    pub fn with_datatype(mut self, datatype: TurtleDatatypeAnnotation) -> Self {
        self.datatype = Some(datatype);
        self
    }
    pub fn build(self) -> TurtleRdfLiteral {
        TurtleRdfLiteral::unwrap_cast(SyntaxNode::new_detached(
            TurtleSyntaxKind::TURTLE_RDF_LITERAL,
            [
                Some(SyntaxElement::Node(self.value.into_syntax())),
                self.language_token.map(|token| SyntaxElement::Token(token)),
                self.datatype
                    .map(|token| SyntaxElement::Node(token.into_syntax())),
            ],
        ))
    }
}
pub fn turtle_root(statements: TurtleStatementList, eof_token: SyntaxToken) -> TurtleRootBuilder {
    TurtleRootBuilder {
        statements,
        eof_token,
        bom_token: None,
    }
}
pub struct TurtleRootBuilder {
    statements: TurtleStatementList,
    eof_token: SyntaxToken,
    bom_token: Option<SyntaxToken>,
}
impl TurtleRootBuilder {
    pub fn with_bom_token(mut self, bom_token: SyntaxToken) -> Self {
        self.bom_token = Some(bom_token);
        self
    }
    pub fn build(self) -> TurtleRoot {
        TurtleRoot::unwrap_cast(SyntaxNode::new_detached(
            TurtleSyntaxKind::TURTLE_ROOT,
            [
                self.bom_token.map(|token| SyntaxElement::Token(token)),
                Some(SyntaxElement::Node(self.statements.into_syntax())),
                Some(SyntaxElement::Token(self.eof_token)),
            ],
        ))
    }
}
pub fn turtle_sparql_base_declaration(
    SPARQL_BASE_token: SyntaxToken,
    iri_token: SyntaxToken,
) -> TurtleSparqlBaseDeclaration {
    TurtleSparqlBaseDeclaration::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_SPARQL_BASE_DECLARATION,
        [
            Some(SyntaxElement::Token(SPARQL_BASE_token)),
            Some(SyntaxElement::Token(iri_token)),
        ],
    ))
}
pub fn turtle_sparql_prefix_declaration(
    SPARQL_PREFIX_token: SyntaxToken,
    namespace_token: SyntaxToken,
    iri_token: SyntaxToken,
) -> TurtleSparqlPrefixDeclaration {
    TurtleSparqlPrefixDeclaration::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_SPARQL_PREFIX_DECLARATION,
        [
            Some(SyntaxElement::Token(SPARQL_PREFIX_token)),
            Some(SyntaxElement::Token(namespace_token)),
            Some(SyntaxElement::Token(iri_token)),
        ],
    ))
}
pub fn turtle_string(value_token: SyntaxToken) -> TurtleString {
    TurtleString::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_STRING,
        [Some(SyntaxElement::Token(value_token))],
    ))
}
pub fn turtle_subject(any_turtle_subject: AnyTurtleSubject) -> TurtleSubject {
    TurtleSubject::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_SUBJECT,
        [Some(SyntaxElement::Node(any_turtle_subject.into_syntax()))],
    ))
}
pub fn turtle_triples(
    subject: TurtleSubject,
    predicates: TurtlePredicateObjectList,
    dot_token: SyntaxToken,
) -> TurtleTriples {
    TurtleTriples::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_TRIPLES,
        [
            Some(SyntaxElement::Node(subject.into_syntax())),
            Some(SyntaxElement::Node(predicates.into_syntax())),
            Some(SyntaxElement::Token(dot_token)),
        ],
    ))
}
pub fn turtle_verb() -> TurtleVerbBuilder {
    TurtleVerbBuilder {
        any_turtle_verb: None,
        a_token_token: None,
    }
}
pub struct TurtleVerbBuilder {
    any_turtle_verb: Option<AnyTurtleVerb>,
    a_token_token: Option<SyntaxToken>,
}
impl TurtleVerbBuilder {
    pub fn with_any_turtle_verb(mut self, any_turtle_verb: AnyTurtleVerb) -> Self {
        self.any_turtle_verb = Some(any_turtle_verb);
        self
    }
    pub fn with_a_token_token(mut self, a_token_token: SyntaxToken) -> Self {
        self.a_token_token = Some(a_token_token);
        self
    }
    pub fn build(self) -> TurtleVerb {
        TurtleVerb::unwrap_cast(SyntaxNode::new_detached(
            TurtleSyntaxKind::TURTLE_VERB,
            [
                self.any_turtle_verb
                    .map(|token| SyntaxElement::Node(token.into_syntax())),
                self.a_token_token.map(|token| SyntaxElement::Token(token)),
            ],
        ))
    }
}
pub fn turtle_collection_object_list<I>(items: I) -> TurtleCollectionObjectList
where
    I: IntoIterator<Item = TurtleObject>,
    I::IntoIter: ExactSizeIterator,
{
    TurtleCollectionObjectList::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_COLLECTION_OBJECT_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn turtle_object_list<I, S>(items: I, separators: S) -> TurtleObjectList
where
    I: IntoIterator<Item = TurtleObject>,
    I::IntoIter: ExactSizeIterator,
    S: IntoIterator<Item = TurtleSyntaxToken>,
    S::IntoIter: ExactSizeIterator,
{
    let mut items = items.into_iter();
    let mut separators = separators.into_iter();
    let length = items.len() + separators.len();
    TurtleObjectList::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_OBJECT_LIST,
        (0..length).map(|index| {
            if index % 2 == 0 {
                Some(items.next()?.into_syntax().into())
            } else {
                Some(separators.next()?.into())
            }
        }),
    ))
}
pub fn turtle_predicate_object_pair_list<I, S>(
    items: I,
    separators: S,
) -> TurtlePredicateObjectPairList
where
    I: IntoIterator<Item = TurtlePredicateObjectPair>,
    I::IntoIter: ExactSizeIterator,
    S: IntoIterator<Item = TurtleSyntaxToken>,
    S::IntoIter: ExactSizeIterator,
{
    let mut items = items.into_iter();
    let mut separators = separators.into_iter();
    let length = items.len() + separators.len();
    TurtlePredicateObjectPairList::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_PREDICATE_OBJECT_PAIR_LIST,
        (0..length).map(|index| {
            if index % 2 == 0 {
                Some(items.next()?.into_syntax().into())
            } else {
                Some(separators.next()?.into())
            }
        }),
    ))
}
pub fn turtle_statement_list<I>(items: I) -> TurtleStatementList
where
    I: IntoIterator<Item = AnyTurtleStatement>,
    I::IntoIter: ExactSizeIterator,
{
    TurtleStatementList::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_STATEMENT_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn turtle_bogus<I>(slots: I) -> TurtleBogus
where
    I: IntoIterator<Item = Option<SyntaxElement>>,
    I::IntoIter: ExactSizeIterator,
{
    TurtleBogus::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_BOGUS,
        slots,
    ))
}
pub fn turtle_bogus_statement<I>(slots: I) -> TurtleBogusStatement
where
    I: IntoIterator<Item = Option<SyntaxElement>>,
    I::IntoIter: ExactSizeIterator,
{
    TurtleBogusStatement::unwrap_cast(SyntaxNode::new_detached(
        TurtleSyntaxKind::TURTLE_BOGUS_STATEMENT,
        slots,
    ))
}
