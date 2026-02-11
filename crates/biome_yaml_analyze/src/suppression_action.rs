use biome_analyze::{ApplySuppression, SuppressionAction};
use biome_rowan::{BatchMutation, SyntaxToken};
use biome_yaml_syntax::YamlLanguage;

pub(crate) struct YamlSuppressionAction;

impl SuppressionAction for YamlSuppressionAction {
    type Language = YamlLanguage;

    fn find_token_for_inline_suppression(
        &self,
        _original_token: SyntaxToken<Self::Language>,
    ) -> Option<ApplySuppression<Self::Language>> {
        None
    }

    fn apply_inline_suppression(
        &self,
        _mutation: &mut BatchMutation<Self::Language>,
        _apply_suppression: ApplySuppression<Self::Language>,
        _suppression_text: &str,
        _suppression_reason: &str,
    ) {
        unreachable!("find_token_for_inline_suppression returns None")
    }

    fn suppression_top_level_comment(&self, suppression_text: &str) -> String {
        format!("# {suppression_text}: <explanation> ")
    }
}
