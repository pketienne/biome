//! Generated file, do not edit by hand, see `xtask/codegen`

#[doc = r" Reconstruct an AstNode from a SyntaxNode"]
#[doc = r""]
#[doc = r" This macros performs a match over the [kind](biome_rowan::SyntaxNode::kind)"]
#[doc = r" of the provided [biome_rowan::SyntaxNode] and constructs the appropriate"]
#[doc = r" AstNode type for it, then execute the provided expression over it."]
#[doc = r""]
#[doc = r" # Examples"]
#[doc = r""]
#[doc = r" ```ignore"]
#[doc = r" map_syntax_node!(syntax_node, node => node.format())"]
#[doc = r" ```"]
#[macro_export]
macro_rules! map_syntax_node {
    ($ node : expr , $ pattern : pat => $ body : expr) => {
        match $node {
            node => match $crate::TurtleSyntaxNode::kind(&node) {
                $crate::TurtleSyntaxKind::TURTLE_BASE_DECLARATION => {
                    let $pattern = unsafe { $crate::TurtleBaseDeclaration::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_BLANK_NODE => {
                    let $pattern = unsafe { $crate::TurtleBlankNode::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_BLANK_NODE_PROPERTY_LIST => {
                    let $pattern =
                        unsafe { $crate::TurtleBlankNodePropertyList::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_BOOLEAN_LITERAL => {
                    let $pattern = unsafe { $crate::TurtleBooleanLiteral::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_COLLECTION => {
                    let $pattern = unsafe { $crate::TurtleCollection::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_DATATYPE_ANNOTATION => {
                    let $pattern = unsafe { $crate::TurtleDatatypeAnnotation::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_IRI => {
                    let $pattern = unsafe { $crate::TurtleIri::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_NUMERIC_LITERAL => {
                    let $pattern = unsafe { $crate::TurtleNumericLiteral::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_OBJECT => {
                    let $pattern = unsafe { $crate::TurtleObject::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_PREDICATE_OBJECT_LIST => {
                    let $pattern =
                        unsafe { $crate::TurtlePredicateObjectList::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_PREDICATE_OBJECT_PAIR => {
                    let $pattern =
                        unsafe { $crate::TurtlePredicateObjectPair::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_PREFIX_DECLARATION => {
                    let $pattern = unsafe { $crate::TurtlePrefixDeclaration::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_PREFIXED_NAME => {
                    let $pattern = unsafe { $crate::TurtlePrefixedName::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_RDF_LITERAL => {
                    let $pattern = unsafe { $crate::TurtleRdfLiteral::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_ROOT => {
                    let $pattern = unsafe { $crate::TurtleRoot::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_SPARQL_BASE_DECLARATION => {
                    let $pattern =
                        unsafe { $crate::TurtleSparqlBaseDeclaration::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_SPARQL_PREFIX_DECLARATION => {
                    let $pattern =
                        unsafe { $crate::TurtleSparqlPrefixDeclaration::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_STRING => {
                    let $pattern = unsafe { $crate::TurtleString::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_SUBJECT => {
                    let $pattern = unsafe { $crate::TurtleSubject::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_TRIPLES => {
                    let $pattern = unsafe { $crate::TurtleTriples::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_VERB => {
                    let $pattern = unsafe { $crate::TurtleVerb::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_BOGUS => {
                    let $pattern = unsafe { $crate::TurtleBogus::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_BOGUS_STATEMENT => {
                    let $pattern = unsafe { $crate::TurtleBogusStatement::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_COLLECTION_OBJECT_LIST => {
                    let $pattern =
                        unsafe { $crate::TurtleCollectionObjectList::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_OBJECT_LIST => {
                    let $pattern = unsafe { $crate::TurtleObjectList::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_PREDICATE_OBJECT_PAIR_LIST => {
                    let $pattern =
                        unsafe { $crate::TurtlePredicateObjectPairList::new_unchecked(node) };
                    $body
                }
                $crate::TurtleSyntaxKind::TURTLE_STATEMENT_LIST => {
                    let $pattern = unsafe { $crate::TurtleStatementList::new_unchecked(node) };
                    $body
                }
                _ => unreachable!(),
            },
        }
    };
}
pub(crate) use map_syntax_node;
