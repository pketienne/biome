// TODO: Once codegen is run (`cargo codegen formatter yaml`), this file should
// use the `map_syntax_node!` macro to dispatch formatting to per-node rules.
// For now, all nodes are formatted as verbatim to keep the crate compilable.

use crate::prelude::*;
use crate::verbatim::format_verbatim_node;
use biome_formatter::{FormatOwnedWithRule, FormatRefWithRule, FormatResult};
use biome_yaml_syntax::YamlSyntaxNode;

#[derive(Debug, Copy, Clone, Default)]
pub struct FormatYamlSyntaxNode;

impl FormatRule<YamlSyntaxNode> for FormatYamlSyntaxNode {
    type Context = YamlFormatContext;

    fn fmt(&self, node: &YamlSyntaxNode, f: &mut YamlFormatter) -> FormatResult<()> {
        format_verbatim_node(node).fmt(f)
    }
}

impl AsFormat<YamlFormatContext> for YamlSyntaxNode {
    type Format<'a> = FormatRefWithRule<'a, Self, FormatYamlSyntaxNode>;

    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, FormatYamlSyntaxNode)
    }
}

impl IntoFormat<YamlFormatContext> for YamlSyntaxNode {
    type Format = FormatOwnedWithRule<Self, FormatYamlSyntaxNode>;

    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, FormatYamlSyntaxNode)
    }
}
