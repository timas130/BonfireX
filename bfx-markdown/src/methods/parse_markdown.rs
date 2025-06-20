use crate::MarkdownService;
use bfx_proto::markdown::node::Node as ProtoNode;
use bfx_proto::markdown::{
    AlignKind as ProtoAlignKind, BlockquoteNode, BreakNode, CodeNode, DefinitionNode, DeleteNode,
    EmphasisNode, FootnoteDefinition, FootnoteReferenceNode, HeadingNode, ImageNode,
    InlineCodeNode, LinkNode, ListItemNode, ListNode, Node as NodeWrapper, NoopNode, ParagraphNode,
    ParseMarkdownReply, ParseMarkdownRequest, RootNode, StrongNode, TableCellNode, TableNode,
    TableRowNode, TextNode, ThematicBreakNode,
};
use markdown::mdast::{AlignKind, Node};
use markdown::{ParseOptions, to_mdast};
use tonic::{Request, Response, Status};

impl MarkdownService {
    // for consistency with other methods
    #[allow(
        clippy::unnecessary_wraps,
        clippy::unused_self,
        clippy::result_large_err
    )]
    pub fn parse_markdown(
        &self,
        request: Request<ParseMarkdownRequest>,
    ) -> Result<Response<ParseMarkdownReply>, Status> {
        let request = request.into_inner();

        let tree = to_mdast(&request.text, &ParseOptions::gfm()).unwrap();
        let tree = node_to_proto(tree);

        Ok(Response::new(ParseMarkdownReply {
            parsed: Some(NodeWrapper { node: Some(tree) }),
        }))
    }
}

#[allow(clippy::too_many_lines)]
fn node_to_proto(value: Node) -> ProtoNode {
    fn map_children(children: Vec<Node>) -> Vec<NodeWrapper> {
        children
            .into_iter()
            .map(node_to_proto)
            .map(|node| NodeWrapper { node: Some(node) })
            .collect()
    }

    match value {
        Node::Root(node) => ProtoNode::Root(RootNode {
            children: map_children(node.children),
        }),
        Node::Blockquote(node) => ProtoNode::Blockquote(BlockquoteNode {
            children: map_children(node.children),
        }),
        Node::FootnoteDefinition(node) => ProtoNode::FootnoteDefinition(FootnoteDefinition {
            children: map_children(node.children),
            identifier: node.identifier,
            label: node.label,
        }),
        Node::List(node) => ProtoNode::List(ListNode {
            children: map_children(node.children),
            ordered: node.ordered,
            spread: node.spread,
            start: node.start,
        }),
        Node::Break(_) => ProtoNode::Break(BreakNode {}),
        Node::InlineCode(node) => ProtoNode::InlineCode(InlineCodeNode { value: node.value }),
        Node::Delete(node) => ProtoNode::Delete(DeleteNode {
            children: map_children(node.children),
        }),
        Node::Emphasis(node) => ProtoNode::Emphasis(EmphasisNode {
            children: map_children(node.children),
        }),
        Node::FootnoteReference(node) => ProtoNode::FootnoteReference(FootnoteReferenceNode {
            label: node.label,
            identifier: node.identifier,
        }),
        Node::Image(node) => ProtoNode::Image(ImageNode {
            alt: node.alt,
            title: node.title,
            url: node.url,
        }),
        Node::Link(node) => ProtoNode::Link(LinkNode {
            url: node.url,
            title: node.title,
            children: map_children(node.children),
        }),
        Node::Strong(node) => ProtoNode::Strong(StrongNode {
            children: map_children(node.children),
        }),
        Node::Text(node) => ProtoNode::Text(TextNode { value: node.value }),
        Node::Code(node) => ProtoNode::Code(CodeNode {
            value: node.value,
            lang: node.lang,
            meta: node.meta,
        }),
        Node::Heading(node) => ProtoNode::Heading(HeadingNode {
            children: map_children(node.children),
            depth: u32::from(node.depth),
        }),
        Node::Table(node) => ProtoNode::Table(TableNode {
            children: map_children(node.children),
            align: node
                .align
                .into_iter()
                .map(|x| {
                    match x {
                        AlignKind::Left => ProtoAlignKind::Left,
                        AlignKind::Right => ProtoAlignKind::Right,
                        AlignKind::Center => ProtoAlignKind::Center,
                        AlignKind::None => ProtoAlignKind::None,
                    }
                    .into()
                })
                .collect(),
        }),
        Node::ThematicBreak(_) => ProtoNode::ThematicBreak(ThematicBreakNode {}),
        Node::TableRow(node) => ProtoNode::TableRow(TableRowNode {
            children: map_children(node.children),
        }),
        Node::TableCell(node) => ProtoNode::TableCell(TableCellNode {
            children: map_children(node.children),
        }),
        Node::ListItem(node) => ProtoNode::ListItem(ListItemNode {
            children: map_children(node.children),
            spread: node.spread,
            checked: node.checked,
        }),
        Node::Definition(node) => ProtoNode::Definition(DefinitionNode {
            title: node.title,
            url: node.url,
            identifier: node.identifier,
            label: node.label,
        }),
        Node::Paragraph(node) => ProtoNode::Paragraph(ParagraphNode {
            children: map_children(node.children),
        }),

        Node::MdxJsxFlowElement(_)
        | Node::MdxjsEsm(_)
        | Node::Toml(_)
        | Node::Yaml(_)
        | Node::InlineMath(_)
        | Node::MdxTextExpression(_)
        | Node::Html(_)
        | Node::ImageReference(_)
        | Node::MdxJsxTextElement(_)
        | Node::LinkReference(_)
        | Node::Math(_)
        | Node::MdxFlowExpression(_) => ProtoNode::Noop(NoopNode {}),
    }
}
