//! Generated file, do not edit by hand, see `xtask/codegen`

#![allow(unused_mut)]
use biome_rowan::{
    AstNode, ParsedChildren, RawNodeSlots, RawSyntaxNode, SyntaxFactory, SyntaxKind,
};
use biome_turtle_syntax::{T, TurtleSyntaxKind, TurtleSyntaxKind::*, *};
#[derive(Debug)]
pub struct TurtleSyntaxFactory;
impl SyntaxFactory for TurtleSyntaxFactory {
    type Kind = TurtleSyntaxKind;
    fn make_syntax(
        kind: Self::Kind,
        children: ParsedChildren<Self::Kind>,
    ) -> RawSyntaxNode<Self::Kind> {
        match kind {
            TURTLE_BOGUS | TURTLE_BOGUS_STATEMENT => {
                RawSyntaxNode::new(kind, children.into_iter().map(Some))
            }
            TURTLE_BASE_DECLARATION => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<3usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && element.kind() == T![base]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == TURTLE_IRIREF_LITERAL
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == T ! [.]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_BASE_DECLARATION.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_BASE_DECLARATION, children)
            }
            TURTLE_BLANK_NODE => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<1usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && matches!(
                        element.kind(),
                        TURTLE_BLANK_NODE_LABEL_LITERAL | TURTLE_ANON_TOKEN
                    )
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_BLANK_NODE.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_BLANK_NODE, children)
            }
            TURTLE_BLANK_NODE_PROPERTY_LIST => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<3usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && element.kind() == T!['[']
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && TurtlePredicateObjectList::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == T![']']
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_BLANK_NODE_PROPERTY_LIST.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_BLANK_NODE_PROPERTY_LIST, children)
            }
            TURTLE_BOOLEAN_LITERAL => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<1usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && matches!(element.kind(), T![true] | T![false])
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_BOOLEAN_LITERAL.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_BOOLEAN_LITERAL, children)
            }
            TURTLE_COLLECTION => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<3usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && element.kind() == T!['(']
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && TurtleCollectionObjectList::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == T![')']
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_COLLECTION.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_COLLECTION, children)
            }
            TURTLE_DATATYPE_ANNOTATION => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<2usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && element.kind() == T!["^^"]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && TurtleIri::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_DATATYPE_ANNOTATION.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_DATATYPE_ANNOTATION, children)
            }
            TURTLE_IRI => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<2usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && AnyTurtleIriValue::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == TURTLE_IRIREF_LITERAL
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_IRI.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_IRI, children)
            }
            TURTLE_NUMERIC_LITERAL => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<1usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && matches!(
                        element.kind(),
                        TURTLE_INTEGER_LITERAL | TURTLE_DECIMAL_LITERAL | TURTLE_DOUBLE_LITERAL
                    )
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_NUMERIC_LITERAL.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_NUMERIC_LITERAL, children)
            }
            TURTLE_OBJECT => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<1usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && AnyTurtleObject::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_OBJECT.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_OBJECT, children)
            }
            TURTLE_PREDICATE_OBJECT_LIST => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<1usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && TurtlePredicateObjectPairList::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_PREDICATE_OBJECT_LIST.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_PREDICATE_OBJECT_LIST, children)
            }
            TURTLE_PREDICATE_OBJECT_PAIR => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<2usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && TurtleVerb::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && TurtleObjectList::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_PREDICATE_OBJECT_PAIR.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_PREDICATE_OBJECT_PAIR, children)
            }
            TURTLE_PREFIX_DECLARATION => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<4usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && element.kind() == T![prefix]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == TURTLE_PNAME_NS_LITERAL
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == TURTLE_IRIREF_LITERAL
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == T ! [.]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_PREFIX_DECLARATION.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_PREFIX_DECLARATION, children)
            }
            TURTLE_PREFIXED_NAME => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<1usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && matches!(
                        element.kind(),
                        TURTLE_PNAME_LN_LITERAL | TURTLE_PNAME_NS_LITERAL
                    )
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_PREFIXED_NAME.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_PREFIXED_NAME, children)
            }
            TURTLE_RDF_LITERAL => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<3usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && TurtleString::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == TURTLE_LANGTAG_LITERAL
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && TurtleDatatypeAnnotation::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_RDF_LITERAL.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_RDF_LITERAL, children)
            }
            TURTLE_ROOT => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<3usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && element.kind() == T![UNICODE_BOM]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && TurtleStatementList::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == T![EOF]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_ROOT.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_ROOT, children)
            }
            TURTLE_SPARQL_BASE_DECLARATION => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<2usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && element.kind() == T![SPARQL_BASE]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == TURTLE_IRIREF_LITERAL
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_SPARQL_BASE_DECLARATION.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_SPARQL_BASE_DECLARATION, children)
            }
            TURTLE_SPARQL_PREFIX_DECLARATION => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<3usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && element.kind() == T![SPARQL_PREFIX]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == TURTLE_PNAME_NS_LITERAL
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == TURTLE_IRIREF_LITERAL
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_SPARQL_PREFIX_DECLARATION.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_SPARQL_PREFIX_DECLARATION, children)
            }
            TURTLE_STRING => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<1usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && matches!(
                        element.kind(),
                        TURTLE_STRING_LITERAL_QUOTE
                            | TURTLE_STRING_LITERAL_SINGLE_QUOTE
                            | TURTLE_STRING_LITERAL_LONG_QUOTE
                            | TURTLE_STRING_LITERAL_LONG_SINGLE_QUOTE
                    )
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_STRING.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_STRING, children)
            }
            TURTLE_SUBJECT => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<1usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && AnyTurtleSubject::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_SUBJECT.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_SUBJECT, children)
            }
            TURTLE_TRIPLES => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<3usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && TurtleSubject::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && TurtlePredicateObjectList::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == T ! [.]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_TRIPLES.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_TRIPLES, children)
            }
            TURTLE_VERB => {
                let mut elements = (&children).into_iter();
                let mut slots: RawNodeSlots<2usize> = RawNodeSlots::default();
                let mut current_element = elements.next();
                if let Some(element) = &current_element
                    && AnyTurtleVerb::can_cast(element.kind())
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if let Some(element) = &current_element
                    && element.kind() == T![a]
                {
                    slots.mark_present();
                    current_element = elements.next();
                }
                slots.next_slot();
                if current_element.is_some() {
                    return RawSyntaxNode::new(
                        TURTLE_VERB.to_bogus(),
                        children.into_iter().map(Some),
                    );
                }
                slots.into_node(TURTLE_VERB, children)
            }
            TURTLE_COLLECTION_OBJECT_LIST => {
                Self::make_node_list_syntax(kind, children, TurtleObject::can_cast)
            }
            TURTLE_OBJECT_LIST => Self::make_separated_list_syntax(
                kind,
                children,
                TurtleObject::can_cast,
                T ! [,],
                false,
            ),
            TURTLE_PREDICATE_OBJECT_PAIR_LIST => Self::make_separated_list_syntax(
                kind,
                children,
                TurtlePredicateObjectPair::can_cast,
                T ! [;],
                true,
            ),
            TURTLE_STATEMENT_LIST => {
                Self::make_node_list_syntax(kind, children, AnyTurtleStatement::can_cast)
            }
            _ => unreachable!("Is {:?} a token?", kind),
        }
    }
}
