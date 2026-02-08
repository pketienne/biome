//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

#![allow(clippy::use_self)]
#![expect(clippy::default_constructed_unit_structs)]
use crate::{
    AsFormat, FormatBogusNodeRule, FormatNodeRule, IntoFormat, TurtleFormatContext, TurtleFormatter,
};
use biome_formatter::{FormatOwnedWithRule, FormatRefWithRule, FormatResult, FormatRule};
impl FormatRule<biome_turtle_syntax::TurtleBaseDeclaration>
    for crate::js::declarations::base_declaration::FormatTurtleBaseDeclaration
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleBaseDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleBaseDeclaration>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBaseDeclaration {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleBaseDeclaration,
        crate::js::declarations::base_declaration::FormatTurtleBaseDeclaration,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::declarations::base_declaration::FormatTurtleBaseDeclaration::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBaseDeclaration {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleBaseDeclaration,
        crate::js::declarations::base_declaration::FormatTurtleBaseDeclaration,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::declarations::base_declaration::FormatTurtleBaseDeclaration::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleBlankNode>
    for crate::js::auxiliary::blank_node::FormatTurtleBlankNode
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleBlankNode,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleBlankNode>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBlankNode {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleBlankNode,
        crate::js::auxiliary::blank_node::FormatTurtleBlankNode,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::blank_node::FormatTurtleBlankNode::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBlankNode {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleBlankNode,
        crate::js::auxiliary::blank_node::FormatTurtleBlankNode,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::blank_node::FormatTurtleBlankNode::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleBlankNodePropertyList>
    for crate::js::auxiliary::blank_node_property_list::FormatTurtleBlankNodePropertyList
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleBlankNodePropertyList,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleBlankNodePropertyList>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBlankNodePropertyList {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleBlankNodePropertyList,
        crate::js::auxiliary::blank_node_property_list::FormatTurtleBlankNodePropertyList,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule :: new (self , crate :: js :: auxiliary :: blank_node_property_list :: FormatTurtleBlankNodePropertyList :: default ())
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBlankNodePropertyList {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleBlankNodePropertyList,
        crate::js::auxiliary::blank_node_property_list::FormatTurtleBlankNodePropertyList,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule :: new (self , crate :: js :: auxiliary :: blank_node_property_list :: FormatTurtleBlankNodePropertyList :: default ())
    }
}
impl FormatRule<biome_turtle_syntax::TurtleBooleanLiteral>
    for crate::js::value::boolean_literal::FormatTurtleBooleanLiteral
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleBooleanLiteral,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleBooleanLiteral>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBooleanLiteral {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleBooleanLiteral,
        crate::js::value::boolean_literal::FormatTurtleBooleanLiteral,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::value::boolean_literal::FormatTurtleBooleanLiteral::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBooleanLiteral {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleBooleanLiteral,
        crate::js::value::boolean_literal::FormatTurtleBooleanLiteral,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::value::boolean_literal::FormatTurtleBooleanLiteral::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleCollection>
    for crate::js::auxiliary::collection::FormatTurtleCollection
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleCollection,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleCollection>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleCollection {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleCollection,
        crate::js::auxiliary::collection::FormatTurtleCollection,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::collection::FormatTurtleCollection::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleCollection {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleCollection,
        crate::js::auxiliary::collection::FormatTurtleCollection,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::collection::FormatTurtleCollection::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleDatatypeAnnotation>
    for crate::js::auxiliary::datatype_annotation::FormatTurtleDatatypeAnnotation
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleDatatypeAnnotation,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleDatatypeAnnotation>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleDatatypeAnnotation {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleDatatypeAnnotation,
        crate::js::auxiliary::datatype_annotation::FormatTurtleDatatypeAnnotation,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::datatype_annotation::FormatTurtleDatatypeAnnotation::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleDatatypeAnnotation {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleDatatypeAnnotation,
        crate::js::auxiliary::datatype_annotation::FormatTurtleDatatypeAnnotation,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::datatype_annotation::FormatTurtleDatatypeAnnotation::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleIri> for crate::js::auxiliary::iri::FormatTurtleIri {
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleIri,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleIri>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleIri {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleIri,
        crate::js::auxiliary::iri::FormatTurtleIri,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, crate::js::auxiliary::iri::FormatTurtleIri::default())
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleIri {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleIri,
        crate::js::auxiliary::iri::FormatTurtleIri,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, crate::js::auxiliary::iri::FormatTurtleIri::default())
    }
}
impl FormatRule<biome_turtle_syntax::TurtleNumericLiteral>
    for crate::js::value::numeric_literal::FormatTurtleNumericLiteral
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleNumericLiteral,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleNumericLiteral>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleNumericLiteral {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleNumericLiteral,
        crate::js::value::numeric_literal::FormatTurtleNumericLiteral,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::value::numeric_literal::FormatTurtleNumericLiteral::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleNumericLiteral {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleNumericLiteral,
        crate::js::value::numeric_literal::FormatTurtleNumericLiteral,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::value::numeric_literal::FormatTurtleNumericLiteral::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleObject>
    for crate::js::auxiliary::object::FormatTurtleObject
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleObject,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleObject>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleObject {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleObject,
        crate::js::auxiliary::object::FormatTurtleObject,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::object::FormatTurtleObject::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleObject {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleObject,
        crate::js::auxiliary::object::FormatTurtleObject,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::object::FormatTurtleObject::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtlePredicateObjectList>
    for crate::js::auxiliary::predicate_object_list::FormatTurtlePredicateObjectList
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtlePredicateObjectList,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtlePredicateObjectList>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtlePredicateObjectList {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtlePredicateObjectList,
        crate::js::auxiliary::predicate_object_list::FormatTurtlePredicateObjectList,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::predicate_object_list::FormatTurtlePredicateObjectList::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtlePredicateObjectList {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtlePredicateObjectList,
        crate::js::auxiliary::predicate_object_list::FormatTurtlePredicateObjectList,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::predicate_object_list::FormatTurtlePredicateObjectList::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtlePredicateObjectPair>
    for crate::js::auxiliary::predicate_object_pair::FormatTurtlePredicateObjectPair
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtlePredicateObjectPair,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtlePredicateObjectPair>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtlePredicateObjectPair {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtlePredicateObjectPair,
        crate::js::auxiliary::predicate_object_pair::FormatTurtlePredicateObjectPair,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::predicate_object_pair::FormatTurtlePredicateObjectPair::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtlePredicateObjectPair {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtlePredicateObjectPair,
        crate::js::auxiliary::predicate_object_pair::FormatTurtlePredicateObjectPair,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::predicate_object_pair::FormatTurtlePredicateObjectPair::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtlePrefixDeclaration>
    for crate::js::declarations::prefix_declaration::FormatTurtlePrefixDeclaration
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtlePrefixDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtlePrefixDeclaration>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtlePrefixDeclaration {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtlePrefixDeclaration,
        crate::js::declarations::prefix_declaration::FormatTurtlePrefixDeclaration,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::declarations::prefix_declaration::FormatTurtlePrefixDeclaration::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtlePrefixDeclaration {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtlePrefixDeclaration,
        crate::js::declarations::prefix_declaration::FormatTurtlePrefixDeclaration,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::declarations::prefix_declaration::FormatTurtlePrefixDeclaration::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtlePrefixedName>
    for crate::js::auxiliary::prefixed_name::FormatTurtlePrefixedName
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtlePrefixedName,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtlePrefixedName>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtlePrefixedName {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtlePrefixedName,
        crate::js::auxiliary::prefixed_name::FormatTurtlePrefixedName,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::prefixed_name::FormatTurtlePrefixedName::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtlePrefixedName {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtlePrefixedName,
        crate::js::auxiliary::prefixed_name::FormatTurtlePrefixedName,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::prefixed_name::FormatTurtlePrefixedName::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleRdfLiteral>
    for crate::js::value::rdf_literal::FormatTurtleRdfLiteral
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleRdfLiteral,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleRdfLiteral>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleRdfLiteral {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleRdfLiteral,
        crate::js::value::rdf_literal::FormatTurtleRdfLiteral,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::value::rdf_literal::FormatTurtleRdfLiteral::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleRdfLiteral {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleRdfLiteral,
        crate::js::value::rdf_literal::FormatTurtleRdfLiteral,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::value::rdf_literal::FormatTurtleRdfLiteral::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleRoot> for crate::js::auxiliary::root::FormatTurtleRoot {
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleRoot,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleRoot>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleRoot {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleRoot,
        crate::js::auxiliary::root::FormatTurtleRoot,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::root::FormatTurtleRoot::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleRoot {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleRoot,
        crate::js::auxiliary::root::FormatTurtleRoot,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::root::FormatTurtleRoot::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleSparqlBaseDeclaration>
    for crate::js::declarations::sparql_base_declaration::FormatTurtleSparqlBaseDeclaration
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleSparqlBaseDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleSparqlBaseDeclaration>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleSparqlBaseDeclaration {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleSparqlBaseDeclaration,
        crate::js::declarations::sparql_base_declaration::FormatTurtleSparqlBaseDeclaration,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule :: new (self , crate :: js :: declarations :: sparql_base_declaration :: FormatTurtleSparqlBaseDeclaration :: default ())
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleSparqlBaseDeclaration {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleSparqlBaseDeclaration,
        crate::js::declarations::sparql_base_declaration::FormatTurtleSparqlBaseDeclaration,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule :: new (self , crate :: js :: declarations :: sparql_base_declaration :: FormatTurtleSparqlBaseDeclaration :: default ())
    }
}
impl FormatRule<biome_turtle_syntax::TurtleSparqlPrefixDeclaration>
    for crate::js::declarations::sparql_prefix_declaration::FormatTurtleSparqlPrefixDeclaration
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleSparqlPrefixDeclaration,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleSparqlPrefixDeclaration>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleSparqlPrefixDeclaration {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleSparqlPrefixDeclaration,
        crate::js::declarations::sparql_prefix_declaration::FormatTurtleSparqlPrefixDeclaration,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule :: new (self , crate :: js :: declarations :: sparql_prefix_declaration :: FormatTurtleSparqlPrefixDeclaration :: default ())
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleSparqlPrefixDeclaration {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleSparqlPrefixDeclaration,
        crate::js::declarations::sparql_prefix_declaration::FormatTurtleSparqlPrefixDeclaration,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule :: new (self , crate :: js :: declarations :: sparql_prefix_declaration :: FormatTurtleSparqlPrefixDeclaration :: default ())
    }
}
impl FormatRule<biome_turtle_syntax::TurtleString>
    for crate::js::auxiliary::string::FormatTurtleString
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleString,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleString>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleString {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleString,
        crate::js::auxiliary::string::FormatTurtleString,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::string::FormatTurtleString::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleString {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleString,
        crate::js::auxiliary::string::FormatTurtleString,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::string::FormatTurtleString::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleSubject>
    for crate::js::auxiliary::subject::FormatTurtleSubject
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleSubject,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleSubject>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleSubject {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleSubject,
        crate::js::auxiliary::subject::FormatTurtleSubject,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::subject::FormatTurtleSubject::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleSubject {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleSubject,
        crate::js::auxiliary::subject::FormatTurtleSubject,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::subject::FormatTurtleSubject::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleTriples>
    for crate::js::auxiliary::triples::FormatTurtleTriples
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleTriples,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleTriples>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleTriples {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleTriples,
        crate::js::auxiliary::triples::FormatTurtleTriples,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::triples::FormatTurtleTriples::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleTriples {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleTriples,
        crate::js::auxiliary::triples::FormatTurtleTriples,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::triples::FormatTurtleTriples::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleVerb> for crate::js::auxiliary::verb::FormatTurtleVerb {
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleVerb,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_turtle_syntax::TurtleVerb>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleVerb {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleVerb,
        crate::js::auxiliary::verb::FormatTurtleVerb,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::verb::FormatTurtleVerb::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleVerb {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleVerb,
        crate::js::auxiliary::verb::FormatTurtleVerb,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::verb::FormatTurtleVerb::default(),
        )
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleCollectionObjectList {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleCollectionObjectList,
        crate::js::lists::collection_object_list::FormatTurtleCollectionObjectList,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::lists::collection_object_list::FormatTurtleCollectionObjectList::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleCollectionObjectList {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleCollectionObjectList,
        crate::js::lists::collection_object_list::FormatTurtleCollectionObjectList,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::lists::collection_object_list::FormatTurtleCollectionObjectList::default(),
        )
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleObjectList {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleObjectList,
        crate::js::lists::object_list::FormatTurtleObjectList,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::lists::object_list::FormatTurtleObjectList::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleObjectList {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleObjectList,
        crate::js::lists::object_list::FormatTurtleObjectList,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::lists::object_list::FormatTurtleObjectList::default(),
        )
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtlePredicateObjectPairList {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtlePredicateObjectPairList,
        crate::js::lists::predicate_object_pair_list::FormatTurtlePredicateObjectPairList,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule :: new (self , crate :: js :: lists :: predicate_object_pair_list :: FormatTurtlePredicateObjectPairList :: default ())
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtlePredicateObjectPairList {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtlePredicateObjectPairList,
        crate::js::lists::predicate_object_pair_list::FormatTurtlePredicateObjectPairList,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule :: new (self , crate :: js :: lists :: predicate_object_pair_list :: FormatTurtlePredicateObjectPairList :: default ())
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleStatementList {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleStatementList,
        crate::js::lists::statement_list::FormatTurtleStatementList,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::lists::statement_list::FormatTurtleStatementList::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleStatementList {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleStatementList,
        crate::js::lists::statement_list::FormatTurtleStatementList,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::lists::statement_list::FormatTurtleStatementList::default(),
        )
    }
}
impl FormatRule<biome_turtle_syntax::TurtleBogus> for crate::js::bogus::bogus::FormatTurtleBogus {
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleBogus,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatBogusNodeRule::<biome_turtle_syntax::TurtleBogus>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBogus {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleBogus,
        crate::js::bogus::bogus::FormatTurtleBogus,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, crate::js::bogus::bogus::FormatTurtleBogus::default())
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBogus {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleBogus,
        crate::js::bogus::bogus::FormatTurtleBogus,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, crate::js::bogus::bogus::FormatTurtleBogus::default())
    }
}
impl FormatRule<biome_turtle_syntax::TurtleBogusStatement>
    for crate::js::bogus::bogus_statement::FormatTurtleBogusStatement
{
    type Context = TurtleFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_turtle_syntax::TurtleBogusStatement,
        f: &mut TurtleFormatter,
    ) -> FormatResult<()> {
        FormatBogusNodeRule::<biome_turtle_syntax::TurtleBogusStatement>::fmt(self, node, f)
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBogusStatement {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::TurtleBogusStatement,
        crate::js::bogus::bogus_statement::FormatTurtleBogusStatement,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::bogus::bogus_statement::FormatTurtleBogusStatement::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::TurtleBogusStatement {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::TurtleBogusStatement,
        crate::js::bogus::bogus_statement::FormatTurtleBogusStatement,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::bogus::bogus_statement::FormatTurtleBogusStatement::default(),
        )
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleDirective {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::AnyTurtleDirective,
        crate::js::any::directive::FormatAnyTurtleDirective,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::any::directive::FormatAnyTurtleDirective::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleDirective {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::AnyTurtleDirective,
        crate::js::any::directive::FormatAnyTurtleDirective,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::any::directive::FormatAnyTurtleDirective::default(),
        )
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleIriValue {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::AnyTurtleIriValue,
        crate::js::any::iri_value::FormatAnyTurtleIriValue,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::any::iri_value::FormatAnyTurtleIriValue::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleIriValue {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::AnyTurtleIriValue,
        crate::js::any::iri_value::FormatAnyTurtleIriValue,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::any::iri_value::FormatAnyTurtleIriValue::default(),
        )
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleObject {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::AnyTurtleObject,
        crate::js::any::object::FormatAnyTurtleObject,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::any::object::FormatAnyTurtleObject::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleObject {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::AnyTurtleObject,
        crate::js::any::object::FormatAnyTurtleObject,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::any::object::FormatAnyTurtleObject::default(),
        )
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleStatement {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::AnyTurtleStatement,
        crate::js::any::statement::FormatAnyTurtleStatement,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::any::statement::FormatAnyTurtleStatement::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleStatement {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::AnyTurtleStatement,
        crate::js::any::statement::FormatAnyTurtleStatement,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::any::statement::FormatAnyTurtleStatement::default(),
        )
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleSubject {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::AnyTurtleSubject,
        crate::js::any::subject::FormatAnyTurtleSubject,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::any::subject::FormatAnyTurtleSubject::default(),
        )
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleSubject {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::AnyTurtleSubject,
        crate::js::any::subject::FormatAnyTurtleSubject,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::any::subject::FormatAnyTurtleSubject::default(),
        )
    }
}
impl AsFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleVerb {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_turtle_syntax::AnyTurtleVerb,
        crate::js::any::verb::FormatAnyTurtleVerb,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, crate::js::any::verb::FormatAnyTurtleVerb::default())
    }
}
impl IntoFormat<TurtleFormatContext> for biome_turtle_syntax::AnyTurtleVerb {
    type Format = FormatOwnedWithRule<
        biome_turtle_syntax::AnyTurtleVerb,
        crate::js::any::verb::FormatAnyTurtleVerb,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, crate::js::any::verb::FormatAnyTurtleVerb::default())
    }
}
