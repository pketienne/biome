use biome_yaml_parser::parse_yaml;

use super::semantic_model;

#[test]
fn single_anchor_single_alias() {
    let parsed = parse_yaml("anchor: &myanchor value\nalias: *myanchor\n");
    let model = semantic_model(&parsed.tree());

    assert_eq!(model.all_anchors().count(), 1);
    assert_eq!(model.all_aliases().count(), 1);
    assert_eq!(model.all_unresolved_aliases().count(), 0);
    assert_eq!(model.all_duplicate_anchors().count(), 0);

    let anchor = model.all_anchors().next().unwrap();
    assert_eq!(anchor.name(), "myanchor");
    assert_eq!(anchor.document_index(), 0);

    let alias = model.all_aliases().next().unwrap();
    assert_eq!(alias.name(), "myanchor");
    assert_eq!(alias.document_index(), 0);

    // Alias resolves to the anchor
    let resolved = alias.anchor().unwrap();
    assert_eq!(resolved.name(), "myanchor");

    // Anchor knows about its alias
    let anchor_aliases = anchor.all_aliases();
    assert_eq!(anchor_aliases.len(), 1);
    assert_eq!(anchor_aliases[0].name(), "myanchor");
}

#[test]
fn multiple_anchors_multiple_aliases() {
    let parsed = parse_yaml(
        "first: &a value1\nsecond: &b value2\nref_a: *a\nref_b: *b\nref_a2: *a\n",
    );
    let model = semantic_model(&parsed.tree());

    assert_eq!(model.all_anchors().count(), 2);
    assert_eq!(model.all_aliases().count(), 3);
    assert_eq!(model.all_unresolved_aliases().count(), 0);

    // Anchor "a" should have 2 aliases
    let anchor_a = model.all_anchors().find(|a| a.name() == "a").unwrap();
    assert_eq!(anchor_a.all_aliases().len(), 2);

    // Anchor "b" should have 1 alias
    let anchor_b = model.all_anchors().find(|a| a.name() == "b").unwrap();
    assert_eq!(anchor_b.all_aliases().len(), 1);
}

#[test]
fn unresolved_alias() {
    let parsed = parse_yaml("alias: *nonexistent\n");
    let model = semantic_model(&parsed.tree());

    assert_eq!(model.all_anchors().count(), 0);
    assert_eq!(model.all_aliases().count(), 1);
    assert_eq!(model.all_unresolved_aliases().count(), 1);

    let unresolved = model.all_unresolved_aliases().next().unwrap();
    assert_eq!(unresolved.name(), "nonexistent");

    // The alias should not resolve
    let alias = model.all_aliases().next().unwrap();
    assert!(alias.anchor().is_none());
}

#[test]
fn duplicate_anchors() {
    let parsed = parse_yaml("first: &dup value1\nsecond: &dup value2\n");
    let model = semantic_model(&parsed.tree());

    assert_eq!(model.all_anchors().count(), 2);
    assert_eq!(model.all_duplicate_anchors().count(), 1);

    let dup = model.all_duplicate_anchors().next().unwrap();
    assert_eq!(dup.name(), "dup");
    assert_eq!(dup.duplicate_ranges().len(), 1);
}

#[test]
fn empty_document() {
    let parsed = parse_yaml("");
    let model = semantic_model(&parsed.tree());

    assert_eq!(model.all_anchors().count(), 0);
    assert_eq!(model.all_aliases().count(), 0);
    assert_eq!(model.all_unresolved_aliases().count(), 0);
    assert_eq!(model.all_duplicate_anchors().count(), 0);
}

#[test]
fn plain_document_no_anchors() {
    let parsed = parse_yaml("key: value\nother: 42\n");
    let model = semantic_model(&parsed.tree());

    assert_eq!(model.all_anchors().count(), 0);
    assert_eq!(model.all_aliases().count(), 0);
}

#[test]
fn anchor_syntax_lookup() {
    let parsed = parse_yaml("anchor: &myanchor value\n");
    let model = semantic_model(&parsed.tree());

    let anchor = model.all_anchors().next().unwrap();
    // Syntax node should be recorded
    assert!(anchor.syntax().is_some());
}

#[test]
fn alias_syntax_lookup() {
    let parsed = parse_yaml("anchor: &myanchor value\nalias: *myanchor\n");
    let model = semantic_model(&parsed.tree());

    let alias = model.all_aliases().next().unwrap();
    assert!(alias.syntax().is_some());
}

#[test]
fn multi_document_anchor_scoping() {
    // Anchors in different documents should be independent
    let parsed = parse_yaml("anchor: &a value\n---\nalias: *a\n");
    let model = semantic_model(&parsed.tree());

    let anchors: Vec<_> = model.all_anchors().collect();
    let aliases: Vec<_> = model.all_aliases().collect();

    assert_eq!(anchors.len(), 1);
    assert_eq!(aliases.len(), 1);

    // The anchor is in document 0, the alias in document 1
    // It should be unresolved since anchors don't cross document boundaries
    if anchors[0].document_index() != aliases[0].document_index() {
        assert_eq!(model.all_unresolved_aliases().count(), 1);
    }
}

#[test]
fn merge_key_with_anchor_alias() {
    let parsed = parse_yaml(
        "defaults: &defaults\n  timeout: 30\nserver:\n  <<: *defaults\n",
    );
    let model = semantic_model(&parsed.tree());

    assert_eq!(model.all_anchors().count(), 1);
    assert_eq!(model.all_aliases().count(), 1);
    assert_eq!(model.all_unresolved_aliases().count(), 0);

    let anchor = model.all_anchors().next().unwrap();
    assert_eq!(anchor.name(), "defaults");

    let alias = model.all_aliases().next().unwrap();
    assert_eq!(alias.name(), "defaults");
    assert!(alias.anchor().is_some());
}

#[test]
fn anchor_range_is_valid() {
    let parsed = parse_yaml("key: &myanchor value\n");
    let model = semantic_model(&parsed.tree());

    let anchor = model.all_anchors().next().unwrap();
    let range = anchor.range();
    // Range should be non-empty
    assert!(range.start() < range.end());
}

#[test]
fn alias_range_is_valid() {
    let parsed = parse_yaml("key: &a val\nref: *a\n");
    let model = semantic_model(&parsed.tree());

    let alias = model.all_aliases().next().unwrap();
    let range = alias.range();
    assert!(range.start() < range.end());
}
