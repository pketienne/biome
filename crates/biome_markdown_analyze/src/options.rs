use crate::lint;
pub type NoMissingLanguage =
    <lint::nursery::no_missing_language::NoMissingLanguage as biome_analyze::Rule>::Options;
pub type UseHeadingIncrement =
    <lint::nursery::use_heading_increment::UseHeadingIncrement as biome_analyze::Rule>::Options;
pub type NoDuplicateHeadings =
    <lint::nursery::no_duplicate_headings::NoDuplicateHeadings as biome_analyze::Rule>::Options;
pub type NoEmptyLinks =
    <lint::nursery::no_empty_links::NoEmptyLinks as biome_analyze::Rule>::Options;
pub type NoReversedLinks =
    <lint::nursery::no_reversed_links::NoReversedLinks as biome_analyze::Rule>::Options;
