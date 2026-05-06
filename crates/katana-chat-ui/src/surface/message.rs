use crate::{CodeBlock, MarkdownBlock, MessageRenderModel, markdown::ListBlock};

pub(super) struct MessageSurfaceBuilder;

impl MessageSurfaceBuilder {
    pub(super) fn body(model: &MessageRenderModel) -> String {
        model
            .blocks
            .iter()
            .map(block_body)
            .filter(|it| !it.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn block_body(block: &MarkdownBlock) -> String {
    match block {
        MarkdownBlock::Heading(heading) => heading.text.plain_text(),
        MarkdownBlock::Paragraph(text) | MarkdownBlock::BlockQuote(text) => text.plain_text(),
        MarkdownBlock::CodeBlock(code) => code_body(code),
        MarkdownBlock::List(list) => list_body(list),
        MarkdownBlock::Table(table) => table
            .headers
            .iter()
            .chain(table.rows.iter().flatten())
            .map(|it| it.plain_text())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn code_body(code: &CodeBlock) -> String {
    match &code.language {
        Some(language) => format!("```{language}\n{}\n```", code.code),
        None => format!("```\n{}\n```", code.code),
    }
}

fn list_body(list: &ListBlock) -> String {
    list.items
        .iter()
        .map(|it| it.content.plain_text())
        .collect::<Vec<_>>()
        .join("\n")
}
