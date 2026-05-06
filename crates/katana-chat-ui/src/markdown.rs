mod inline;
mod parser;

use serde::{Deserialize, Serialize};

pub use parser::MarkdownParser;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownSubset {
    pub blocks: Vec<MarkdownBlock>,
}

impl MarkdownSubset {
    pub fn parse(source: &str) -> Self {
        MarkdownParser::new(source).parse()
    }

    pub fn plain_text(&self) -> String {
        let mut text = String::new();
        for block in &self.blocks {
            text.push_str(&block.plain_text());
            text.push('\n');
        }
        text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkdownBlock {
    Heading(HeadingBlock),
    Paragraph(TextBlock),
    CodeBlock(CodeBlock),
    BlockQuote(TextBlock),
    List(ListBlock),
    Table(TableBlock),
}

impl MarkdownBlock {
    fn plain_text(&self) -> String {
        match self {
            Self::Heading(block) => block.text.plain_text(),
            Self::Paragraph(block) | Self::BlockQuote(block) => block.plain_text(),
            Self::CodeBlock(block) => block.code.clone(),
            Self::List(block) => block.plain_text(),
            Self::Table(block) => block.plain_text(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadingBlock {
    pub level: u8,
    pub text: TextBlock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextBlock {
    pub segments: Vec<InlineSegment>,
}

impl TextBlock {
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            segments: vec![InlineSegment::Text(text.into())],
        }
    }

    pub fn plain_text(&self) -> String {
        let mut text = String::new();
        for segment in &self.segments {
            text.push_str(&segment.plain_text());
        }
        text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InlineSegment {
    Text(String),
    Emphasis(String),
    Strong(String),
    Strikethrough(String),
    InlineCode(String),
    Link { text: String, destination: String },
    Image { alt: String, destination: String },
}

impl InlineSegment {
    fn plain_text(&self) -> String {
        match self {
            Self::Text(text)
            | Self::Emphasis(text)
            | Self::Strong(text)
            | Self::Strikethrough(text)
            | Self::InlineCode(text) => text.clone(),
            Self::Link { text, .. } => text.clone(),
            Self::Image { alt, destination } => {
                if alt.is_empty() {
                    destination.clone()
                } else {
                    alt.clone()
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListBlock {
    pub kind: ListKind,
    pub items: Vec<ListItem>,
}

impl ListBlock {
    fn plain_text(&self) -> String {
        let mut text = String::new();
        for item in &self.items {
            text.push_str(&item.plain_text());
            text.push('\n');
        }
        text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub content: TextBlock,
}

impl ListItem {
    pub fn plain_text(&self) -> String {
        self.content.plain_text()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListKind {
    Ordered { start: u64 },
    Unordered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableBlock {
    pub headers: Vec<TextBlock>,
    pub rows: Vec<Vec<TextBlock>>,
}

impl TableBlock {
    fn plain_text(&self) -> String {
        let mut lines = Vec::new();
        if !self.headers.is_empty() {
            lines.push(join_cells(&self.headers));
        }
        for row in &self.rows {
            lines.push(join_cells(row));
        }
        lines.join("\n")
    }
}

fn join_cells(cells: &[TextBlock]) -> String {
    cells
        .iter()
        .map(TextBlock::plain_text)
        .collect::<Vec<_>>()
        .join(" | ")
}

#[cfg(test)]
mod tests;
