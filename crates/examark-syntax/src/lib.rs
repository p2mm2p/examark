//! examark 格式的 AST、解析器与 HTML 渲染器。

mod ast;
mod error;
mod parser;

pub use ast::{Choice, Document, Metadata, Question, Section};
pub use error::ParseError;
pub use parser::parse;

/// 格式名称。
pub const FORMAT_NAME: &str = "examark";
