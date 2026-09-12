//! examark 文档的抽象语法树。

use crate::category::{Module, SubCategory};

/// 一份题目文档。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// 文档级元数据；作者未写时为各字段 `None`。
    pub metadata: Metadata,
    /// 模块分节，按文档顺序。
    pub sections: Vec<Section>,
}

/// 文档级元数据。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Metadata {
    /// 题名。
    pub title: Option<String>,
    /// 考试名称。
    pub exam: Option<String>,
    /// 年份。
    pub year: Option<String>,
    /// 卷别，如副省级、地市级、行政执法。
    pub paper: Option<String>,
    /// 地区，省考用。
    pub region: Option<String>,
    /// 出处。
    pub source: Option<String>,
    /// 参考时限，单位分钟。
    pub duration: Option<String>,
    /// 总分。
    pub score: Option<String>,
}

impl Metadata {
    /// 受支持的元数据键。
    pub(crate) const KEYS: [&str; 8] = [
        "title", "exam", "year", "paper", "region", "source", "duration", "score",
    ];

    /// 该关键字是否是元数据键。
    pub(crate) fn is_key(keyword: &str) -> bool {
        Self::KEYS.contains(&keyword)
    }

    /// 拿下该关键字对应的字段；不是元数据键时返回 `None`。
    pub(crate) fn slot(&mut self, keyword: &str) -> Option<&mut Option<String>> {
        Some(match keyword {
            "title" => &mut self.title,
            "exam" => &mut self.exam,
            "year" => &mut self.year,
            "paper" => &mut self.paper,
            "region" => &mut self.region,
            "source" => &mut self.source,
            "duration" => &mut self.duration,
            "score" => &mut self.score,
            _ => return None,
        })
    }
}

/// 模块分节：声明一次模块，承载若干题目。同一模块可占多个分节。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// 分节声明的模块。
    pub module: Module,
    /// 子分类；作者未声明、或该模块本就没有子分类时为 `None`。
    pub sub_category: Option<SubCategory>,
    /// 本节内的题目，按文档顺序。
    pub questions: Vec<Question>,
}

/// 单选题。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// 文档顺序序号，从 1 开始；由解析器分配，作者从不手写。
    pub number: usize,
    /// 题干。
    pub stem: String,
    /// 四个选项，依次对应 A、B、C、D。
    pub options: [String; 4],
    /// 正确答案。
    pub answer: Choice,
    /// 解析；作者未写时为 `None`。
    pub explanation: Option<String>,
}

/// 单选题的正确选项。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    A,
    B,
    C,
    D,
}

impl Choice {
    /// 该选项的字母。
    pub(crate) fn letter(self) -> char {
        match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
            Self::D => 'D',
        }
    }
}
