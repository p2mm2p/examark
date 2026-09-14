//! 固定的两级官方分类：模块与子分类。
//!
//! 两者都是封闭枚举——v1 只有四个模块与九个官方子分类，不接受自定义分类；
//! 未知名称与「子分类不属于其模块」都由解析器报错。

/// 模块：四个固定的顶级行测类别之一。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    /// 言语理解与表达。
    VerbalComprehension,
    /// 判断推理。
    JudgmentReasoning,
    /// 数量关系。
    QuantitativeRelations,
    /// 资料分析。
    DataAnalysis,
}

/// 子分类：模块内官方的二级分类，各归属一个模块。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubCategory {
    /// 言语理解与表达 · 逻辑填空。
    LogicalFill,
    /// 言语理解与表达 · 片段阅读。
    PassageReading,
    /// 言语理解与表达 · 语句表达。
    SentenceExpression,
    /// 判断推理 · 图形推理。
    FigureReasoning,
    /// 判断推理 · 定义判断。
    DefinitionJudgment,
    /// 判断推理 · 类比推理。
    AnalogyReasoning,
    /// 判断推理 · 逻辑判断。
    LogicalJudgment,
    /// 数量关系 · 数学运算。
    MathOperation,
    /// 数量关系 · 数字推理。
    NumberReasoning,
}

/// 言语理解与表达的子分类。
const VERBAL_SUB_CATEGORIES: [SubCategory; 3] = [
    SubCategory::LogicalFill,
    SubCategory::PassageReading,
    SubCategory::SentenceExpression,
];

/// 判断推理的子分类。
const JUDGMENT_SUB_CATEGORIES: [SubCategory; 4] = [
    SubCategory::FigureReasoning,
    SubCategory::DefinitionJudgment,
    SubCategory::AnalogyReasoning,
    SubCategory::LogicalJudgment,
];

/// 数量关系的子分类。
const QUANTITATIVE_SUB_CATEGORIES: [SubCategory; 2] =
    [SubCategory::MathOperation, SubCategory::NumberReasoning];

impl Module {
    /// 四个模块，按卷面顺序。
    pub const ALL: [Module; 4] = [
        Module::VerbalComprehension,
        Module::JudgmentReasoning,
        Module::QuantitativeRelations,
        Module::DataAnalysis,
    ];

    /// 模块的官方名称。
    pub fn name(self) -> &'static str {
        match self {
            Self::VerbalComprehension => "言语理解与表达",
            Self::JudgmentReasoning => "判断推理",
            Self::QuantitativeRelations => "数量关系",
            Self::DataAnalysis => "资料分析",
        }
    }

    /// 按官方名称取模块；不是官方名称时返回 `None`。
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|module| module.name() == name)
    }

    /// 该模块的官方子分类；资料分析没有子分类。
    pub fn sub_categories(self) -> &'static [SubCategory] {
        match self {
            Self::VerbalComprehension => &VERBAL_SUB_CATEGORIES,
            Self::JudgmentReasoning => &JUDGMENT_SUB_CATEGORIES,
            Self::QuantitativeRelations => &QUANTITATIVE_SUB_CATEGORIES,
            Self::DataAnalysis => &[],
        }
    }
}

impl SubCategory {
    /// 子分类的官方名称。
    pub fn name(self) -> &'static str {
        match self {
            Self::LogicalFill => "逻辑填空",
            Self::PassageReading => "片段阅读",
            Self::SentenceExpression => "语句表达",
            Self::FigureReasoning => "图形推理",
            Self::DefinitionJudgment => "定义判断",
            Self::AnalogyReasoning => "类比推理",
            Self::LogicalJudgment => "逻辑判断",
            Self::MathOperation => "数学运算",
            Self::NumberReasoning => "数字推理",
        }
    }

    /// 按官方名称取子分类；不是官方名称时返回 `None`。
    pub fn from_name(name: &str) -> Option<Self> {
        Module::ALL
            .into_iter()
            .flat_map(|module| module.sub_categories())
            .copied()
            .find(|sub_category| sub_category.name() == name)
    }

    /// 子分类所属的模块。
    pub fn module(self) -> Module {
        match self {
            Self::LogicalFill | Self::PassageReading | Self::SentenceExpression => {
                Module::VerbalComprehension
            }
            Self::FigureReasoning
            | Self::DefinitionJudgment
            | Self::AnalogyReasoning
            | Self::LogicalJudgment => Module::JudgmentReasoning,
            Self::MathOperation | Self::NumberReasoning => Module::QuantitativeRelations,
        }
    }
}
