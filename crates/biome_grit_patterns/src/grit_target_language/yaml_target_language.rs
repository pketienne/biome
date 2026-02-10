mod constants;
pub mod generated_mappings;

use super::GritTargetLanguageImpl;
use crate::grit_target_node::GritTargetSyntaxKind;
use biome_rowan::{RawSyntaxKind, SyntaxKindSet};
use biome_yaml_syntax::{YamlLanguage, YamlSyntaxKind};
use generated_mappings::kind_by_name;

const COMMENT_KINDS: SyntaxKindSet<YamlLanguage> =
    SyntaxKindSet::from_raw(RawSyntaxKind(YamlSyntaxKind::COMMENT as u16));

#[derive(Clone, Debug)]
pub struct YamlTargetLanguage;

impl GritTargetLanguageImpl for YamlTargetLanguage {
    type Kind = YamlSyntaxKind;

    fn kind_by_name(&self, node_name: &str) -> Option<YamlSyntaxKind> {
        kind_by_name(node_name)
    }

    fn name_for_kind(&self, _kind: GritTargetSyntaxKind) -> &'static str {
        "(unknown node)"
    }

    fn named_slots_for_kind(&self, _kind: GritTargetSyntaxKind) -> &'static [(&'static str, u32)] {
        &[]
    }

    fn snippet_context_strings(&self) -> &[(&'static str, &'static str)] {
        &[
            ("", ""),
            ("GRIT_KEY: ", ""),
            ("GRIT_KEY:\n  ", ""),
        ]
    }

    fn is_comment_kind(kind: GritTargetSyntaxKind) -> bool {
        kind.as_yaml_kind()
            .is_some_and(|kind| COMMENT_KINDS.matches(kind))
    }

    fn metavariable_kind() -> Self::Kind {
        // YAML doesn't have metavariable support yet.
        // Use YAML_BOGUS as a placeholder — it will never match any real node.
        YamlSyntaxKind::YAML_BOGUS
    }
}
