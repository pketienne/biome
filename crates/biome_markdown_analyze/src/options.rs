use crate::lint;
pub type NoMissingLanguage =
    <lint::style::no_missing_language::NoMissingLanguage as biome_analyze::Rule>::Options;
pub type UseHeadingIncrement =
    <lint::correctness::use_heading_increment::UseHeadingIncrement as biome_analyze::Rule>::Options;
pub type NoDuplicateHeadings =
    <lint::suspicious::no_duplicate_headings::NoDuplicateHeadings as biome_analyze::Rule>::Options;
pub type NoEmptyLinks =
    <lint::correctness::no_empty_links::NoEmptyLinks as biome_analyze::Rule>::Options;
pub type NoReversedLinks =
    <lint::correctness::no_reversed_links::NoReversedLinks as biome_analyze::Rule>::Options;
