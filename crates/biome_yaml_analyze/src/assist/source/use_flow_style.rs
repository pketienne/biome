use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_source_rule,
};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt, Direction, TextRange};
use biome_yaml_syntax::{
    AnyYamlBlockMapEntry, YamlBlockMapping, YamlBlockSequence, YamlLanguage, YamlRoot,
    YamlSyntaxKind, YamlSyntaxToken,
};

declare_source_rule! {
    /// Convert a block mapping or sequence to flow style.
    ///
    /// Block collections with simple, single-level entries can be
    /// represented more compactly in flow style (`{a: 1, b: 2}` or `[a, b]`).
    ///
    /// Only flat (single-level) block collections are converted.
    ///
    /// ## Examples
    ///
    /// ```yaml,expect_diff
    /// items:
    ///   a: 1
    ///   b: 2
    /// ```
    pub UseFlowStyle {
        version: "next",
        name: "useFlowStyle",
        language: "yaml",
        recommended: false,
        fix_kind: FixKind::Safe,
    }
}

pub struct BlockToFlowState {
    range: TextRange,
    kind: BlockKind,
}

enum BlockKind {
    Mapping,
    Sequence,
}

impl Rule for UseFlowStyle {
    type Query = Ast<YamlRoot>;
    type State = BlockToFlowState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut states = Vec::new();

        for node in root.syntax().descendants() {
            match node.kind() {
                YamlSyntaxKind::YAML_BLOCK_MAPPING => {
                    // Skip if this mapping is nested inside another block mapping/sequence
                    // (i.e., it's a value in a parent collection, not a top-level collection)
                    if is_nested_block_collection(&node) {
                        continue;
                    }

                    // Only convert flat mappings (no nested block/flow nodes as values)
                    if let Some(mapping) = YamlBlockMapping::cast(node) {
                        let has_nested = mapping.entries().iter().any(|entry| {
                            let value = match &entry {
                                AnyYamlBlockMapEntry::YamlBlockMapImplicitEntry(e) => {
                                    e.value().map(|v| v.syntax().kind())
                                }
                                AnyYamlBlockMapEntry::YamlBlockMapExplicitEntry(e) => {
                                    e.value().map(|v| v.syntax().kind())
                                }
                                _ => None,
                            };
                            matches!(
                                value,
                                Some(
                                    YamlSyntaxKind::YAML_BLOCK_MAPPING
                                        | YamlSyntaxKind::YAML_BLOCK_SEQUENCE
                                        | YamlSyntaxKind::YAML_INDENTED_BLOCK
                                )
                            )
                        });
                        if !has_nested {
                            states.push(BlockToFlowState {
                                range: mapping.syntax().text_trimmed_range(),
                                kind: BlockKind::Mapping,
                            });
                        }
                    }
                }
                YamlSyntaxKind::YAML_BLOCK_SEQUENCE => {
                    // Skip if nested inside another block collection
                    if is_nested_block_collection(&node) {
                        continue;
                    }

                    if let Some(seq) = YamlBlockSequence::cast(node) {
                        // Only convert flat sequences (entries are simple scalars)
                        let has_nested = seq.syntax().descendants().skip(1).any(|d| {
                            matches!(
                                d.kind(),
                                YamlSyntaxKind::YAML_BLOCK_MAPPING
                                    | YamlSyntaxKind::YAML_BLOCK_SEQUENCE
                                    | YamlSyntaxKind::YAML_INDENTED_BLOCK
                            )
                        });
                        if !has_nested {
                            states.push(BlockToFlowState {
                                range: seq.syntax().text_trimmed_range(),
                                kind: BlockKind::Sequence,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        states.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let kind_name = match state.kind {
            BlockKind::Mapping => "mapping",
            BlockKind::Sequence => "sequence",
        };
        Some(
            RuleDiagnostic::new(
                category!("assist/source/useFlowStyle"),
                state.range,
                markup! {
                    "This block "{kind_name}" can be converted to flow style."
                },
            )
            .note(markup! {
                "Flow style is more compact for simple, single-level collections."
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

            let flow_text = match state.kind {
                BlockKind::Mapping => {
                    let mapping = YamlBlockMapping::cast(node.clone())?;
                    block_mapping_to_flow(&mapping)?
                }
                BlockKind::Sequence => {
                    let seq = YamlBlockSequence::cast(node.clone())?;
                    block_sequence_to_flow(&seq)?
                }
            };

            // Collect all tokens in the block node
            let tokens: Vec<_> = node.descendants_tokens(Direction::Next).collect();

            if tokens.is_empty() {
                return None;
            }

            // Replace the first token with the flow text
            let first_token = tokens[0].clone();
            let new_token =
                YamlSyntaxToken::new_detached(first_token.kind(), &flow_text, [], []);
            mutation.replace_token_transfer_trivia(first_token, new_token);

            // Remove all subsequent tokens
            for token in &tokens[1..] {
                let empty = YamlSyntaxToken::new_detached(token.kind(), "", [], []);
                mutation.replace_token_transfer_trivia(token.clone(), empty);
            }

            return Some(RuleAction::new(
                ctx.metadata().action_category(ctx.category(), ctx.group()),
                ctx.metadata().applicability(),
                markup! { "Convert to flow style." }.to_owned(),
                mutation,
            ));
        }

        None
    }
}

/// Convert a block mapping to flow text: `{k1: v1, k2: v2}`
fn block_mapping_to_flow(mapping: &YamlBlockMapping) -> Option<String> {
    let mut entries_text = Vec::new();

    for entry in mapping.entries().iter() {
        match entry {
            AnyYamlBlockMapEntry::YamlBlockMapImplicitEntry(e) => {
                let key_text = e
                    .key()
                    .map(|k| k.syntax().text_trimmed().to_string())
                    .unwrap_or_default();
                let value_text = e
                    .value()
                    .map(|v| v.syntax().text_trimmed().to_string())
                    .unwrap_or_default();
                entries_text.push(format!("{key_text}: {value_text}"));
            }
            _ => {
                // Skip explicit entries for now
                return None;
            }
        }
    }

    Some(format!("{{{}}}", entries_text.join(", ")))
}

/// Convert a block sequence to flow text: `[a, b, c]`
fn block_sequence_to_flow(seq: &YamlBlockSequence) -> Option<String> {
    let mut items = Vec::new();

    for entry in seq.syntax().children() {
        if entry.kind() == YamlSyntaxKind::YAML_BLOCK_SEQUENCE_ENTRY {
            // Extract the value after the dash
            for child in entry.children() {
                let text = child.text_trimmed().to_string();
                if !text.is_empty() {
                    items.push(text);
                }
            }
        }
    }

    Some(format!("[{}]", items.join(", ")))
}

/// Check if a block collection is deeply nested (2+ ancestor block collections).
/// A collection that is the direct value of a top-level key (depth 1) is fine
/// to convert. Only skip collections nested 2+ levels deep.
fn is_nested_block_collection(node: &biome_yaml_syntax::YamlSyntaxNode) -> bool {
    let mut depth = 0;
    let mut parent = node.parent();
    while let Some(p) = parent {
        if matches!(
            p.kind(),
            YamlSyntaxKind::YAML_BLOCK_MAPPING | YamlSyntaxKind::YAML_BLOCK_SEQUENCE
        ) {
            depth += 1;
            if depth >= 2 {
                return true;
            }
        }
        parent = p.parent();
    }
    false
}
