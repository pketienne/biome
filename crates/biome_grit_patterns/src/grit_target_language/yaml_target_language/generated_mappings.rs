//! Generated file, do not edit by hand, see `xtask/codegen`

//! Maps GritQL pattern names to Biome's internal syntax kinds.
use biome_rowan::AstNode;
use biome_yaml_syntax as lang;
use lang::YamlSyntaxKind;

/// Returns the syntax kind for a legacy or native node name.
pub fn kind_by_name(node_name: &str) -> Option<YamlSyntaxKind> {
    match node_name {
        // Native Biome AST patterns
        "YamlAliasNode" => lang::YamlAliasNode::KIND_SET.iter().next(),
        "YamlAnchorProperty" => lang::YamlAnchorProperty::KIND_SET.iter().next(),
        "YamlBlockContent" => lang::YamlBlockContent::KIND_SET.iter().next(),
        "YamlBlockKeepIndicator" => lang::YamlBlockKeepIndicator::KIND_SET.iter().next(),
        "YamlBlockMapExplicitEntry" => lang::YamlBlockMapExplicitEntry::KIND_SET.iter().next(),
        "YamlBlockMapImplicitEntry" => lang::YamlBlockMapImplicitEntry::KIND_SET.iter().next(),
        "YamlBlockMapping" => lang::YamlBlockMapping::KIND_SET.iter().next(),
        "YamlBlockSequence" => lang::YamlBlockSequence::KIND_SET.iter().next(),
        "YamlBlockSequenceEntry" => lang::YamlBlockSequenceEntry::KIND_SET.iter().next(),
        "YamlBlockStripIndicator" => lang::YamlBlockStripIndicator::KIND_SET.iter().next(),
        "YamlDirective" => lang::YamlDirective::KIND_SET.iter().next(),
        "YamlDocument" => lang::YamlDocument::KIND_SET.iter().next(),
        "YamlDoubleQuotedScalar" => lang::YamlDoubleQuotedScalar::KIND_SET.iter().next(),
        "YamlFlowInBlockNode" => lang::YamlFlowInBlockNode::KIND_SET.iter().next(),
        "YamlFlowJsonNode" => lang::YamlFlowJsonNode::KIND_SET.iter().next(),
        "YamlFlowMapExplicitEntry" => lang::YamlFlowMapExplicitEntry::KIND_SET.iter().next(),
        "YamlFlowMapImplicitEntry" => lang::YamlFlowMapImplicitEntry::KIND_SET.iter().next(),
        "YamlFlowMapping" => lang::YamlFlowMapping::KIND_SET.iter().next(),
        "YamlFlowSequence" => lang::YamlFlowSequence::KIND_SET.iter().next(),
        "YamlFlowYamlNode" => lang::YamlFlowYamlNode::KIND_SET.iter().next(),
        "YamlFoldedScalar" => lang::YamlFoldedScalar::KIND_SET.iter().next(),
        "YamlIndentationIndicator" => lang::YamlIndentationIndicator::KIND_SET.iter().next(),
        "YamlLiteralScalar" => lang::YamlLiteralScalar::KIND_SET.iter().next(),
        "YamlPlainScalar" => lang::YamlPlainScalar::KIND_SET.iter().next(),
        "YamlPropertiesAnchorFirst" => lang::YamlPropertiesAnchorFirst::KIND_SET.iter().next(),
        "YamlPropertiesTagFirst" => lang::YamlPropertiesTagFirst::KIND_SET.iter().next(),
        "YamlRoot" => lang::YamlRoot::KIND_SET.iter().next(),
        "YamlSingleQuotedScalar" => lang::YamlSingleQuotedScalar::KIND_SET.iter().next(),
        "YamlTagProperty" => lang::YamlTagProperty::KIND_SET.iter().next(),
        _ => None,
    }
}
