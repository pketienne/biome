//! Generated file, do not edit by hand, see `xtask/codegen`

#![allow(dead_code)]
#![allow(unused)]
use crate::{
    TurtleLanguage as Language, TurtleSyntaxElement as SyntaxElement,
    TurtleSyntaxElementChildren as SyntaxElementChildren,
    TurtleSyntaxKind::{self as SyntaxKind, *},
    TurtleSyntaxList as SyntaxList, TurtleSyntaxNode as SyntaxNode,
    TurtleSyntaxToken as SyntaxToken,
    macros::map_syntax_node,
};
use biome_rowan::{
    AstNode, AstNodeList, AstNodeListIterator, AstNodeSlotMap, AstSeparatedList,
    AstSeparatedListNodesIterator, RawSyntaxKind, SyntaxKindSet, SyntaxResult, support,
};
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use std::fmt::{Debug, Formatter};
#[doc = r" Sentinel value indicating a missing element in a dynamic node, where"]
#[doc = r" the slots are not statically known."]
pub(crate) const SLOT_MAP_EMPTY_VALUE: u8 = u8::MAX;
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleBaseDeclaration {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleBaseDeclaration {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleBaseDeclarationFields {
        TurtleBaseDeclarationFields {
            base_token: self.base_token(),
            iri_token: self.iri_token(),
            dot_token: self.dot_token(),
        }
    }
    pub fn base_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn iri_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn dot_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 2usize)
    }
}
impl Serialize for TurtleBaseDeclaration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleBaseDeclarationFields {
    pub base_token: SyntaxResult<SyntaxToken>,
    pub iri_token: SyntaxResult<SyntaxToken>,
    pub dot_token: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleBlankNode {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleBlankNode {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleBlankNodeFields {
        TurtleBlankNodeFields {
            value: self.value(),
        }
    }
    pub fn value(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
}
impl Serialize for TurtleBlankNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleBlankNodeFields {
    pub value: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleBlankNodePropertyList {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleBlankNodePropertyList {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleBlankNodePropertyListFields {
        TurtleBlankNodePropertyListFields {
            l_brack_token: self.l_brack_token(),
            predicates: self.predicates(),
            r_brack_token: self.r_brack_token(),
        }
    }
    pub fn l_brack_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn predicates(&self) -> SyntaxResult<TurtlePredicateObjectList> {
        support::required_node(&self.syntax, 1usize)
    }
    pub fn r_brack_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 2usize)
    }
}
impl Serialize for TurtleBlankNodePropertyList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleBlankNodePropertyListFields {
    pub l_brack_token: SyntaxResult<SyntaxToken>,
    pub predicates: SyntaxResult<TurtlePredicateObjectList>,
    pub r_brack_token: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleBooleanLiteral {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleBooleanLiteral {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleBooleanLiteralFields {
        TurtleBooleanLiteralFields {
            value_token: self.value_token(),
        }
    }
    pub fn value_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
}
impl Serialize for TurtleBooleanLiteral {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleBooleanLiteralFields {
    pub value_token: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleCollection {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleCollection {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleCollectionFields {
        TurtleCollectionFields {
            l_paren_token: self.l_paren_token(),
            objects: self.objects(),
            r_paren_token: self.r_paren_token(),
        }
    }
    pub fn l_paren_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn objects(&self) -> TurtleCollectionObjectList {
        support::list(&self.syntax, 1usize)
    }
    pub fn r_paren_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 2usize)
    }
}
impl Serialize for TurtleCollection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleCollectionFields {
    pub l_paren_token: SyntaxResult<SyntaxToken>,
    pub objects: TurtleCollectionObjectList,
    pub r_paren_token: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleDatatypeAnnotation {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleDatatypeAnnotation {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleDatatypeAnnotationFields {
        TurtleDatatypeAnnotationFields {
            caret_caret_token: self.caret_caret_token(),
            datatype: self.datatype(),
        }
    }
    pub fn caret_caret_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn datatype(&self) -> SyntaxResult<TurtleIri> {
        support::required_node(&self.syntax, 1usize)
    }
}
impl Serialize for TurtleDatatypeAnnotation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleDatatypeAnnotationFields {
    pub caret_caret_token: SyntaxResult<SyntaxToken>,
    pub datatype: SyntaxResult<TurtleIri>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleIri {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleIri {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleIriFields {
        TurtleIriFields {
            value: self.value(),
        }
    }
    pub fn value(&self) -> SyntaxResult<AnyTurtleIriValue> {
        support::required_node(&self.syntax, 0usize)
    }
}
impl Serialize for TurtleIri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleIriFields {
    pub value: SyntaxResult<AnyTurtleIriValue>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleNumericLiteral {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleNumericLiteral {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleNumericLiteralFields {
        TurtleNumericLiteralFields {
            value: self.value(),
        }
    }
    pub fn value(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
}
impl Serialize for TurtleNumericLiteral {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleNumericLiteralFields {
    pub value: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleObject {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleObject {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleObjectFields {
        TurtleObjectFields {
            any_turtle_object: self.any_turtle_object(),
        }
    }
    pub fn any_turtle_object(&self) -> SyntaxResult<AnyTurtleObject> {
        support::required_node(&self.syntax, 0usize)
    }
}
impl Serialize for TurtleObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleObjectFields {
    pub any_turtle_object: SyntaxResult<AnyTurtleObject>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtlePredicateObjectList {
    pub(crate) syntax: SyntaxNode,
}
impl TurtlePredicateObjectList {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtlePredicateObjectListFields {
        TurtlePredicateObjectListFields {
            pairs: self.pairs(),
        }
    }
    pub fn pairs(&self) -> TurtlePredicateObjectPairList {
        support::list(&self.syntax, 0usize)
    }
}
impl Serialize for TurtlePredicateObjectList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtlePredicateObjectListFields {
    pub pairs: TurtlePredicateObjectPairList,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtlePredicateObjectPair {
    pub(crate) syntax: SyntaxNode,
}
impl TurtlePredicateObjectPair {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtlePredicateObjectPairFields {
        TurtlePredicateObjectPairFields {
            verb: self.verb(),
            objects: self.objects(),
        }
    }
    pub fn verb(&self) -> SyntaxResult<TurtleVerb> {
        support::required_node(&self.syntax, 0usize)
    }
    pub fn objects(&self) -> TurtleObjectList {
        support::list(&self.syntax, 1usize)
    }
}
impl Serialize for TurtlePredicateObjectPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtlePredicateObjectPairFields {
    pub verb: SyntaxResult<TurtleVerb>,
    pub objects: TurtleObjectList,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtlePrefixDeclaration {
    pub(crate) syntax: SyntaxNode,
}
impl TurtlePrefixDeclaration {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtlePrefixDeclarationFields {
        TurtlePrefixDeclarationFields {
            prefix_token: self.prefix_token(),
            namespace_token: self.namespace_token(),
            iri_token: self.iri_token(),
            dot_token: self.dot_token(),
        }
    }
    pub fn prefix_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn namespace_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn iri_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 2usize)
    }
    pub fn dot_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 3usize)
    }
}
impl Serialize for TurtlePrefixDeclaration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtlePrefixDeclarationFields {
    pub prefix_token: SyntaxResult<SyntaxToken>,
    pub namespace_token: SyntaxResult<SyntaxToken>,
    pub iri_token: SyntaxResult<SyntaxToken>,
    pub dot_token: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtlePrefixedName {
    pub(crate) syntax: SyntaxNode,
}
impl TurtlePrefixedName {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtlePrefixedNameFields {
        TurtlePrefixedNameFields {
            value: self.value(),
        }
    }
    pub fn value(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
}
impl Serialize for TurtlePrefixedName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtlePrefixedNameFields {
    pub value: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleRdfLiteral {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleRdfLiteral {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleRdfLiteralFields {
        TurtleRdfLiteralFields {
            value: self.value(),
            language_token: self.language_token(),
            datatype: self.datatype(),
        }
    }
    pub fn value(&self) -> SyntaxResult<TurtleString> {
        support::required_node(&self.syntax, 0usize)
    }
    pub fn language_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, 1usize)
    }
    pub fn datatype(&self) -> Option<TurtleDatatypeAnnotation> {
        support::node(&self.syntax, 2usize)
    }
}
impl Serialize for TurtleRdfLiteral {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleRdfLiteralFields {
    pub value: SyntaxResult<TurtleString>,
    pub language_token: Option<SyntaxToken>,
    pub datatype: Option<TurtleDatatypeAnnotation>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleRoot {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleRoot {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleRootFields {
        TurtleRootFields {
            bom_token: self.bom_token(),
            statements: self.statements(),
            eof_token: self.eof_token(),
        }
    }
    pub fn bom_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, 0usize)
    }
    pub fn statements(&self) -> TurtleStatementList {
        support::list(&self.syntax, 1usize)
    }
    pub fn eof_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 2usize)
    }
}
impl Serialize for TurtleRoot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleRootFields {
    pub bom_token: Option<SyntaxToken>,
    pub statements: TurtleStatementList,
    pub eof_token: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleSparqlBaseDeclaration {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleSparqlBaseDeclaration {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleSparqlBaseDeclarationFields {
        TurtleSparqlBaseDeclarationFields {
            SPARQL_BASE_token: self.SPARQL_BASE_token(),
            iri_token: self.iri_token(),
        }
    }
    pub fn SPARQL_BASE_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn iri_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 1usize)
    }
}
impl Serialize for TurtleSparqlBaseDeclaration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleSparqlBaseDeclarationFields {
    pub SPARQL_BASE_token: SyntaxResult<SyntaxToken>,
    pub iri_token: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleSparqlPrefixDeclaration {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleSparqlPrefixDeclaration {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleSparqlPrefixDeclarationFields {
        TurtleSparqlPrefixDeclarationFields {
            SPARQL_PREFIX_token: self.SPARQL_PREFIX_token(),
            namespace_token: self.namespace_token(),
            iri_token: self.iri_token(),
        }
    }
    pub fn SPARQL_PREFIX_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn namespace_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn iri_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 2usize)
    }
}
impl Serialize for TurtleSparqlPrefixDeclaration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleSparqlPrefixDeclarationFields {
    pub SPARQL_PREFIX_token: SyntaxResult<SyntaxToken>,
    pub namespace_token: SyntaxResult<SyntaxToken>,
    pub iri_token: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleString {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleString {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleStringFields {
        TurtleStringFields {
            value: self.value(),
        }
    }
    pub fn value(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 0usize)
    }
}
impl Serialize for TurtleString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleStringFields {
    pub value: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleSubject {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleSubject {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleSubjectFields {
        TurtleSubjectFields {
            any_turtle_subject: self.any_turtle_subject(),
        }
    }
    pub fn any_turtle_subject(&self) -> SyntaxResult<AnyTurtleSubject> {
        support::required_node(&self.syntax, 0usize)
    }
}
impl Serialize for TurtleSubject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleSubjectFields {
    pub any_turtle_subject: SyntaxResult<AnyTurtleSubject>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleTriples {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleTriples {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleTriplesFields {
        TurtleTriplesFields {
            subject: self.subject(),
            predicates: self.predicates(),
            dot_token: self.dot_token(),
        }
    }
    pub fn subject(&self) -> SyntaxResult<TurtleSubject> {
        support::required_node(&self.syntax, 0usize)
    }
    pub fn predicates(&self) -> SyntaxResult<TurtlePredicateObjectList> {
        support::required_node(&self.syntax, 1usize)
    }
    pub fn dot_token(&self) -> SyntaxResult<SyntaxToken> {
        support::required_token(&self.syntax, 2usize)
    }
}
impl Serialize for TurtleTriples {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleTriplesFields {
    pub subject: SyntaxResult<TurtleSubject>,
    pub predicates: SyntaxResult<TurtlePredicateObjectList>,
    pub dot_token: SyntaxResult<SyntaxToken>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TurtleVerb {
    pub(crate) syntax: SyntaxNode,
}
impl TurtleVerb {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn as_fields(&self) -> TurtleVerbFields {
        TurtleVerbFields {
            any_turtle_verb: self.any_turtle_verb(),
        }
    }
    pub fn any_turtle_verb(&self) -> SyntaxResult<AnyTurtleVerb> {
        support::required_node(&self.syntax, 0usize)
    }
}
impl Serialize for TurtleVerb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_fields().serialize(serializer)
    }
}
#[derive(Serialize)]
pub struct TurtleVerbFields {
    pub any_turtle_verb: SyntaxResult<AnyTurtleVerb>,
}
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub enum AnyTurtleDirective {
    TurtleBaseDeclaration(TurtleBaseDeclaration),
    TurtlePrefixDeclaration(TurtlePrefixDeclaration),
    TurtleSparqlBaseDeclaration(TurtleSparqlBaseDeclaration),
    TurtleSparqlPrefixDeclaration(TurtleSparqlPrefixDeclaration),
}
impl AnyTurtleDirective {
    pub fn as_turtle_base_declaration(&self) -> Option<&TurtleBaseDeclaration> {
        match &self {
            Self::TurtleBaseDeclaration(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_prefix_declaration(&self) -> Option<&TurtlePrefixDeclaration> {
        match &self {
            Self::TurtlePrefixDeclaration(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_sparql_base_declaration(&self) -> Option<&TurtleSparqlBaseDeclaration> {
        match &self {
            Self::TurtleSparqlBaseDeclaration(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_sparql_prefix_declaration(&self) -> Option<&TurtleSparqlPrefixDeclaration> {
        match &self {
            Self::TurtleSparqlPrefixDeclaration(item) => Some(item),
            _ => None,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub enum AnyTurtleIriValue {
    TurtleBogus(TurtleBogus),
    TurtlePrefixedName(TurtlePrefixedName),
}
impl AnyTurtleIriValue {
    pub fn as_turtle_bogus(&self) -> Option<&TurtleBogus> {
        match &self {
            Self::TurtleBogus(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_prefixed_name(&self) -> Option<&TurtlePrefixedName> {
        match &self {
            Self::TurtlePrefixedName(item) => Some(item),
            _ => None,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub enum AnyTurtleObject {
    TurtleBlankNode(TurtleBlankNode),
    TurtleBlankNodePropertyList(TurtleBlankNodePropertyList),
    TurtleBogus(TurtleBogus),
    TurtleBooleanLiteral(TurtleBooleanLiteral),
    TurtleCollection(TurtleCollection),
    TurtleIri(TurtleIri),
    TurtleNumericLiteral(TurtleNumericLiteral),
    TurtleRdfLiteral(TurtleRdfLiteral),
}
impl AnyTurtleObject {
    pub fn as_turtle_blank_node(&self) -> Option<&TurtleBlankNode> {
        match &self {
            Self::TurtleBlankNode(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_blank_node_property_list(&self) -> Option<&TurtleBlankNodePropertyList> {
        match &self {
            Self::TurtleBlankNodePropertyList(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_bogus(&self) -> Option<&TurtleBogus> {
        match &self {
            Self::TurtleBogus(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_boolean_literal(&self) -> Option<&TurtleBooleanLiteral> {
        match &self {
            Self::TurtleBooleanLiteral(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_collection(&self) -> Option<&TurtleCollection> {
        match &self {
            Self::TurtleCollection(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_iri(&self) -> Option<&TurtleIri> {
        match &self {
            Self::TurtleIri(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_numeric_literal(&self) -> Option<&TurtleNumericLiteral> {
        match &self {
            Self::TurtleNumericLiteral(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_rdf_literal(&self) -> Option<&TurtleRdfLiteral> {
        match &self {
            Self::TurtleRdfLiteral(item) => Some(item),
            _ => None,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub enum AnyTurtleStatement {
    AnyTurtleDirective(AnyTurtleDirective),
    TurtleBogusStatement(TurtleBogusStatement),
    TurtleTriples(TurtleTriples),
}
impl AnyTurtleStatement {
    pub fn as_any_turtle_directive(&self) -> Option<&AnyTurtleDirective> {
        match &self {
            Self::AnyTurtleDirective(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_bogus_statement(&self) -> Option<&TurtleBogusStatement> {
        match &self {
            Self::TurtleBogusStatement(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_triples(&self) -> Option<&TurtleTriples> {
        match &self {
            Self::TurtleTriples(item) => Some(item),
            _ => None,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub enum AnyTurtleSubject {
    TurtleBlankNode(TurtleBlankNode),
    TurtleBlankNodePropertyList(TurtleBlankNodePropertyList),
    TurtleCollection(TurtleCollection),
    TurtleIri(TurtleIri),
}
impl AnyTurtleSubject {
    pub fn as_turtle_blank_node(&self) -> Option<&TurtleBlankNode> {
        match &self {
            Self::TurtleBlankNode(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_blank_node_property_list(&self) -> Option<&TurtleBlankNodePropertyList> {
        match &self {
            Self::TurtleBlankNodePropertyList(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_collection(&self) -> Option<&TurtleCollection> {
        match &self {
            Self::TurtleCollection(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_iri(&self) -> Option<&TurtleIri> {
        match &self {
            Self::TurtleIri(item) => Some(item),
            _ => None,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub enum AnyTurtleVerb {
    TurtleBogus(TurtleBogus),
    TurtleIri(TurtleIri),
}
impl AnyTurtleVerb {
    pub fn as_turtle_bogus(&self) -> Option<&TurtleBogus> {
        match &self {
            Self::TurtleBogus(item) => Some(item),
            _ => None,
        }
    }
    pub fn as_turtle_iri(&self) -> Option<&TurtleIri> {
        match &self {
            Self::TurtleIri(item) => Some(item),
            _ => None,
        }
    }
}
impl AstNode for TurtleBaseDeclaration {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_BASE_DECLARATION as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_BASE_DECLARATION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleBaseDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleBaseDeclaration")
                .field("base_token", &support::DebugSyntaxResult(self.base_token()))
                .field("iri_token", &support::DebugSyntaxResult(self.iri_token()))
                .field("dot_token", &support::DebugSyntaxResult(self.dot_token()))
                .finish()
        } else {
            f.debug_struct("TurtleBaseDeclaration").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleBaseDeclaration> for SyntaxNode {
    fn from(n: TurtleBaseDeclaration) -> Self {
        n.syntax
    }
}
impl From<TurtleBaseDeclaration> for SyntaxElement {
    fn from(n: TurtleBaseDeclaration) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleBlankNode {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_BLANK_NODE as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_BLANK_NODE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleBlankNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleBlankNode")
                .field("value", &support::DebugSyntaxResult(self.value()))
                .finish()
        } else {
            f.debug_struct("TurtleBlankNode").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleBlankNode> for SyntaxNode {
    fn from(n: TurtleBlankNode) -> Self {
        n.syntax
    }
}
impl From<TurtleBlankNode> for SyntaxElement {
    fn from(n: TurtleBlankNode) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleBlankNodePropertyList {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_BLANK_NODE_PROPERTY_LIST as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_BLANK_NODE_PROPERTY_LIST
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleBlankNodePropertyList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleBlankNodePropertyList")
                .field(
                    "l_brack_token",
                    &support::DebugSyntaxResult(self.l_brack_token()),
                )
                .field("predicates", &support::DebugSyntaxResult(self.predicates()))
                .field(
                    "r_brack_token",
                    &support::DebugSyntaxResult(self.r_brack_token()),
                )
                .finish()
        } else {
            f.debug_struct("TurtleBlankNodePropertyList").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleBlankNodePropertyList> for SyntaxNode {
    fn from(n: TurtleBlankNodePropertyList) -> Self {
        n.syntax
    }
}
impl From<TurtleBlankNodePropertyList> for SyntaxElement {
    fn from(n: TurtleBlankNodePropertyList) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleBooleanLiteral {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_BOOLEAN_LITERAL as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_BOOLEAN_LITERAL
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleBooleanLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleBooleanLiteral")
                .field(
                    "value_token",
                    &support::DebugSyntaxResult(self.value_token()),
                )
                .finish()
        } else {
            f.debug_struct("TurtleBooleanLiteral").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleBooleanLiteral> for SyntaxNode {
    fn from(n: TurtleBooleanLiteral) -> Self {
        n.syntax
    }
}
impl From<TurtleBooleanLiteral> for SyntaxElement {
    fn from(n: TurtleBooleanLiteral) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleCollection {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_COLLECTION as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_COLLECTION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleCollection")
                .field(
                    "l_paren_token",
                    &support::DebugSyntaxResult(self.l_paren_token()),
                )
                .field("objects", &self.objects())
                .field(
                    "r_paren_token",
                    &support::DebugSyntaxResult(self.r_paren_token()),
                )
                .finish()
        } else {
            f.debug_struct("TurtleCollection").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleCollection> for SyntaxNode {
    fn from(n: TurtleCollection) -> Self {
        n.syntax
    }
}
impl From<TurtleCollection> for SyntaxElement {
    fn from(n: TurtleCollection) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleDatatypeAnnotation {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_DATATYPE_ANNOTATION as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_DATATYPE_ANNOTATION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleDatatypeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleDatatypeAnnotation")
                .field(
                    "caret_caret_token",
                    &support::DebugSyntaxResult(self.caret_caret_token()),
                )
                .field("datatype", &support::DebugSyntaxResult(self.datatype()))
                .finish()
        } else {
            f.debug_struct("TurtleDatatypeAnnotation").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleDatatypeAnnotation> for SyntaxNode {
    fn from(n: TurtleDatatypeAnnotation) -> Self {
        n.syntax
    }
}
impl From<TurtleDatatypeAnnotation> for SyntaxElement {
    fn from(n: TurtleDatatypeAnnotation) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleIri {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_IRI as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_IRI
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleIri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleIri")
                .field("value", &support::DebugSyntaxResult(self.value()))
                .finish()
        } else {
            f.debug_struct("TurtleIri").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleIri> for SyntaxNode {
    fn from(n: TurtleIri) -> Self {
        n.syntax
    }
}
impl From<TurtleIri> for SyntaxElement {
    fn from(n: TurtleIri) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleNumericLiteral {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_NUMERIC_LITERAL as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_NUMERIC_LITERAL
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleNumericLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleNumericLiteral")
                .field("value", &support::DebugSyntaxResult(self.value()))
                .finish()
        } else {
            f.debug_struct("TurtleNumericLiteral").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleNumericLiteral> for SyntaxNode {
    fn from(n: TurtleNumericLiteral) -> Self {
        n.syntax
    }
}
impl From<TurtleNumericLiteral> for SyntaxElement {
    fn from(n: TurtleNumericLiteral) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleObject {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_OBJECT as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_OBJECT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleObject")
                .field(
                    "any_turtle_object",
                    &support::DebugSyntaxResult(self.any_turtle_object()),
                )
                .finish()
        } else {
            f.debug_struct("TurtleObject").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleObject> for SyntaxNode {
    fn from(n: TurtleObject) -> Self {
        n.syntax
    }
}
impl From<TurtleObject> for SyntaxElement {
    fn from(n: TurtleObject) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtlePredicateObjectList {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_PREDICATE_OBJECT_LIST as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_PREDICATE_OBJECT_LIST
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtlePredicateObjectList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtlePredicateObjectList")
                .field("pairs", &self.pairs())
                .finish()
        } else {
            f.debug_struct("TurtlePredicateObjectList").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtlePredicateObjectList> for SyntaxNode {
    fn from(n: TurtlePredicateObjectList) -> Self {
        n.syntax
    }
}
impl From<TurtlePredicateObjectList> for SyntaxElement {
    fn from(n: TurtlePredicateObjectList) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtlePredicateObjectPair {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_PREDICATE_OBJECT_PAIR as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_PREDICATE_OBJECT_PAIR
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtlePredicateObjectPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtlePredicateObjectPair")
                .field("verb", &support::DebugSyntaxResult(self.verb()))
                .field("objects", &self.objects())
                .finish()
        } else {
            f.debug_struct("TurtlePredicateObjectPair").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtlePredicateObjectPair> for SyntaxNode {
    fn from(n: TurtlePredicateObjectPair) -> Self {
        n.syntax
    }
}
impl From<TurtlePredicateObjectPair> for SyntaxElement {
    fn from(n: TurtlePredicateObjectPair) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtlePrefixDeclaration {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_PREFIX_DECLARATION as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_PREFIX_DECLARATION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtlePrefixDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtlePrefixDeclaration")
                .field(
                    "prefix_token",
                    &support::DebugSyntaxResult(self.prefix_token()),
                )
                .field(
                    "namespace_token",
                    &support::DebugSyntaxResult(self.namespace_token()),
                )
                .field("iri_token", &support::DebugSyntaxResult(self.iri_token()))
                .field("dot_token", &support::DebugSyntaxResult(self.dot_token()))
                .finish()
        } else {
            f.debug_struct("TurtlePrefixDeclaration").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtlePrefixDeclaration> for SyntaxNode {
    fn from(n: TurtlePrefixDeclaration) -> Self {
        n.syntax
    }
}
impl From<TurtlePrefixDeclaration> for SyntaxElement {
    fn from(n: TurtlePrefixDeclaration) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtlePrefixedName {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_PREFIXED_NAME as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_PREFIXED_NAME
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtlePrefixedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtlePrefixedName")
                .field("value", &support::DebugSyntaxResult(self.value()))
                .finish()
        } else {
            f.debug_struct("TurtlePrefixedName").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtlePrefixedName> for SyntaxNode {
    fn from(n: TurtlePrefixedName) -> Self {
        n.syntax
    }
}
impl From<TurtlePrefixedName> for SyntaxElement {
    fn from(n: TurtlePrefixedName) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleRdfLiteral {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_RDF_LITERAL as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_RDF_LITERAL
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleRdfLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleRdfLiteral")
                .field("value", &support::DebugSyntaxResult(self.value()))
                .field(
                    "language_token",
                    &support::DebugOptionalElement(self.language_token()),
                )
                .field("datatype", &support::DebugOptionalElement(self.datatype()))
                .finish()
        } else {
            f.debug_struct("TurtleRdfLiteral").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleRdfLiteral> for SyntaxNode {
    fn from(n: TurtleRdfLiteral) -> Self {
        n.syntax
    }
}
impl From<TurtleRdfLiteral> for SyntaxElement {
    fn from(n: TurtleRdfLiteral) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleRoot {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_ROOT as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_ROOT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleRoot")
                .field(
                    "bom_token",
                    &support::DebugOptionalElement(self.bom_token()),
                )
                .field("statements", &self.statements())
                .field("eof_token", &support::DebugSyntaxResult(self.eof_token()))
                .finish()
        } else {
            f.debug_struct("TurtleRoot").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleRoot> for SyntaxNode {
    fn from(n: TurtleRoot) -> Self {
        n.syntax
    }
}
impl From<TurtleRoot> for SyntaxElement {
    fn from(n: TurtleRoot) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleSparqlBaseDeclaration {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_SPARQL_BASE_DECLARATION as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_SPARQL_BASE_DECLARATION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleSparqlBaseDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleSparqlBaseDeclaration")
                .field(
                    "SPARQL_BASE_token",
                    &support::DebugSyntaxResult(self.SPARQL_BASE_token()),
                )
                .field("iri_token", &support::DebugSyntaxResult(self.iri_token()))
                .finish()
        } else {
            f.debug_struct("TurtleSparqlBaseDeclaration").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleSparqlBaseDeclaration> for SyntaxNode {
    fn from(n: TurtleSparqlBaseDeclaration) -> Self {
        n.syntax
    }
}
impl From<TurtleSparqlBaseDeclaration> for SyntaxElement {
    fn from(n: TurtleSparqlBaseDeclaration) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleSparqlPrefixDeclaration {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_SPARQL_PREFIX_DECLARATION as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_SPARQL_PREFIX_DECLARATION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleSparqlPrefixDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleSparqlPrefixDeclaration")
                .field(
                    "SPARQL_PREFIX_token",
                    &support::DebugSyntaxResult(self.SPARQL_PREFIX_token()),
                )
                .field(
                    "namespace_token",
                    &support::DebugSyntaxResult(self.namespace_token()),
                )
                .field("iri_token", &support::DebugSyntaxResult(self.iri_token()))
                .finish()
        } else {
            f.debug_struct("TurtleSparqlPrefixDeclaration").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleSparqlPrefixDeclaration> for SyntaxNode {
    fn from(n: TurtleSparqlPrefixDeclaration) -> Self {
        n.syntax
    }
}
impl From<TurtleSparqlPrefixDeclaration> for SyntaxElement {
    fn from(n: TurtleSparqlPrefixDeclaration) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleString {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_STRING as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_STRING
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleString")
                .field("value", &support::DebugSyntaxResult(self.value()))
                .finish()
        } else {
            f.debug_struct("TurtleString").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleString> for SyntaxNode {
    fn from(n: TurtleString) -> Self {
        n.syntax
    }
}
impl From<TurtleString> for SyntaxElement {
    fn from(n: TurtleString) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleSubject {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_SUBJECT as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_SUBJECT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleSubject")
                .field(
                    "any_turtle_subject",
                    &support::DebugSyntaxResult(self.any_turtle_subject()),
                )
                .finish()
        } else {
            f.debug_struct("TurtleSubject").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleSubject> for SyntaxNode {
    fn from(n: TurtleSubject) -> Self {
        n.syntax
    }
}
impl From<TurtleSubject> for SyntaxElement {
    fn from(n: TurtleSubject) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleTriples {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_TRIPLES as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_TRIPLES
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleTriples {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleTriples")
                .field("subject", &support::DebugSyntaxResult(self.subject()))
                .field("predicates", &support::DebugSyntaxResult(self.predicates()))
                .field("dot_token", &support::DebugSyntaxResult(self.dot_token()))
                .finish()
        } else {
            f.debug_struct("TurtleTriples").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleTriples> for SyntaxNode {
    fn from(n: TurtleTriples) -> Self {
        n.syntax
    }
}
impl From<TurtleTriples> for SyntaxElement {
    fn from(n: TurtleTriples) -> Self {
        n.syntax.into()
    }
}
impl AstNode for TurtleVerb {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_VERB as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_VERB
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleVerb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        thread_local! { static DEPTH : std :: cell :: Cell < u8 > = const { std :: cell :: Cell :: new (0) } };
        let current_depth = DEPTH.get();
        let result = if current_depth < 16 {
            DEPTH.set(current_depth + 1);
            f.debug_struct("TurtleVerb")
                .field(
                    "any_turtle_verb",
                    &support::DebugSyntaxResult(self.any_turtle_verb()),
                )
                .finish()
        } else {
            f.debug_struct("TurtleVerb").finish()
        };
        DEPTH.set(current_depth);
        result
    }
}
impl From<TurtleVerb> for SyntaxNode {
    fn from(n: TurtleVerb) -> Self {
        n.syntax
    }
}
impl From<TurtleVerb> for SyntaxElement {
    fn from(n: TurtleVerb) -> Self {
        n.syntax.into()
    }
}
impl From<TurtleBaseDeclaration> for AnyTurtleDirective {
    fn from(node: TurtleBaseDeclaration) -> Self {
        Self::TurtleBaseDeclaration(node)
    }
}
impl From<TurtlePrefixDeclaration> for AnyTurtleDirective {
    fn from(node: TurtlePrefixDeclaration) -> Self {
        Self::TurtlePrefixDeclaration(node)
    }
}
impl From<TurtleSparqlBaseDeclaration> for AnyTurtleDirective {
    fn from(node: TurtleSparqlBaseDeclaration) -> Self {
        Self::TurtleSparqlBaseDeclaration(node)
    }
}
impl From<TurtleSparqlPrefixDeclaration> for AnyTurtleDirective {
    fn from(node: TurtleSparqlPrefixDeclaration) -> Self {
        Self::TurtleSparqlPrefixDeclaration(node)
    }
}
impl AstNode for AnyTurtleDirective {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> = TurtleBaseDeclaration::KIND_SET
        .union(TurtlePrefixDeclaration::KIND_SET)
        .union(TurtleSparqlBaseDeclaration::KIND_SET)
        .union(TurtleSparqlPrefixDeclaration::KIND_SET);
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            TURTLE_BASE_DECLARATION
                | TURTLE_PREFIX_DECLARATION
                | TURTLE_SPARQL_BASE_DECLARATION
                | TURTLE_SPARQL_PREFIX_DECLARATION
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            TURTLE_BASE_DECLARATION => {
                Self::TurtleBaseDeclaration(TurtleBaseDeclaration { syntax })
            }
            TURTLE_PREFIX_DECLARATION => {
                Self::TurtlePrefixDeclaration(TurtlePrefixDeclaration { syntax })
            }
            TURTLE_SPARQL_BASE_DECLARATION => {
                Self::TurtleSparqlBaseDeclaration(TurtleSparqlBaseDeclaration { syntax })
            }
            TURTLE_SPARQL_PREFIX_DECLARATION => {
                Self::TurtleSparqlPrefixDeclaration(TurtleSparqlPrefixDeclaration { syntax })
            }
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::TurtleBaseDeclaration(it) => it.syntax(),
            Self::TurtlePrefixDeclaration(it) => it.syntax(),
            Self::TurtleSparqlBaseDeclaration(it) => it.syntax(),
            Self::TurtleSparqlPrefixDeclaration(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            Self::TurtleBaseDeclaration(it) => it.into_syntax(),
            Self::TurtlePrefixDeclaration(it) => it.into_syntax(),
            Self::TurtleSparqlBaseDeclaration(it) => it.into_syntax(),
            Self::TurtleSparqlPrefixDeclaration(it) => it.into_syntax(),
        }
    }
}
impl std::fmt::Debug for AnyTurtleDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TurtleBaseDeclaration(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtlePrefixDeclaration(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleSparqlBaseDeclaration(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleSparqlPrefixDeclaration(it) => std::fmt::Debug::fmt(it, f),
        }
    }
}
impl From<AnyTurtleDirective> for SyntaxNode {
    fn from(n: AnyTurtleDirective) -> Self {
        match n {
            AnyTurtleDirective::TurtleBaseDeclaration(it) => it.into_syntax(),
            AnyTurtleDirective::TurtlePrefixDeclaration(it) => it.into_syntax(),
            AnyTurtleDirective::TurtleSparqlBaseDeclaration(it) => it.into_syntax(),
            AnyTurtleDirective::TurtleSparqlPrefixDeclaration(it) => it.into_syntax(),
        }
    }
}
impl From<AnyTurtleDirective> for SyntaxElement {
    fn from(n: AnyTurtleDirective) -> Self {
        let node: SyntaxNode = n.into();
        node.into()
    }
}
impl From<TurtleBogus> for AnyTurtleIriValue {
    fn from(node: TurtleBogus) -> Self {
        Self::TurtleBogus(node)
    }
}
impl From<TurtlePrefixedName> for AnyTurtleIriValue {
    fn from(node: TurtlePrefixedName) -> Self {
        Self::TurtlePrefixedName(node)
    }
}
impl AstNode for AnyTurtleIriValue {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        TurtleBogus::KIND_SET.union(TurtlePrefixedName::KIND_SET);
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, TURTLE_BOGUS | TURTLE_PREFIXED_NAME)
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            TURTLE_BOGUS => Self::TurtleBogus(TurtleBogus { syntax }),
            TURTLE_PREFIXED_NAME => Self::TurtlePrefixedName(TurtlePrefixedName { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::TurtleBogus(it) => it.syntax(),
            Self::TurtlePrefixedName(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            Self::TurtleBogus(it) => it.into_syntax(),
            Self::TurtlePrefixedName(it) => it.into_syntax(),
        }
    }
}
impl std::fmt::Debug for AnyTurtleIriValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TurtleBogus(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtlePrefixedName(it) => std::fmt::Debug::fmt(it, f),
        }
    }
}
impl From<AnyTurtleIriValue> for SyntaxNode {
    fn from(n: AnyTurtleIriValue) -> Self {
        match n {
            AnyTurtleIriValue::TurtleBogus(it) => it.into_syntax(),
            AnyTurtleIriValue::TurtlePrefixedName(it) => it.into_syntax(),
        }
    }
}
impl From<AnyTurtleIriValue> for SyntaxElement {
    fn from(n: AnyTurtleIriValue) -> Self {
        let node: SyntaxNode = n.into();
        node.into()
    }
}
impl From<TurtleBlankNode> for AnyTurtleObject {
    fn from(node: TurtleBlankNode) -> Self {
        Self::TurtleBlankNode(node)
    }
}
impl From<TurtleBlankNodePropertyList> for AnyTurtleObject {
    fn from(node: TurtleBlankNodePropertyList) -> Self {
        Self::TurtleBlankNodePropertyList(node)
    }
}
impl From<TurtleBogus> for AnyTurtleObject {
    fn from(node: TurtleBogus) -> Self {
        Self::TurtleBogus(node)
    }
}
impl From<TurtleBooleanLiteral> for AnyTurtleObject {
    fn from(node: TurtleBooleanLiteral) -> Self {
        Self::TurtleBooleanLiteral(node)
    }
}
impl From<TurtleCollection> for AnyTurtleObject {
    fn from(node: TurtleCollection) -> Self {
        Self::TurtleCollection(node)
    }
}
impl From<TurtleIri> for AnyTurtleObject {
    fn from(node: TurtleIri) -> Self {
        Self::TurtleIri(node)
    }
}
impl From<TurtleNumericLiteral> for AnyTurtleObject {
    fn from(node: TurtleNumericLiteral) -> Self {
        Self::TurtleNumericLiteral(node)
    }
}
impl From<TurtleRdfLiteral> for AnyTurtleObject {
    fn from(node: TurtleRdfLiteral) -> Self {
        Self::TurtleRdfLiteral(node)
    }
}
impl AstNode for AnyTurtleObject {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> = TurtleBlankNode::KIND_SET
        .union(TurtleBlankNodePropertyList::KIND_SET)
        .union(TurtleBogus::KIND_SET)
        .union(TurtleBooleanLiteral::KIND_SET)
        .union(TurtleCollection::KIND_SET)
        .union(TurtleIri::KIND_SET)
        .union(TurtleNumericLiteral::KIND_SET)
        .union(TurtleRdfLiteral::KIND_SET);
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            TURTLE_BLANK_NODE
                | TURTLE_BLANK_NODE_PROPERTY_LIST
                | TURTLE_BOGUS
                | TURTLE_BOOLEAN_LITERAL
                | TURTLE_COLLECTION
                | TURTLE_IRI
                | TURTLE_NUMERIC_LITERAL
                | TURTLE_RDF_LITERAL
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            TURTLE_BLANK_NODE => Self::TurtleBlankNode(TurtleBlankNode { syntax }),
            TURTLE_BLANK_NODE_PROPERTY_LIST => {
                Self::TurtleBlankNodePropertyList(TurtleBlankNodePropertyList { syntax })
            }
            TURTLE_BOGUS => Self::TurtleBogus(TurtleBogus { syntax }),
            TURTLE_BOOLEAN_LITERAL => Self::TurtleBooleanLiteral(TurtleBooleanLiteral { syntax }),
            TURTLE_COLLECTION => Self::TurtleCollection(TurtleCollection { syntax }),
            TURTLE_IRI => Self::TurtleIri(TurtleIri { syntax }),
            TURTLE_NUMERIC_LITERAL => Self::TurtleNumericLiteral(TurtleNumericLiteral { syntax }),
            TURTLE_RDF_LITERAL => Self::TurtleRdfLiteral(TurtleRdfLiteral { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::TurtleBlankNode(it) => it.syntax(),
            Self::TurtleBlankNodePropertyList(it) => it.syntax(),
            Self::TurtleBogus(it) => it.syntax(),
            Self::TurtleBooleanLiteral(it) => it.syntax(),
            Self::TurtleCollection(it) => it.syntax(),
            Self::TurtleIri(it) => it.syntax(),
            Self::TurtleNumericLiteral(it) => it.syntax(),
            Self::TurtleRdfLiteral(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            Self::TurtleBlankNode(it) => it.into_syntax(),
            Self::TurtleBlankNodePropertyList(it) => it.into_syntax(),
            Self::TurtleBogus(it) => it.into_syntax(),
            Self::TurtleBooleanLiteral(it) => it.into_syntax(),
            Self::TurtleCollection(it) => it.into_syntax(),
            Self::TurtleIri(it) => it.into_syntax(),
            Self::TurtleNumericLiteral(it) => it.into_syntax(),
            Self::TurtleRdfLiteral(it) => it.into_syntax(),
        }
    }
}
impl std::fmt::Debug for AnyTurtleObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TurtleBlankNode(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleBlankNodePropertyList(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleBogus(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleBooleanLiteral(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleCollection(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleIri(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleNumericLiteral(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleRdfLiteral(it) => std::fmt::Debug::fmt(it, f),
        }
    }
}
impl From<AnyTurtleObject> for SyntaxNode {
    fn from(n: AnyTurtleObject) -> Self {
        match n {
            AnyTurtleObject::TurtleBlankNode(it) => it.into_syntax(),
            AnyTurtleObject::TurtleBlankNodePropertyList(it) => it.into_syntax(),
            AnyTurtleObject::TurtleBogus(it) => it.into_syntax(),
            AnyTurtleObject::TurtleBooleanLiteral(it) => it.into_syntax(),
            AnyTurtleObject::TurtleCollection(it) => it.into_syntax(),
            AnyTurtleObject::TurtleIri(it) => it.into_syntax(),
            AnyTurtleObject::TurtleNumericLiteral(it) => it.into_syntax(),
            AnyTurtleObject::TurtleRdfLiteral(it) => it.into_syntax(),
        }
    }
}
impl From<AnyTurtleObject> for SyntaxElement {
    fn from(n: AnyTurtleObject) -> Self {
        let node: SyntaxNode = n.into();
        node.into()
    }
}
impl From<TurtleBogusStatement> for AnyTurtleStatement {
    fn from(node: TurtleBogusStatement) -> Self {
        Self::TurtleBogusStatement(node)
    }
}
impl From<TurtleTriples> for AnyTurtleStatement {
    fn from(node: TurtleTriples) -> Self {
        Self::TurtleTriples(node)
    }
}
impl AstNode for AnyTurtleStatement {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> = AnyTurtleDirective::KIND_SET
        .union(TurtleBogusStatement::KIND_SET)
        .union(TurtleTriples::KIND_SET);
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            TURTLE_BOGUS_STATEMENT | TURTLE_TRIPLES => true,
            k if AnyTurtleDirective::can_cast(k) => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            TURTLE_BOGUS_STATEMENT => Self::TurtleBogusStatement(TurtleBogusStatement { syntax }),
            TURTLE_TRIPLES => Self::TurtleTriples(TurtleTriples { syntax }),
            _ => {
                if let Some(any_turtle_directive) = AnyTurtleDirective::cast(syntax) {
                    return Some(Self::AnyTurtleDirective(any_turtle_directive));
                }
                return None;
            }
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::TurtleBogusStatement(it) => it.syntax(),
            Self::TurtleTriples(it) => it.syntax(),
            Self::AnyTurtleDirective(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            Self::TurtleBogusStatement(it) => it.into_syntax(),
            Self::TurtleTriples(it) => it.into_syntax(),
            Self::AnyTurtleDirective(it) => it.into_syntax(),
        }
    }
}
impl std::fmt::Debug for AnyTurtleStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnyTurtleDirective(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleBogusStatement(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleTriples(it) => std::fmt::Debug::fmt(it, f),
        }
    }
}
impl From<AnyTurtleStatement> for SyntaxNode {
    fn from(n: AnyTurtleStatement) -> Self {
        match n {
            AnyTurtleStatement::AnyTurtleDirective(it) => it.into_syntax(),
            AnyTurtleStatement::TurtleBogusStatement(it) => it.into_syntax(),
            AnyTurtleStatement::TurtleTriples(it) => it.into_syntax(),
        }
    }
}
impl From<AnyTurtleStatement> for SyntaxElement {
    fn from(n: AnyTurtleStatement) -> Self {
        let node: SyntaxNode = n.into();
        node.into()
    }
}
impl From<TurtleBlankNode> for AnyTurtleSubject {
    fn from(node: TurtleBlankNode) -> Self {
        Self::TurtleBlankNode(node)
    }
}
impl From<TurtleBlankNodePropertyList> for AnyTurtleSubject {
    fn from(node: TurtleBlankNodePropertyList) -> Self {
        Self::TurtleBlankNodePropertyList(node)
    }
}
impl From<TurtleCollection> for AnyTurtleSubject {
    fn from(node: TurtleCollection) -> Self {
        Self::TurtleCollection(node)
    }
}
impl From<TurtleIri> for AnyTurtleSubject {
    fn from(node: TurtleIri) -> Self {
        Self::TurtleIri(node)
    }
}
impl AstNode for AnyTurtleSubject {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> = TurtleBlankNode::KIND_SET
        .union(TurtleBlankNodePropertyList::KIND_SET)
        .union(TurtleCollection::KIND_SET)
        .union(TurtleIri::KIND_SET);
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            TURTLE_BLANK_NODE | TURTLE_BLANK_NODE_PROPERTY_LIST | TURTLE_COLLECTION | TURTLE_IRI
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            TURTLE_BLANK_NODE => Self::TurtleBlankNode(TurtleBlankNode { syntax }),
            TURTLE_BLANK_NODE_PROPERTY_LIST => {
                Self::TurtleBlankNodePropertyList(TurtleBlankNodePropertyList { syntax })
            }
            TURTLE_COLLECTION => Self::TurtleCollection(TurtleCollection { syntax }),
            TURTLE_IRI => Self::TurtleIri(TurtleIri { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::TurtleBlankNode(it) => it.syntax(),
            Self::TurtleBlankNodePropertyList(it) => it.syntax(),
            Self::TurtleCollection(it) => it.syntax(),
            Self::TurtleIri(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            Self::TurtleBlankNode(it) => it.into_syntax(),
            Self::TurtleBlankNodePropertyList(it) => it.into_syntax(),
            Self::TurtleCollection(it) => it.into_syntax(),
            Self::TurtleIri(it) => it.into_syntax(),
        }
    }
}
impl std::fmt::Debug for AnyTurtleSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TurtleBlankNode(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleBlankNodePropertyList(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleCollection(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleIri(it) => std::fmt::Debug::fmt(it, f),
        }
    }
}
impl From<AnyTurtleSubject> for SyntaxNode {
    fn from(n: AnyTurtleSubject) -> Self {
        match n {
            AnyTurtleSubject::TurtleBlankNode(it) => it.into_syntax(),
            AnyTurtleSubject::TurtleBlankNodePropertyList(it) => it.into_syntax(),
            AnyTurtleSubject::TurtleCollection(it) => it.into_syntax(),
            AnyTurtleSubject::TurtleIri(it) => it.into_syntax(),
        }
    }
}
impl From<AnyTurtleSubject> for SyntaxElement {
    fn from(n: AnyTurtleSubject) -> Self {
        let node: SyntaxNode = n.into();
        node.into()
    }
}
impl From<TurtleBogus> for AnyTurtleVerb {
    fn from(node: TurtleBogus) -> Self {
        Self::TurtleBogus(node)
    }
}
impl From<TurtleIri> for AnyTurtleVerb {
    fn from(node: TurtleIri) -> Self {
        Self::TurtleIri(node)
    }
}
impl AstNode for AnyTurtleVerb {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> = TurtleBogus::KIND_SET.union(TurtleIri::KIND_SET);
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, TURTLE_BOGUS | TURTLE_IRI)
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            TURTLE_BOGUS => Self::TurtleBogus(TurtleBogus { syntax }),
            TURTLE_IRI => Self::TurtleIri(TurtleIri { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::TurtleBogus(it) => it.syntax(),
            Self::TurtleIri(it) => it.syntax(),
        }
    }
    fn into_syntax(self) -> SyntaxNode {
        match self {
            Self::TurtleBogus(it) => it.into_syntax(),
            Self::TurtleIri(it) => it.into_syntax(),
        }
    }
}
impl std::fmt::Debug for AnyTurtleVerb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TurtleBogus(it) => std::fmt::Debug::fmt(it, f),
            Self::TurtleIri(it) => std::fmt::Debug::fmt(it, f),
        }
    }
}
impl From<AnyTurtleVerb> for SyntaxNode {
    fn from(n: AnyTurtleVerb) -> Self {
        match n {
            AnyTurtleVerb::TurtleBogus(it) => it.into_syntax(),
            AnyTurtleVerb::TurtleIri(it) => it.into_syntax(),
        }
    }
}
impl From<AnyTurtleVerb> for SyntaxElement {
    fn from(n: AnyTurtleVerb) -> Self {
        let node: SyntaxNode = n.into();
        node.into()
    }
}
impl std::fmt::Display for AnyTurtleDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for AnyTurtleIriValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for AnyTurtleObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for AnyTurtleStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for AnyTurtleSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for AnyTurtleVerb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleBaseDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleBlankNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleBlankNodePropertyList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleBooleanLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleDatatypeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleIri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleNumericLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtlePredicateObjectList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtlePredicateObjectPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtlePrefixDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtlePrefixedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleRdfLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleSparqlBaseDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleSparqlPrefixDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleTriples {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TurtleVerb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub struct TurtleBogus {
    syntax: SyntaxNode,
}
impl TurtleBogus {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn items(&self) -> SyntaxElementChildren {
        support::elements(&self.syntax)
    }
}
impl AstNode for TurtleBogus {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_BOGUS as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_BOGUS
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleBogus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TurtleBogus")
            .field("items", &DebugSyntaxElementChildren(self.items()))
            .finish()
    }
}
impl From<TurtleBogus> for SyntaxNode {
    fn from(n: TurtleBogus) -> Self {
        n.syntax
    }
}
impl From<TurtleBogus> for SyntaxElement {
    fn from(n: TurtleBogus) -> Self {
        n.syntax.into()
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub struct TurtleBogusStatement {
    syntax: SyntaxNode,
}
impl TurtleBogusStatement {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub const unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
    pub fn items(&self) -> SyntaxElementChildren {
        support::elements(&self.syntax)
    }
}
impl AstNode for TurtleBogusStatement {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_BOGUS_STATEMENT as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_BOGUS_STATEMENT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax
    }
}
impl std::fmt::Debug for TurtleBogusStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TurtleBogusStatement")
            .field("items", &DebugSyntaxElementChildren(self.items()))
            .finish()
    }
}
impl From<TurtleBogusStatement> for SyntaxNode {
    fn from(n: TurtleBogusStatement) -> Self {
        n.syntax
    }
}
impl From<TurtleBogusStatement> for SyntaxElement {
    fn from(n: TurtleBogusStatement) -> Self {
        n.syntax.into()
    }
}
biome_rowan::declare_node_union! { pub AnyTurtleBogusNode = TurtleBogus | TurtleBogusStatement }
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TurtleCollectionObjectList {
    syntax_list: SyntaxList,
}
impl TurtleCollectionObjectList {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self {
            syntax_list: syntax.into_list(),
        }
    }
}
impl AstNode for TurtleCollectionObjectList {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_COLLECTION_OBJECT_LIST as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_COLLECTION_OBJECT_LIST
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self {
                syntax_list: syntax.into_list(),
            })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        self.syntax_list.node()
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax_list.into_node()
    }
}
impl Serialize for TurtleCollectionObjectList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for e in self.iter() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }
}
impl AstNodeList for TurtleCollectionObjectList {
    type Language = Language;
    type Node = TurtleObject;
    fn syntax_list(&self) -> &SyntaxList {
        &self.syntax_list
    }
    fn into_syntax_list(self) -> SyntaxList {
        self.syntax_list
    }
}
impl Debug for TurtleCollectionObjectList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("TurtleCollectionObjectList ")?;
        f.debug_list().entries(self.iter()).finish()
    }
}
impl IntoIterator for &TurtleCollectionObjectList {
    type Item = TurtleObject;
    type IntoIter = AstNodeListIterator<Language, TurtleObject>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl IntoIterator for TurtleCollectionObjectList {
    type Item = TurtleObject;
    type IntoIter = AstNodeListIterator<Language, TurtleObject>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TurtleObjectList {
    syntax_list: SyntaxList,
}
impl TurtleObjectList {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self {
            syntax_list: syntax.into_list(),
        }
    }
}
impl AstNode for TurtleObjectList {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_OBJECT_LIST as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_OBJECT_LIST
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self {
                syntax_list: syntax.into_list(),
            })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        self.syntax_list.node()
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax_list.into_node()
    }
}
impl Serialize for TurtleObjectList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for e in self.iter() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }
}
impl AstSeparatedList for TurtleObjectList {
    type Language = Language;
    type Node = TurtleObject;
    fn syntax_list(&self) -> &SyntaxList {
        &self.syntax_list
    }
    fn into_syntax_list(self) -> SyntaxList {
        self.syntax_list
    }
}
impl Debug for TurtleObjectList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("TurtleObjectList ")?;
        f.debug_list().entries(self.elements()).finish()
    }
}
impl IntoIterator for TurtleObjectList {
    type Item = SyntaxResult<TurtleObject>;
    type IntoIter = AstSeparatedListNodesIterator<Language, TurtleObject>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl IntoIterator for &TurtleObjectList {
    type Item = SyntaxResult<TurtleObject>;
    type IntoIter = AstSeparatedListNodesIterator<Language, TurtleObject>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TurtlePredicateObjectPairList {
    syntax_list: SyntaxList,
}
impl TurtlePredicateObjectPairList {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self {
            syntax_list: syntax.into_list(),
        }
    }
}
impl AstNode for TurtlePredicateObjectPairList {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_PREDICATE_OBJECT_PAIR_LIST as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_PREDICATE_OBJECT_PAIR_LIST
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self {
                syntax_list: syntax.into_list(),
            })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        self.syntax_list.node()
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax_list.into_node()
    }
}
impl Serialize for TurtlePredicateObjectPairList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for e in self.iter() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }
}
impl AstSeparatedList for TurtlePredicateObjectPairList {
    type Language = Language;
    type Node = TurtlePredicateObjectPair;
    fn syntax_list(&self) -> &SyntaxList {
        &self.syntax_list
    }
    fn into_syntax_list(self) -> SyntaxList {
        self.syntax_list
    }
}
impl Debug for TurtlePredicateObjectPairList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("TurtlePredicateObjectPairList ")?;
        f.debug_list().entries(self.elements()).finish()
    }
}
impl IntoIterator for TurtlePredicateObjectPairList {
    type Item = SyntaxResult<TurtlePredicateObjectPair>;
    type IntoIter = AstSeparatedListNodesIterator<Language, TurtlePredicateObjectPair>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl IntoIterator for &TurtlePredicateObjectPairList {
    type Item = SyntaxResult<TurtlePredicateObjectPair>;
    type IntoIter = AstSeparatedListNodesIterator<Language, TurtlePredicateObjectPair>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TurtleStatementList {
    syntax_list: SyntaxList,
}
impl TurtleStatementList {
    #[doc = r" Create an AstNode from a SyntaxNode without checking its kind"]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r" This function must be guarded with a call to [AstNode::can_cast]"]
    #[doc = r" or a match on [SyntaxNode::kind]"]
    #[inline]
    pub unsafe fn new_unchecked(syntax: SyntaxNode) -> Self {
        Self {
            syntax_list: syntax.into_list(),
        }
    }
}
impl AstNode for TurtleStatementList {
    type Language = Language;
    const KIND_SET: SyntaxKindSet<Language> =
        SyntaxKindSet::from_raw(RawSyntaxKind(TURTLE_STATEMENT_LIST as u16));
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TURTLE_STATEMENT_LIST
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self {
                syntax_list: syntax.into_list(),
            })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        self.syntax_list.node()
    }
    fn into_syntax(self) -> SyntaxNode {
        self.syntax_list.into_node()
    }
}
impl Serialize for TurtleStatementList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for e in self.iter() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }
}
impl AstNodeList for TurtleStatementList {
    type Language = Language;
    type Node = AnyTurtleStatement;
    fn syntax_list(&self) -> &SyntaxList {
        &self.syntax_list
    }
    fn into_syntax_list(self) -> SyntaxList {
        self.syntax_list
    }
}
impl Debug for TurtleStatementList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("TurtleStatementList ")?;
        f.debug_list().entries(self.iter()).finish()
    }
}
impl IntoIterator for &TurtleStatementList {
    type Item = AnyTurtleStatement;
    type IntoIter = AstNodeListIterator<Language, AnyTurtleStatement>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl IntoIterator for TurtleStatementList {
    type Item = AnyTurtleStatement;
    type IntoIter = AstNodeListIterator<Language, AnyTurtleStatement>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
#[derive(Clone)]
pub struct DebugSyntaxElementChildren(pub SyntaxElementChildren);
impl Debug for DebugSyntaxElementChildren {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.clone().0.map(DebugSyntaxElement))
            .finish()
    }
}
struct DebugSyntaxElement(SyntaxElement);
impl Debug for DebugSyntaxElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            SyntaxElement::Node(node) => {
                map_syntax_node ! (node . clone () , node => std :: fmt :: Debug :: fmt (& node , f))
            }
            SyntaxElement::Token(token) => Debug::fmt(token, f),
        }
    }
}
