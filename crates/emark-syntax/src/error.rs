//! 解析错误。

use std::error::Error;
use std::fmt;

/// 解析失败：出错行号加人类可读的说明。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    line: usize,
    message: String,
}

impl ParseError {
    pub(crate) fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }

    /// 出错行号，从 1 开始。
    pub fn line(&self) -> usize {
        self.line
    }

    /// 错误说明（不含行号）。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "第 {} 行：{}", self.line, self.message)
    }
}

impl Error for ParseError {}
