use crate::lint;
pub type NoDuplicatePrefixDeclaration =
    <lint::nursery::no_duplicate_prefix_declaration::NoDuplicatePrefixDeclaration as biome_analyze::Rule>::Options;
pub type NoUndefinedPrefix =
    <lint::nursery::no_undefined_prefix::NoUndefinedPrefix as biome_analyze::Rule>::Options;
pub type NoUndefinedSubjectReference =
    <lint::nursery::no_undefined_subject_reference::NoUndefinedSubjectReference as biome_analyze::Rule>::Options;
pub type NoUnusedPrefix =
    <lint::nursery::no_unused_prefix::NoUnusedPrefix as biome_analyze::Rule>::Options;
pub type NoInvalidIri =
    <lint::nursery::no_invalid_iri::NoInvalidIri as biome_analyze::Rule>::Options;
pub type NoInvalidLanguageTag =
    <lint::nursery::no_invalid_language_tag::NoInvalidLanguageTag as biome_analyze::Rule>::Options;
pub type UseShorthandRdfType =
    <lint::nursery::use_shorthand_rdf_type::UseShorthandRdfType as biome_analyze::Rule>::Options;
pub type UseConsistentQuotes =
    <lint::nursery::use_consistent_quotes::UseConsistentQuotes as biome_analyze::Rule>::Options;
pub type UseConsistentDirectiveStyle =
    <lint::nursery::use_consistent_directive_style::UseConsistentDirectiveStyle as biome_analyze::Rule>::Options;
pub type NoLiteralTrimIssues =
    <lint::nursery::no_literal_trim_issues::NoLiteralTrimIssues as biome_analyze::Rule>::Options;
pub type NoDuplicateTriple =
    <lint::nursery::no_duplicate_triple::NoDuplicateTriple as biome_analyze::Rule>::Options;
