use biome_markdown_syntax::{MdDocument, MarkdownSyntaxToken};
use biome_rowan::{AstNode, BatchMutation, BatchMutationExt, TextRange};

/// Replace text at a given range with the corrected text, preserving surrounding trivia.
///
/// This is the standard pattern for markdown lint rule fixes. It finds the tokens
/// that overlap the target range, computes prefix/suffix from the first/last tokens
/// to preserve trivia (whitespace, newlines), and replaces with the corrected text.
///
/// Returns the mutation which the caller can wrap in a `RuleAction`.
pub fn make_text_replacement(
    root: &MdDocument,
    range: TextRange,
    corrected: &str,
) -> Option<BatchMutation<biome_markdown_syntax::MarkdownLanguage>> {
    let mut token = root
        .syntax()
        .token_at_offset(range.start())
        .right_biased()?;
    let mut tokens = vec![token.clone()];
    while token.text_range().end() < range.end() {
        token = token.next_token()?;
        tokens.push(token.clone());
    }
    let first = &tokens[0];
    let last = tokens.last()?;
    let prefix_len = u32::from(range.start() - first.text_range().start()) as usize;
    let suffix_start = u32::from(range.end() - last.text_range().start()) as usize;
    let prefix = &first.text()[..prefix_len];
    let suffix = &last.text()[suffix_start..];
    let new_text = format!("{}{}{}", prefix, corrected, suffix);
    let new_token = MarkdownSyntaxToken::new_detached(first.kind(), &new_text, [], []);
    let mut mutation = root.clone().begin();
    mutation.replace_element_discard_trivia(first.clone().into(), new_token.into());
    for t in &tokens[1..] {
        let empty = MarkdownSyntaxToken::new_detached(t.kind(), "", [], []);
        mutation.replace_element_discard_trivia(t.clone().into(), empty.into());
    }
    Some(mutation)
}
