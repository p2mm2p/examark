//! emark 格式的 AST、解析器与 HTML 渲染器。

mod ast;
mod category;
mod error;
mod parser;
mod render;

pub use ast::{
    Choice, Content, Document, Inline, MaterialQuestion, Metadata, Paragraph, Question, Section,
    SingleChoice,
};
pub use category::{Module, SubCategory};
pub use error::ParseError;
pub use parser::parse;
pub use render::{render, render_with_assets};

/// 格式名称。
pub const FORMAT_NAME: &str = "emark";

/// 格式的文件扩展名，含前导点号。
pub const FORMAT_EXTENSION: &str = ".emark";
