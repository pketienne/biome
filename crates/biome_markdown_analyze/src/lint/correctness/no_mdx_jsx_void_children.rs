use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdHtmlBlock;
use biome_rowan::{AstNode, TextRange, TextSize};

declare_lint_rule! {
    /// Disallow children for void HTML elements in MDX.
    ///
    /// Void HTML elements like `<br>`, `<hr>`, and `<img>` cannot have children.
    /// Using them with content is always an error.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <hr>content</hr>
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <hr />
    /// ```
    pub NoMdxJsxVoidChildren {
        version: "next",
        name: "noMdxJsxVoidChildren",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn is_void_element(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag.to_lowercase().as_str())
}

pub struct VoidWithChildren {
    range: TextRange,
    tag: String,
}

impl Rule for NoMdxJsxVoidChildren {
    type Query = Ast<MdHtmlBlock>;
    type State = VoidWithChildren;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        let text = node.syntax().text_trimmed().to_string();
        let base = node.syntax().text_trimmed_range().start();
        let mut signals = Vec::new();
        let mut byte_offset: usize = 0;

        for line in text.lines() {
            let trimmed = line.trim_start();

            if let Some(tag) = extract_opening_tag(trimmed) {
                if is_void_element(&tag) {
                    let closing = format!("</{}>", tag);
                    if line.contains(&closing) {
                        // Find the range of the opening tag (from `<` to `>`)
                        if let Some(lt_pos) = line.find('<') {
                            let tag_region = &line[lt_pos..];
                            let gt_pos = tag_region
                                .find('>')
                                .map(|p| lt_pos + p + 1)
                                .unwrap_or(lt_pos + tag.len() + 2);
                            signals.push(VoidWithChildren {
                                range: TextRange::new(
                                    base + TextSize::from((byte_offset + lt_pos) as u32),
                                    base + TextSize::from((byte_offset + gt_pos) as u32),
                                ),
                                tag,
                            });
                        }
                    }
                }
            }
            byte_offset += line.len() + 1;
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Void element \""{ &state.tag }"\" cannot have children."
                },
            )
            .note(markup! {
                "Use a self-closing tag instead."
            }),
        )
    }
}

/// Extract the tag name from a line starting with `<tagname...>`.
fn extract_opening_tag(trimmed: &str) -> Option<String> {
    if !trimmed.starts_with('<') || trimmed.starts_with("</") || trimmed.starts_with("<!") {
        return None;
    }
    let rest = &trimmed[1..];
    let end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')?;
    if end == 0 {
        return None;
    }
    Some(rest[..end].to_string())
}
