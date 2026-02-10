use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstNodeList, TextRange, TextSize};
use biome_yaml_syntax::{
    YamlBlockMapImplicitEntry, YamlBlockSequence, YamlRoot, YamlSyntaxKind,
};

declare_lint_rule! {
    /// Enforce consistent indentation style for block sequences inside mappings.
    ///
    /// In YAML, block sequences nested inside mappings can be indented (the `-`
    /// indicator is deeper than the parent key) or non-indented (the `-` indicator
    /// is at the same column as the parent key). Both are valid YAML, but mixing
    /// styles within a file is inconsistent.
    ///
    /// This rule detects the first style used and requires all subsequent
    /// sequences-in-mappings to use the same style.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// parent:
    ///   - item1
    /// other:
    /// - item2
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// parent:
    ///   - item1
    /// other:
    ///   - item2
    /// ```
    pub UseConsistentSequenceIndentation {
        version: "next",
        name: "useConsistentSequenceIndentation",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeqIndentStyle {
    Indented,
    NonIndented,
}

impl std::fmt::Display for SeqIndentStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeqIndentStyle::Indented => write!(f, "indented"),
            SeqIndentStyle::NonIndented => write!(f, "non-indented"),
        }
    }
}

pub struct InconsistentSequenceIndent {
    range: TextRange,
    found: SeqIndentStyle,
    expected: SeqIndentStyle,
}

/// Compute the column (0-indexed) of a given offset within the source text.
fn column_of(text: &str, offset: TextSize) -> usize {
    let offset: usize = offset.into();
    let before = &text[..offset];
    match before.rfind('\n') {
        Some(nl) => offset - nl - 1,
        None => offset,
    }
}

impl Rule for UseConsistentSequenceIndentation {
    type Query = Ast<YamlRoot>;
    type State = InconsistentSequenceIndent;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let source = root.syntax().to_string();
        let mut violations = Vec::new();
        let mut expected_style: Option<SeqIndentStyle> = None;

        for node in root.syntax().descendants() {
            if !YamlBlockSequence::can_cast(node.kind()) {
                continue;
            }

            // Check if this sequence is a value of a mapping entry
            let Some(parent) = node.parent() else {
                continue;
            };
            // The sequence may be wrapped in AnyYamlBlockNode/AnyYamlBlockInBlockNode
            // union nodes. Walk up until we find a mapping entry or run out.
            let mapping_entry = parent.ancestors().find(|ancestor| {
                ancestor.kind() == YamlSyntaxKind::YAML_BLOCK_MAP_IMPLICIT_ENTRY
                    || ancestor.kind() == YamlSyntaxKind::YAML_BLOCK_MAP_EXPLICIT_ENTRY
            });
            let Some(entry_node) = mapping_entry else {
                continue;
            };

            // Get the column of the mapping key
            let key_col = if let Some(entry) = YamlBlockMapImplicitEntry::cast(entry_node.clone()) {
                let Some(key) = entry.key() else { continue };
                column_of(&source, key.syntax().text_trimmed_range().start())
            } else {
                // Explicit entry — use the `?` token position
                let first_token = entry_node.first_token();
                let Some(token) = first_token else { continue };
                column_of(&source, token.text_trimmed_range().start())
            };

            // Get the first `-` token in this sequence
            let Some(seq) = YamlBlockSequence::cast(node.clone()) else {
                continue;
            };
            let Some(first_entry) = seq
                .entries()
                .iter()
                .next()
                .and_then(|e| e.as_yaml_block_sequence_entry().cloned())
            else {
                continue;
            };
            let Ok(minus_token) = first_entry.minus_token() else {
                continue;
            };
            let dash_col = column_of(&source, minus_token.text_trimmed_range().start());

            let style = if dash_col > key_col {
                SeqIndentStyle::Indented
            } else {
                SeqIndentStyle::NonIndented
            };

            match expected_style {
                None => {
                    expected_style = Some(style);
                }
                Some(expected) if style != expected => {
                    violations.push(InconsistentSequenceIndent {
                        range: minus_token.text_trimmed_range(),
                        found: style,
                        expected,
                    });
                }
                _ => {}
            }
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Inconsistent sequence indentation: found "{state.found.to_string()}" style, but "{state.expected.to_string()}" style was used earlier."
                },
            )
            .note(markup! {
                "Use consistent sequence indentation throughout the file."
            }),
        )
    }
}
