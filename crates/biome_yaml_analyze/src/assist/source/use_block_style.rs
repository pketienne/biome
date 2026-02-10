use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_source_rule,
};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, BatchMutationExt, Direction, TextRange};
use biome_yaml_syntax::{YamlLanguage, YamlRoot, YamlSyntaxKind, YamlSyntaxToken};

declare_source_rule! {
    /// Convert a flow mapping or sequence to block style.
    ///
    /// Flow collections (`{a: 1, b: 2}`, `[a, b]`) can be converted to
    /// block-style equivalents for improved readability, especially when
    /// the collection has many entries.
    ///
    /// Only flat (single-level) flow collections are converted.
    ///
    /// ## Examples
    ///
    /// ```yaml,expect_diff
    /// items: {a: 1, b: 2}
    /// ```
    pub UseBlockStyle {
        version: "next",
        name: "useBlockStyle",
        language: "yaml",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

pub struct FlowToBlockState {
    range: TextRange,
    kind: FlowKind,
}

enum FlowKind {
    Mapping,
    Sequence,
}

impl Rule for UseBlockStyle {
    type Query = Ast<YamlRoot>;
    type State = FlowToBlockState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut states = Vec::new();

        for node in root.syntax().descendants() {
            match node.kind() {
                YamlSyntaxKind::YAML_FLOW_MAPPING => {
                    // Only convert flat mappings (no nested flow/block nodes)
                    let has_nested = node.descendants().skip(1).any(|d| {
                        matches!(
                            d.kind(),
                            YamlSyntaxKind::YAML_FLOW_MAPPING
                                | YamlSyntaxKind::YAML_FLOW_SEQUENCE
                        )
                    });
                    if !has_nested {
                        states.push(FlowToBlockState {
                            range: node.text_trimmed_range(),
                            kind: FlowKind::Mapping,
                        });
                    }
                }
                YamlSyntaxKind::YAML_FLOW_SEQUENCE => {
                    let has_nested = node.descendants().skip(1).any(|d| {
                        matches!(
                            d.kind(),
                            YamlSyntaxKind::YAML_FLOW_MAPPING
                                | YamlSyntaxKind::YAML_FLOW_SEQUENCE
                        )
                    });
                    if !has_nested {
                        states.push(FlowToBlockState {
                            range: node.text_trimmed_range(),
                            kind: FlowKind::Sequence,
                        });
                    }
                }
                _ => {}
            }
        }

        states.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let kind_name = match state.kind {
            FlowKind::Mapping => "mapping",
            FlowKind::Sequence => "sequence",
        };
        Some(
            RuleDiagnostic::new(
                category!("assist/source/useBlockStyle"),
                state.range,
                markup! {
                    "This flow "{kind_name}" can be converted to block style."
                },
            )
            .note(markup! {
                "Block style is often more readable for multi-entry collections."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        for node in root.syntax().descendants() {
            if node.text_trimmed_range() != state.range {
                continue;
            }

            let text = node.text_trimmed().to_string();

            let block_text = match state.kind {
                FlowKind::Mapping => flow_mapping_to_block(&text)?,
                FlowKind::Sequence => flow_sequence_to_block(&text)?,
            };

            // Collect all tokens in the flow node
            let tokens: Vec<_> = node.descendants_tokens(Direction::Next).collect();

            if tokens.is_empty() {
                return None;
            }

            // Replace the first token with the block text
            let first_token = tokens[0].clone();
            let new_token =
                YamlSyntaxToken::new_detached(first_token.kind(), &block_text, [], []);
            mutation.replace_token_transfer_trivia(first_token, new_token);

            // Remove all subsequent tokens by replacing with empty tokens
            for token in &tokens[1..] {
                let empty = YamlSyntaxToken::new_detached(token.kind(), "", [], []);
                mutation.replace_token_transfer_trivia(token.clone(), empty);
            }

            return Some(RuleAction::new(
                ctx.metadata().action_category(ctx.category(), ctx.group()),
                ctx.metadata().applicability(),
                markup! { "Convert to block style." }.to_owned(),
                mutation,
            ));
        }

        None
    }
}

/// Convert `{k1: v1, k2: v2}` to block mapping text.
fn flow_mapping_to_block(text: &str) -> Option<String> {
    let inner = text.strip_prefix('{')?.strip_suffix('}')?;
    let inner = inner.trim();
    if inner.is_empty() {
        return Some("{}".to_string());
    }

    let entries: Vec<&str> = split_flow_entries(inner);
    let mut lines = Vec::new();
    for entry in &entries {
        let entry = entry.trim();
        if !entry.is_empty() {
            lines.push(format!("\n  {entry}"));
        }
    }
    Some(lines.join(""))
}

/// Convert `[a, b, c]` to block sequence text.
fn flow_sequence_to_block(text: &str) -> Option<String> {
    let inner = text.strip_prefix('[')?.strip_suffix(']')?;
    let inner = inner.trim();
    if inner.is_empty() {
        return Some("[]".to_string());
    }

    let entries: Vec<&str> = split_flow_entries(inner);
    let mut lines = Vec::new();
    for entry in &entries {
        let entry = entry.trim();
        if !entry.is_empty() {
            lines.push(format!("\n  - {entry}"));
        }
    }
    Some(lines.join(""))
}

/// Split flow collection entries by commas, respecting nested brackets/braces/quotes.
fn split_flow_entries(s: &str) -> Vec<&str> {
    let mut entries = Vec::new();
    let mut depth = 0;
    let mut in_single = false;
    let mut in_double = false;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '{' | '[' if !in_single && !in_double => depth += 1,
            '}' | ']' if !in_single && !in_double => depth -= 1,
            ',' if depth == 0 && !in_single && !in_double => {
                entries.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        entries.push(&s[start..]);
    }
    entries
}
