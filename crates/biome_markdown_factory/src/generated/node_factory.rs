//! Generated file, do not edit by hand, see `xtask/codegen`

#![allow(clippy::redundant_closure)]
use biome_markdown_syntax::{
    MarkdownSyntaxElement as SyntaxElement, MarkdownSyntaxNode as SyntaxNode,
    MarkdownSyntaxToken as SyntaxToken, *,
};
use biome_rowan::AstNode;
pub fn md_bullet(bullet_token: SyntaxToken, content: MdBlockList) -> MdBulletBuilder {
    MdBulletBuilder {
        bullet_token,
        content,
        checkbox: None,
    }
}
pub struct MdBulletBuilder {
    bullet_token: SyntaxToken,
    content: MdBlockList,
    checkbox: Option<MdCheckbox>,
}
impl MdBulletBuilder {
    pub fn with_checkbox(mut self, checkbox: MdCheckbox) -> Self {
        self.checkbox = Some(checkbox);
        self
    }
    pub fn build(self) -> MdBullet {
        MdBullet::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_BULLET,
            [
                Some(SyntaxElement::Token(self.bullet_token)),
                self.checkbox
                    .map(|token| SyntaxElement::Node(token.into_syntax())),
                Some(SyntaxElement::Node(self.content.into_syntax())),
            ],
        ))
    }
}
pub fn md_bullet_list_item(md_bullet_list: MdBulletList) -> MdBulletListItem {
    MdBulletListItem::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_BULLET_LIST_ITEM,
        [Some(SyntaxElement::Node(md_bullet_list.into_syntax()))],
    ))
}
pub fn md_checkbox(l_brack_token: SyntaxToken, r_brack_token: SyntaxToken) -> MdCheckboxBuilder {
    MdCheckboxBuilder {
        l_brack_token,
        r_brack_token,
        value_token: None,
    }
}
pub struct MdCheckboxBuilder {
    l_brack_token: SyntaxToken,
    r_brack_token: SyntaxToken,
    value_token: Option<SyntaxToken>,
}
impl MdCheckboxBuilder {
    pub fn with_value_token(mut self, value_token: SyntaxToken) -> Self {
        self.value_token = Some(value_token);
        self
    }
    pub fn build(self) -> MdCheckbox {
        MdCheckbox::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_CHECKBOX,
            [
                Some(SyntaxElement::Token(self.l_brack_token)),
                self.value_token.map(|token| SyntaxElement::Token(token)),
                Some(SyntaxElement::Token(self.r_brack_token)),
            ],
        ))
    }
}
pub fn md_directive(
    marker_token: SyntaxToken,
    name: MdInlineItemList,
    attributes: MdDirectiveAttributeList,
) -> MdDirectiveBuilder {
    MdDirectiveBuilder {
        marker_token,
        name,
        attributes,
        l_curly_token: None,
        r_curly_token: None,
    }
}
pub struct MdDirectiveBuilder {
    marker_token: SyntaxToken,
    name: MdInlineItemList,
    attributes: MdDirectiveAttributeList,
    l_curly_token: Option<SyntaxToken>,
    r_curly_token: Option<SyntaxToken>,
}
impl MdDirectiveBuilder {
    pub fn with_l_curly_token(mut self, l_curly_token: SyntaxToken) -> Self {
        self.l_curly_token = Some(l_curly_token);
        self
    }
    pub fn with_r_curly_token(mut self, r_curly_token: SyntaxToken) -> Self {
        self.r_curly_token = Some(r_curly_token);
        self
    }
    pub fn build(self) -> MdDirective {
        MdDirective::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_DIRECTIVE,
            [
                Some(SyntaxElement::Token(self.marker_token)),
                Some(SyntaxElement::Node(self.name.into_syntax())),
                self.l_curly_token.map(|token| SyntaxElement::Token(token)),
                Some(SyntaxElement::Node(self.attributes.into_syntax())),
                self.r_curly_token.map(|token| SyntaxElement::Token(token)),
            ],
        ))
    }
}
pub fn md_directive_attribute(name: MdInlineItemList) -> MdDirectiveAttributeBuilder {
    MdDirectiveAttributeBuilder {
        name,
        eq_token: None,
        value: None,
    }
}
pub struct MdDirectiveAttributeBuilder {
    name: MdInlineItemList,
    eq_token: Option<SyntaxToken>,
    value: Option<MdDirectiveAttributeValue>,
}
impl MdDirectiveAttributeBuilder {
    pub fn with_eq_token(mut self, eq_token: SyntaxToken) -> Self {
        self.eq_token = Some(eq_token);
        self
    }
    pub fn with_value(mut self, value: MdDirectiveAttributeValue) -> Self {
        self.value = Some(value);
        self
    }
    pub fn build(self) -> MdDirectiveAttribute {
        MdDirectiveAttribute::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_DIRECTIVE_ATTRIBUTE,
            [
                Some(SyntaxElement::Node(self.name.into_syntax())),
                self.eq_token.map(|token| SyntaxElement::Token(token)),
                self.value
                    .map(|token| SyntaxElement::Node(token.into_syntax())),
            ],
        ))
    }
}
pub fn md_directive_attribute_value(
    delimiter_token: SyntaxToken,
    content: MdInlineItemList,
    closing_delimiter_token: SyntaxToken,
) -> MdDirectiveAttributeValue {
    MdDirectiveAttributeValue::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_DIRECTIVE_ATTRIBUTE_VALUE,
        [
            Some(SyntaxElement::Token(delimiter_token)),
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Token(closing_delimiter_token)),
        ],
    ))
}
pub fn md_document(value: MdBlockList, eof_token: SyntaxToken) -> MdDocumentBuilder {
    MdDocumentBuilder {
        value,
        eof_token,
        bom_token: None,
    }
}
pub struct MdDocumentBuilder {
    value: MdBlockList,
    eof_token: SyntaxToken,
    bom_token: Option<SyntaxToken>,
}
impl MdDocumentBuilder {
    pub fn with_bom_token(mut self, bom_token: SyntaxToken) -> Self {
        self.bom_token = Some(bom_token);
        self
    }
    pub fn build(self) -> MdDocument {
        MdDocument::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_DOCUMENT,
            [
                self.bom_token.map(|token| SyntaxElement::Token(token)),
                Some(SyntaxElement::Node(self.value.into_syntax())),
                Some(SyntaxElement::Token(self.eof_token)),
            ],
        ))
    }
}
pub fn md_fenced_code_block(
    l_fence_token: SyntaxToken,
    code_list: MdCodeNameList,
    content: MdInlineItemList,
    r_fence_token: SyntaxToken,
) -> MdFencedCodeBlock {
    MdFencedCodeBlock::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_FENCED_CODE_BLOCK,
        [
            Some(SyntaxElement::Token(l_fence_token)),
            Some(SyntaxElement::Node(code_list.into_syntax())),
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Token(r_fence_token)),
        ],
    ))
}
pub fn md_hard_line(value_token: SyntaxToken) -> MdHardLine {
    MdHardLine::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_HARD_LINE,
        [Some(SyntaxElement::Token(value_token))],
    ))
}
pub fn md_hash(hash_token: SyntaxToken) -> MdHash {
    MdHash::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_HASH,
        [Some(SyntaxElement::Token(hash_token))],
    ))
}
pub fn md_header(before: MdHashList, after: MdHashList) -> MdHeaderBuilder {
    MdHeaderBuilder {
        before,
        after,
        content: None,
    }
}
pub struct MdHeaderBuilder {
    before: MdHashList,
    after: MdHashList,
    content: Option<MdParagraph>,
}
impl MdHeaderBuilder {
    pub fn with_content(mut self, content: MdParagraph) -> Self {
        self.content = Some(content);
        self
    }
    pub fn build(self) -> MdHeader {
        MdHeader::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_HEADER,
            [
                Some(SyntaxElement::Node(self.before.into_syntax())),
                self.content
                    .map(|token| SyntaxElement::Node(token.into_syntax())),
                Some(SyntaxElement::Node(self.after.into_syntax())),
            ],
        ))
    }
}
pub fn md_html_block(content: MdInlineItemList) -> MdHtmlBlock {
    MdHtmlBlock::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_HTML_BLOCK,
        [Some(SyntaxElement::Node(content.into_syntax()))],
    ))
}
pub fn md_indent(value_token: SyntaxToken) -> MdIndent {
    MdIndent::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INDENT,
        [Some(SyntaxElement::Token(value_token))],
    ))
}
pub fn md_indent_code_block(content: MdInlineItemList) -> MdIndentCodeBlock {
    MdIndentCodeBlock::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INDENT_CODE_BLOCK,
        [Some(SyntaxElement::Node(content.into_syntax()))],
    ))
}
pub fn md_indented_code_line(indentation: MdIndent, content: MdTextual) -> MdIndentedCodeLine {
    MdIndentedCodeLine::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INDENTED_CODE_LINE,
        [
            Some(SyntaxElement::Node(indentation.into_syntax())),
            Some(SyntaxElement::Node(content.into_syntax())),
        ],
    ))
}
pub fn md_inline_code(
    l_tick_token: SyntaxToken,
    content: MdInlineItemList,
    r_tick_token: SyntaxToken,
) -> MdInlineCode {
    MdInlineCode::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INLINE_CODE,
        [
            Some(SyntaxElement::Token(l_tick_token)),
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Token(r_tick_token)),
        ],
    ))
}
pub fn md_inline_emphasis(
    l_fence_token: SyntaxToken,
    content: MdInlineItemList,
    r_fence_token: SyntaxToken,
) -> MdInlineEmphasis {
    MdInlineEmphasis::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INLINE_EMPHASIS,
        [
            Some(SyntaxElement::Token(l_fence_token)),
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Token(r_fence_token)),
        ],
    ))
}
pub fn md_inline_image(
    excl_token: SyntaxToken,
    alt: MdInlineImageAlt,
    source: MdInlineImageSource,
) -> MdInlineImageBuilder {
    MdInlineImageBuilder {
        excl_token,
        alt,
        source,
        link: None,
    }
}
pub struct MdInlineImageBuilder {
    excl_token: SyntaxToken,
    alt: MdInlineImageAlt,
    source: MdInlineImageSource,
    link: Option<MdInlineImageLink>,
}
impl MdInlineImageBuilder {
    pub fn with_link(mut self, link: MdInlineImageLink) -> Self {
        self.link = Some(link);
        self
    }
    pub fn build(self) -> MdInlineImage {
        MdInlineImage::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_INLINE_IMAGE,
            [
                Some(SyntaxElement::Token(self.excl_token)),
                Some(SyntaxElement::Node(self.alt.into_syntax())),
                Some(SyntaxElement::Node(self.source.into_syntax())),
                self.link
                    .map(|token| SyntaxElement::Node(token.into_syntax())),
            ],
        ))
    }
}
pub fn md_inline_image_alt(
    l_brack_token: SyntaxToken,
    content: MdInlineItemList,
    r_brack_token: SyntaxToken,
) -> MdInlineImageAlt {
    MdInlineImageAlt::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INLINE_IMAGE_ALT,
        [
            Some(SyntaxElement::Token(l_brack_token)),
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Token(r_brack_token)),
        ],
    ))
}
pub fn md_inline_image_link(
    l_paren_token: SyntaxToken,
    content: MdInlineItemList,
    r_paren_token: SyntaxToken,
) -> MdInlineImageLink {
    MdInlineImageLink::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INLINE_IMAGE_LINK,
        [
            Some(SyntaxElement::Token(l_paren_token)),
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Token(r_paren_token)),
        ],
    ))
}
pub fn md_inline_image_source(
    l_paren_token: SyntaxToken,
    content: MdInlineItemList,
    r_paren_token: SyntaxToken,
) -> MdInlineImageSource {
    MdInlineImageSource::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INLINE_IMAGE_SOURCE,
        [
            Some(SyntaxElement::Token(l_paren_token)),
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Token(r_paren_token)),
        ],
    ))
}
pub fn md_inline_italic(
    l_fence_token: SyntaxToken,
    content: MdInlineItemList,
    r_fence_token: SyntaxToken,
) -> MdInlineItalic {
    MdInlineItalic::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INLINE_ITALIC,
        [
            Some(SyntaxElement::Token(l_fence_token)),
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Token(r_fence_token)),
        ],
    ))
}
pub fn md_inline_link(
    l_brack_token: SyntaxToken,
    text: MdInlineItemList,
    r_brack_token: SyntaxToken,
    l_paren_token: SyntaxToken,
    source: MdInlineItemList,
    r_paren_token: SyntaxToken,
) -> MdInlineLink {
    MdInlineLink::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INLINE_LINK,
        [
            Some(SyntaxElement::Token(l_brack_token)),
            Some(SyntaxElement::Node(text.into_syntax())),
            Some(SyntaxElement::Token(r_brack_token)),
            Some(SyntaxElement::Token(l_paren_token)),
            Some(SyntaxElement::Node(source.into_syntax())),
            Some(SyntaxElement::Token(r_paren_token)),
        ],
    ))
}
pub fn md_inline_strikethrough(
    l_fence_token: SyntaxToken,
    content: MdInlineItemList,
    r_fence_token: SyntaxToken,
) -> MdInlineStrikethrough {
    MdInlineStrikethrough::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INLINE_STRIKETHROUGH,
        [
            Some(SyntaxElement::Token(l_fence_token)),
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Token(r_fence_token)),
        ],
    ))
}
pub fn md_link_block(
    l_brack_token: SyntaxToken,
    label: MdInlineItemList,
    r_brack_token: SyntaxToken,
    colon_token: SyntaxToken,
    url: MdInlineItemList,
) -> MdLinkBlock {
    MdLinkBlock::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_LINK_BLOCK,
        [
            Some(SyntaxElement::Token(l_brack_token)),
            Some(SyntaxElement::Node(label.into_syntax())),
            Some(SyntaxElement::Token(r_brack_token)),
            Some(SyntaxElement::Token(colon_token)),
            Some(SyntaxElement::Node(url.into_syntax())),
        ],
    ))
}
pub fn md_mdx_jsx_attribute(name: MdInlineItemList) -> MdMdxJsxAttributeBuilder {
    MdMdxJsxAttributeBuilder {
        name,
        eq_token: None,
        value: None,
    }
}
pub struct MdMdxJsxAttributeBuilder {
    name: MdInlineItemList,
    eq_token: Option<SyntaxToken>,
    value: Option<MdMdxJsxAttributeValue>,
}
impl MdMdxJsxAttributeBuilder {
    pub fn with_eq_token(mut self, eq_token: SyntaxToken) -> Self {
        self.eq_token = Some(eq_token);
        self
    }
    pub fn with_value(mut self, value: MdMdxJsxAttributeValue) -> Self {
        self.value = Some(value);
        self
    }
    pub fn build(self) -> MdMdxJsxAttribute {
        MdMdxJsxAttribute::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_MDX_JSX_ATTRIBUTE,
            [
                Some(SyntaxElement::Node(self.name.into_syntax())),
                self.eq_token.map(|token| SyntaxElement::Token(token)),
                self.value
                    .map(|token| SyntaxElement::Node(token.into_syntax())),
            ],
        ))
    }
}
pub fn md_mdx_jsx_attribute_value(
    delimiter_token: SyntaxToken,
    content: MdInlineItemList,
    closing_delimiter_token: SyntaxToken,
) -> MdMdxJsxAttributeValue {
    MdMdxJsxAttributeValue::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_MDX_JSX_ATTRIBUTE_VALUE,
        [
            Some(SyntaxElement::Token(delimiter_token)),
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Token(closing_delimiter_token)),
        ],
    ))
}
pub fn md_mdx_jsx_element(
    l_angle_token: SyntaxToken,
    name: MdInlineItemList,
    attributes: MdMdxJsxAttributeList,
    r_angle_token: SyntaxToken,
) -> MdMdxJsxElementBuilder {
    MdMdxJsxElementBuilder {
        l_angle_token,
        name,
        attributes,
        r_angle_token,
        slash_token: None,
    }
}
pub struct MdMdxJsxElementBuilder {
    l_angle_token: SyntaxToken,
    name: MdInlineItemList,
    attributes: MdMdxJsxAttributeList,
    r_angle_token: SyntaxToken,
    slash_token: Option<SyntaxToken>,
}
impl MdMdxJsxElementBuilder {
    pub fn with_slash_token(mut self, slash_token: SyntaxToken) -> Self {
        self.slash_token = Some(slash_token);
        self
    }
    pub fn build(self) -> MdMdxJsxElement {
        MdMdxJsxElement::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_MDX_JSX_ELEMENT,
            [
                Some(SyntaxElement::Token(self.l_angle_token)),
                Some(SyntaxElement::Node(self.name.into_syntax())),
                Some(SyntaxElement::Node(self.attributes.into_syntax())),
                self.slash_token.map(|token| SyntaxElement::Token(token)),
                Some(SyntaxElement::Token(self.r_angle_token)),
            ],
        ))
    }
}
pub fn md_order_bullet(marker_token: SyntaxToken, content: MdBlockList) -> MdOrderBulletBuilder {
    MdOrderBulletBuilder {
        marker_token,
        content,
        checkbox: None,
    }
}
pub struct MdOrderBulletBuilder {
    marker_token: SyntaxToken,
    content: MdBlockList,
    checkbox: Option<MdCheckbox>,
}
impl MdOrderBulletBuilder {
    pub fn with_checkbox(mut self, checkbox: MdCheckbox) -> Self {
        self.checkbox = Some(checkbox);
        self
    }
    pub fn build(self) -> MdOrderBullet {
        MdOrderBullet::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_ORDER_BULLET,
            [
                Some(SyntaxElement::Token(self.marker_token)),
                self.checkbox
                    .map(|token| SyntaxElement::Node(token.into_syntax())),
                Some(SyntaxElement::Node(self.content.into_syntax())),
            ],
        ))
    }
}
pub fn md_order_list_item(md_order_list: MdOrderList) -> MdOrderListItem {
    MdOrderListItem::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_ORDER_LIST_ITEM,
        [Some(SyntaxElement::Node(md_order_list.into_syntax()))],
    ))
}
pub fn md_paragraph(list: MdInlineItemList) -> MdParagraphBuilder {
    MdParagraphBuilder {
        list,
        hard_line: None,
    }
}
pub struct MdParagraphBuilder {
    list: MdInlineItemList,
    hard_line: Option<MdHardLine>,
}
impl MdParagraphBuilder {
    pub fn with_hard_line(mut self, hard_line: MdHardLine) -> Self {
        self.hard_line = Some(hard_line);
        self
    }
    pub fn build(self) -> MdParagraph {
        MdParagraph::unwrap_cast(SyntaxNode::new_detached(
            MarkdownSyntaxKind::MD_PARAGRAPH,
            [
                Some(SyntaxElement::Node(self.list.into_syntax())),
                self.hard_line
                    .map(|token| SyntaxElement::Node(token.into_syntax())),
            ],
        ))
    }
}
pub fn md_quote(r_angle_token: SyntaxToken, content: MdBlockList) -> MdQuote {
    MdQuote::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_QUOTE,
        [
            Some(SyntaxElement::Token(r_angle_token)),
            Some(SyntaxElement::Node(content.into_syntax())),
        ],
    ))
}
pub fn md_setext_header(content: MdParagraph, underline: MdTextual) -> MdSetextHeader {
    MdSetextHeader::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_SETEXT_HEADER,
        [
            Some(SyntaxElement::Node(content.into_syntax())),
            Some(SyntaxElement::Node(underline.into_syntax())),
        ],
    ))
}
pub fn md_soft_break(value_token: SyntaxToken) -> MdSoftBreak {
    MdSoftBreak::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_SOFT_BREAK,
        [Some(SyntaxElement::Token(value_token))],
    ))
}
pub fn md_table(header: MdTableRow, separator: MdTableRow, rows: MdTableRowList) -> MdTable {
    MdTable::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_TABLE,
        [
            Some(SyntaxElement::Node(header.into_syntax())),
            Some(SyntaxElement::Node(separator.into_syntax())),
            Some(SyntaxElement::Node(rows.into_syntax())),
        ],
    ))
}
pub fn md_table_row(content: MdInlineItemList) -> MdTableRow {
    MdTableRow::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_TABLE_ROW,
        [Some(SyntaxElement::Node(content.into_syntax()))],
    ))
}
pub fn md_textual(value_token: SyntaxToken) -> MdTextual {
    MdTextual::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_TEXTUAL,
        [Some(SyntaxElement::Token(value_token))],
    ))
}
pub fn md_thematic_break_block(value_token: SyntaxToken) -> MdThematicBreakBlock {
    MdThematicBreakBlock::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_THEMATIC_BREAK_BLOCK,
        [Some(SyntaxElement::Token(value_token))],
    ))
}
pub fn md_block_list<I>(items: I) -> MdBlockList
where
    I: IntoIterator<Item = AnyMdBlock>,
    I::IntoIter: ExactSizeIterator,
{
    MdBlockList::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_BLOCK_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn md_bullet_list<I>(items: I) -> MdBulletList
where
    I: IntoIterator<Item = MdBullet>,
    I::IntoIter: ExactSizeIterator,
{
    MdBulletList::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_BULLET_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn md_code_name_list<I>(items: I) -> MdCodeNameList
where
    I: IntoIterator<Item = MdTextual>,
    I::IntoIter: ExactSizeIterator,
{
    MdCodeNameList::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_CODE_NAME_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn md_directive_attribute_list<I>(items: I) -> MdDirectiveAttributeList
where
    I: IntoIterator<Item = MdDirectiveAttribute>,
    I::IntoIter: ExactSizeIterator,
{
    MdDirectiveAttributeList::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_DIRECTIVE_ATTRIBUTE_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn md_hash_list<I>(items: I) -> MdHashList
where
    I: IntoIterator<Item = MdHash>,
    I::IntoIter: ExactSizeIterator,
{
    MdHashList::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_HASH_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn md_indented_code_line_list<I>(items: I) -> MdIndentedCodeLineList
where
    I: IntoIterator<Item = MdIndentedCodeLine>,
    I::IntoIter: ExactSizeIterator,
{
    MdIndentedCodeLineList::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INDENTED_CODE_LINE_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn md_inline_item_list<I>(items: I) -> MdInlineItemList
where
    I: IntoIterator<Item = AnyMdInline>,
    I::IntoIter: ExactSizeIterator,
{
    MdInlineItemList::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_INLINE_ITEM_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn md_mdx_jsx_attribute_list<I>(items: I) -> MdMdxJsxAttributeList
where
    I: IntoIterator<Item = MdMdxJsxAttribute>,
    I::IntoIter: ExactSizeIterator,
{
    MdMdxJsxAttributeList::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_MDX_JSX_ATTRIBUTE_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn md_order_list<I>(items: I) -> MdOrderList
where
    I: IntoIterator<Item = MdOrderBullet>,
    I::IntoIter: ExactSizeIterator,
{
    MdOrderList::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_ORDER_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn md_table_row_list<I>(items: I) -> MdTableRowList
where
    I: IntoIterator<Item = MdTableRow>,
    I::IntoIter: ExactSizeIterator,
{
    MdTableRowList::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_TABLE_ROW_LIST,
        items
            .into_iter()
            .map(|item| Some(item.into_syntax().into())),
    ))
}
pub fn md_bogus<I>(slots: I) -> MdBogus
where
    I: IntoIterator<Item = Option<SyntaxElement>>,
    I::IntoIter: ExactSizeIterator,
{
    MdBogus::unwrap_cast(SyntaxNode::new_detached(
        MarkdownSyntaxKind::MD_BOGUS,
        slots,
    ))
}
