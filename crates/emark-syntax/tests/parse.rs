//! `emark-syntax` 解析接缝的测试：字符串进，AST 出。
//!
//! 语法：`@module` 顶格，其下每深一级缩进 2 个空格；内容行比它的关键字深一级。
//! 正文里的行内元素是 `@math{…}`、`@image{…}` 与裸 token `@blank`。

use emark_syntax::{
    Choice, Content, Document, Inline, MaterialQuestion, Module, ParseError, Question,
    SingleChoice, SubCategory, parse,
};

fn parse_ok(source: &str) -> Document {
    parse(source).expect("文档应解析成功")
}

fn parse_err(source: &str) -> ParseError {
    parse(source).expect_err("文档应解析失败")
}

/// 独立单选题的字段。
fn single(question: &Question) -> &SingleChoice {
    match question {
        Question::Single(single) => single,
        Question::Material(_) => panic!("应为独立单选题，实际是材料题"),
    }
}

/// 材料题的字段。
fn material(question: &Question) -> &MaterialQuestion {
    match question {
        Question::Material(material) => material,
        Question::Single(_) => panic!("应为材料题，实际是独立单选题"),
    }
}

/// 把内容还原成源文本的样子：段落用空行连接，行内元素写回记号。
fn source_of(content: &Content) -> String {
    content
        .paragraphs
        .iter()
        .map(|paragraph| {
            paragraph
                .0
                .iter()
                .map(|inline| match inline {
                    Inline::Text(text) => text.clone(),
                    Inline::Math(source) => format!("@math{{{source}}}"),
                    Inline::Image(path) => format!("@image{{{path}}}"),
                    Inline::Blank => "@blank".to_string(),
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// 单选题四个选项还原成源文本。
fn options_of(question: &SingleChoice) -> Vec<String> {
    question.options.iter().map(source_of).collect()
}

#[test]
fn parses_metadata_sections_and_questions() {
    let source = "\
@title 2025 年国考行测模拟卷（一）
@exam 国家公务员录用考试
@year 2025
@paper 副省级
@source 网络整理

@module 言语理解与表达
  @subcategory 逻辑填空

  @question
    @stem 填入划横线部分最恰当的一项是：
      他山之石，可以攻玉。两者差异。
    @option A 截然不同
    @option B 大相径庭
    @option C 迥然不同
    @option D 天壤之别
    @answer B
    @explanation 上下文强调差距悬殊，故选 B。
";

    let document = parse_ok(source);

    assert_eq!(
        document.metadata.title.as_deref(),
        Some("2025 年国考行测模拟卷（一）")
    );
    assert_eq!(
        document.metadata.exam.as_deref(),
        Some("国家公务员录用考试")
    );
    assert_eq!(document.metadata.year.as_deref(), Some("2025"));
    assert_eq!(document.metadata.paper.as_deref(), Some("副省级"));
    assert_eq!(document.metadata.source.as_deref(), Some("网络整理"));

    assert_eq!(document.sections.len(), 1);

    let section = &document.sections[0];
    assert_eq!(section.module, Module::VerbalComprehension);
    assert_eq!(section.sub_category, Some(SubCategory::LogicalFill));
    assert_eq!(section.questions.len(), 1);

    let question = single(&section.questions[0]);
    assert_eq!(question.number, 1);
    assert_eq!(
        source_of(&question.stem),
        "填入划横线部分最恰当的一项是：\n他山之石，可以攻玉。两者差异。"
    );
    assert_eq!(
        options_of(question),
        ["截然不同", "大相径庭", "迥然不同", "天壤之别"]
    );
    assert_eq!(question.answer, Choice::B);
    assert_eq!(
        source_of(question.explanation.as_ref().unwrap()),
        "上下文强调差距悬殊，故选 B。"
    );
}

#[test]
fn metadata_and_sub_category_are_optional() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer D
";

    let document = parse_ok(source);

    assert_eq!(document.metadata.title, None);
    assert_eq!(document.sections[0].sub_category, None);
    assert_eq!(single(&document.sections[0].questions[0]).explanation, None);
}

/// 一份最小合法文档：声明模块与（可选）子分类，带一道单选题。
fn document_with(module: &str, sub_category: Option<&str>) -> String {
    let sub_category = sub_category
        .map(|name| format!("  @subcategory {name}\n"))
        .unwrap_or_default();

    format!(
        "@module {module}\n{sub_category}\n  @question\n    @stem 题干。\n    @option A 甲\n    @option B 乙\n    @option C 丙\n    @option D 丁\n    @answer A\n"
    )
}

#[test]
fn the_official_two_level_classification_is_closed() {
    let expected = [
        (
            Module::VerbalComprehension,
            &["逻辑填空", "片段阅读", "语句表达"][..],
        ),
        (
            Module::JudgmentReasoning,
            &["图形推理", "定义判断", "类比推理", "逻辑判断"][..],
        ),
        (Module::QuantitativeRelations, &["数学运算", "数字推理"][..]),
        (Module::DataAnalysis, &[][..]),
    ];

    let modules = expected
        .iter()
        .map(|(module, _)| *module)
        .collect::<Vec<_>>();
    assert_eq!(Module::ALL.to_vec(), modules);

    for (module, names) in expected {
        let sub_categories = module
            .sub_categories()
            .iter()
            .map(|sub_category| sub_category.name())
            .collect::<Vec<_>>();

        assert_eq!(sub_categories, names, "{}", module.name());
        for sub_category in module.sub_categories() {
            assert_eq!(sub_category.module(), module, "{}", sub_category.name());
        }
    }
}

#[test]
fn accepts_every_official_module_and_sub_category() {
    for module in Module::ALL {
        let cases: Vec<Option<SubCategory>> = if module.sub_categories().is_empty() {
            vec![None]
        } else {
            module.sub_categories().iter().copied().map(Some).collect()
        };

        for expected in cases {
            let source = document_with(module.name(), expected.map(SubCategory::name));
            let document = parse_ok(&source);

            assert_eq!(document.sections[0].module, module, "{source}");
            assert_eq!(document.sections[0].sub_category, expected, "{source}");
        }
    }
}

#[test]
fn unknown_module_is_an_error() {
    let error = parse_err("@module 常识判断\n");

    assert_eq!(error.line(), 1);
    assert!(error.message().contains("常识判断"), "{}", error.message());
    assert!(
        error.message().contains("言语理解与表达"),
        "错误应列出官方模块：{}",
        error.message()
    );
}

#[test]
fn sub_category_of_another_module_is_an_error() {
    let error = parse_err(&document_with("数量关系", Some("片段阅读")));

    assert_eq!(error.line(), 2);
    assert!(error.message().contains("片段阅读"), "{}", error.message());
    assert!(
        error.message().contains("言语理解与表达"),
        "错误应指出子分类所属的模块：{}",
        error.message()
    );
    assert!(
        error.message().contains("数量关系"),
        "错误应指出所在的模块分节：{}",
        error.message()
    );
}

#[test]
fn unknown_sub_category_is_an_error() {
    let error = parse_err(&document_with("数量关系", Some("速算技巧")));

    assert_eq!(error.line(), 2);
    assert!(error.message().contains("速算技巧"), "{}", error.message());
    assert!(
        error.message().contains("数学运算"),
        "错误应列出该模块的官方子分类：{}",
        error.message()
    );
}

#[test]
fn data_analysis_rejects_any_sub_category() {
    let error = parse_err(&document_with("资料分析", Some("统计图")));

    assert_eq!(error.line(), 2);
    assert!(error.message().contains("统计图"), "{}", error.message());
    assert!(
        error.message().contains("资料分析"),
        "错误应指出该模块没有子分类：{}",
        error.message()
    );
}

#[test]
fn numbers_questions_in_document_order_across_sections() {
    let source = "\
@module 言语理解与表达

  @question
    @stem 第一题。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A

  @question
    @stem 第二题。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer B

@module 数量关系

  @question
    @stem 第三题。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer C
";

    let document = parse_ok(source);

    assert_eq!(document.sections.len(), 2);
    let numbers: Vec<usize> = document
        .sections
        .iter()
        .flat_map(|section| {
            section
                .questions
                .iter()
                .map(|question| single(question).number)
        })
        .collect();
    assert_eq!(numbers, [1, 2, 3]);
}

#[test]
fn stem_keeps_inner_blank_lines() {
    let source = "\
@module 数量关系

  @question
    @stem 第一段题干。

      第二段题干。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let document = parse_ok(source);

    assert_eq!(
        source_of(&single(&document.sections[0].questions[0]).stem),
        "第一段题干。\n\n第二段题干。"
    );
}

#[test]
fn accepts_all_four_answer_letters() {
    let expected = [Choice::A, Choice::B, Choice::C, Choice::D];

    for (index, (letter, choice)) in ["A", "B", "C", "D"].iter().zip(expected).enumerate() {
        let source = format!(
            "@module 资料分析\n\n  @question\n    @stem 第 {index} 题。\n    @option A 甲\n    @option B 乙\n    @option C 丙\n    @option D 丁\n    @answer {letter}\n"
        );

        let document = parse_ok(&source);

        assert_eq!(single(&document.sections[0].questions[0]).answer, choice);
    }
}

#[test]
fn accepts_windows_line_endings() {
    let source = "@module 资料分析\r\n\r\n  @question\r\n    @stem 题干。\r\n    @option A 甲\r\n    @option B 乙\r\n    @option C 丙\r\n    @option D 丁\r\n    @answer B\r\n";

    let document = parse_ok(source);

    assert_eq!(single(&document.sections[0].questions[0]).answer, Choice::B);
}

#[test]
fn accepts_a_utf8_bom() {
    let source = "\u{feff}@module 资料分析\n\n  @question\n    @stem 题干。\n    @option A 甲\n    @option B 乙\n    @option C 丙\n    @option D 丁\n    @answer A\n";

    let document = parse_ok(source);

    assert_eq!(document.sections[0].module, Module::DataAnalysis);
}

#[test]
fn blank_lines_between_blocks_are_ignored() {
    let source = "\
@module 判断推理



  @question

    @stem 题干。

    @option A 甲

    @option B 乙

    @option C 丙

    @option D 丁

    @answer D

    @explanation 解析。
";

    let document = parse_ok(source);

    let question = single(&document.sections[0].questions[0]);
    assert_eq!(options_of(question), ["甲", "乙", "丙", "丁"]);
    assert_eq!(source_of(question.explanation.as_ref().unwrap()), "解析。");
}

#[test]
fn error_displays_the_line_and_the_message() {
    let error = parse_err("@module 资料分析\n");

    let rendered = error.to_string();
    assert!(rendered.contains("第 1 行"), "{rendered}");
    assert!(rendered.contains(error.message()), "{rendered}");
}

#[test]
fn empty_document_is_an_error() {
    let error = parse_err("");

    assert_eq!(error.line(), 1);
    assert!(error.message().contains("空"), "{}", error.message());
}

#[test]
fn document_with_only_metadata_is_an_error() {
    let error = parse_err("@title 某卷\n");

    assert_eq!(error.line(), 1);
    assert!(error.message().contains("模块分节"), "{}", error.message());
}

#[test]
fn unknown_metadata_key_is_an_error() {
    let error = parse_err("@author 张三\n\n@module 资料分析\n");

    assert_eq!(error.line(), 1);
    assert!(error.message().contains("author"), "{}", error.message());
}

#[test]
fn duplicate_metadata_key_is_an_error() {
    let error = parse_err("@year 2025\n@year 2026\n\n@module 资料分析\n");

    assert_eq!(error.line(), 2);
    assert!(error.message().contains("year"), "{}", error.message());
}

#[test]
fn metadata_without_a_value_is_an_error() {
    let error = parse_err("@title\n\n@module 资料分析\n");

    assert_eq!(error.line(), 1);
    assert!(error.message().contains("title"), "{}", error.message());
}

#[test]
fn metadata_after_a_section_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A

@year 2025
";

    let error = parse_err(source);

    assert_eq!(error.line(), 11);
    assert!(error.message().contains("开头"), "{}", error.message());
}

#[test]
fn section_without_a_module_name_is_an_error() {
    let error = parse_err("@module\n");

    assert_eq!(error.line(), 1);
    assert!(error.message().contains("模块名"), "{}", error.message());
}

#[test]
fn section_without_questions_is_an_error() {
    let error = parse_err("@module 资料分析\n");

    assert_eq!(error.line(), 1);
    assert!(error.message().contains("题目"), "{}", error.message());
}

#[test]
fn sub_category_after_a_question_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A

  @subcategory 资料分析
";

    let error = parse_err(source);

    assert_eq!(error.line(), 11);
    assert!(error.message().contains("子分类"), "{}", error.message());
}

#[test]
fn question_outside_a_section_is_an_error() {
    let error = parse_err("@question\n  @stem 题干。\n");

    assert_eq!(error.line(), 1);
    assert!(error.message().contains("分节"), "{}", error.message());
}

#[test]
fn tab_indentation_is_an_error() {
    let source = "@module 资料分析\n\n\t@question\n";

    let error = parse_err(source);

    assert_eq!(error.line(), 3);
    assert!(error.message().contains("TAB"), "{}", error.message());
}

#[test]
fn wrong_indentation_is_an_error() {
    let source = "\
@module 资料分析

   @question
";

    let error = parse_err(source);

    assert_eq!(error.line(), 3);
    assert!(error.message().contains('2'), "{}", error.message());
}

#[test]
fn stem_continuation_at_the_wrong_indent_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 第一行。
  第二行。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 5);
    assert!(error.message().contains("内容行"), "{}", error.message());
}

#[test]
fn unknown_keyword_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
    @difficulty 高
";

    let error = parse_err(source);

    assert_eq!(error.line(), 10);
    assert!(
        error.message().contains("difficulty"),
        "{}",
        error.message()
    );
}

#[test]
fn unrecognized_line_is_an_error() {
    let source = "\
@module 资料分析

这是一行没有关键字的文本。
";

    let error = parse_err(source);

    assert_eq!(error.line(), 3);
    assert!(error.message().contains('@'), "{}", error.message());
}

#[test]
fn question_does_not_take_a_value() {
    let source = "\
@module 资料分析

  @question 一号题
";

    let error = parse_err(source);

    assert_eq!(error.line(), 3);
    assert!(error.message().contains("@question"), "{}", error.message());
}

#[test]
fn question_without_a_stem_is_an_error() {
    let error = parse_err("@module 资料分析\n\n  @question\n");

    assert_eq!(error.line(), 3);
    assert!(error.message().contains("题干"), "{}", error.message());
}

#[test]
fn a_field_other_than_the_stem_where_the_stem_belongs_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 4);
    assert!(error.message().contains("@stem"), "{}", error.message());
}

#[test]
fn empty_stem_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 4);
    assert!(error.message().contains("题干"), "{}", error.message());
}

#[test]
fn missing_option_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 7);
    assert!(error.message().contains('C'), "{}", error.message());
}

#[test]
fn options_out_of_order_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option D 丁
    @option C 丙
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 7);
    assert!(error.message().contains('C'), "{}", error.message());
}

#[test]
fn option_without_a_letter_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 5);
    assert!(error.message().contains("字母"), "{}", error.message());
}

#[test]
fn option_letter_outside_a_to_d_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option E 戊
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 5);
    assert!(error.message().contains('A'), "{}", error.message());
}

#[test]
fn option_without_a_value_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 5);
    assert!(error.message().contains("选项"), "{}", error.message());
}

#[test]
fn missing_answer_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
";

    let error = parse_err(source);

    assert_eq!(error.line(), 3);
    assert!(error.message().contains("答案"), "{}", error.message());
}

#[test]
fn answer_outside_a_to_d_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer E
";

    let error = parse_err(source);

    assert_eq!(error.line(), 9);
    assert!(error.message().contains("答案"), "{}", error.message());
}

#[test]
fn duplicate_answer_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
    @answer B
";

    let error = parse_err(source);

    assert_eq!(error.line(), 10);
    assert!(error.message().contains("答案"), "{}", error.message());
}

#[test]
fn answer_before_the_options_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @answer A
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
";

    let error = parse_err(source);

    assert_eq!(error.line(), 5);
    assert!(error.message().contains("答案"), "{}", error.message());
}

#[test]
fn explanation_before_the_answer_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @explanation 先写了。
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 9);
    assert!(error.message().contains("解析"), "{}", error.message());
}

#[test]
fn explanation_without_content_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
    @explanation
";

    let error = parse_err(source);

    assert_eq!(error.line(), 10);
    assert!(error.message().contains("解析"), "{}", error.message());
}

#[test]
fn parses_a_material_question_with_its_sub_questions() {
    let source = "\
@module 资料分析

  @material
    根据以下资料，回答下列问题。

    @image{assets/表1.png}

  @question
    @stem 第一小题。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A

  @question
    @stem 第二小题。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer D
    @explanation 由图可知。
";

    let document = parse_ok(source);

    assert_eq!(document.sections[0].questions.len(), 1);

    let material_question = material(&document.sections[0].questions[0]);
    assert_eq!(
        source_of(&material_question.material),
        "根据以下资料，回答下列问题。\n\n@image{assets/表1.png}"
    );

    let paragraphs = &material_question.material.paragraphs;
    assert_eq!(paragraphs.len(), 2);
    assert_eq!(
        paragraphs[1].0,
        [Inline::Image("assets/表1.png".to_string())]
    );

    assert_eq!(material_question.questions.len(), 2);
    assert_eq!(material_question.questions[0].number, 1);
    assert_eq!(material_question.questions[1].number, 2);
    assert_eq!(material_question.questions[1].answer, Choice::D);
    assert_eq!(
        source_of(material_question.questions[1].explanation.as_ref().unwrap()),
        "由图可知。"
    );
}

#[test]
fn parses_math_images_and_blanks_as_their_own_nodes() {
    let source = "\
@module 数量关系

  @question
    @stem 当 @math{x>0} 时，@image{assets/图1.png} 成立，在 @blank 处填数。
    @option A @math{\\frac{1}{2}}
    @option B 乙
    @option C 丙
    @option D @image{assets/选项丁.png}
    @answer A
    @explanation 由 @math{x>0} 得。
";

    let document = parse_ok(source);
    let question = single(&document.sections[0].questions[0]);

    assert_eq!(
        question.stem.paragraphs[0].0,
        [
            Inline::Text("当 ".to_string()),
            Inline::Math("x>0".to_string()),
            Inline::Text(" 时，".to_string()),
            Inline::Image("assets/图1.png".to_string()),
            Inline::Text(" 成立，在 ".to_string()),
            Inline::Blank,
            Inline::Text(" 处填数。".to_string()),
        ]
    );

    assert_eq!(
        question.options[0].paragraphs[0].0,
        [Inline::Math("\\frac{1}{2}".to_string())]
    );
    assert_eq!(
        question.options[3].paragraphs[0].0,
        [Inline::Image("assets/选项丁.png".to_string())]
    );
    assert_eq!(
        question.explanation.as_ref().unwrap().paragraphs[0].0,
        [
            Inline::Text("由 ".to_string()),
            Inline::Math("x>0".to_string()),
            Inline::Text(" 得。".to_string()),
        ]
    );
}

#[test]
fn math_braces_balance_and_escaped_braces_do_not_count() {
    let source = "\
@module 数量关系

  @question
    @stem @math{f(x)=\\{x\\}} 与 @math{\\frac{a}{b}}。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let document = parse_ok(source);
    let stem = &single(&document.sections[0].questions[0]).stem;

    assert_eq!(
        stem.paragraphs[0].0,
        [
            Inline::Math("f(x)=\\{x\\}".to_string()),
            Inline::Text(" 与 ".to_string()),
            Inline::Math("\\frac{a}{b}".to_string()),
            Inline::Text("。".to_string()),
        ]
    );
}

#[test]
fn each_material_binds_the_questions_that_follow_it() {
    let source = "\
@module 资料分析

  @question
    @stem 独立题。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A

  @material
    材料一。

  @question
    @stem 材料一的小题。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer B

  @material
    材料二。

  @question
    @stem 材料二的小题。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer C
";

    let document = parse_ok(source);
    let questions = &document.sections[0].questions;

    assert_eq!(questions.len(), 3);
    assert_eq!(single(&questions[0]).number, 1);
    assert_eq!(single(&questions[0]).answer, Choice::A);

    let first = material(&questions[1]);
    assert_eq!(source_of(&first.material), "材料一。");
    assert_eq!(first.questions.len(), 1);
    assert_eq!(first.questions[0].number, 2);
    assert_eq!(first.questions[0].answer, Choice::B);

    let second = material(&questions[2]);
    assert_eq!(source_of(&second.material), "材料二。");
    assert_eq!(second.questions.len(), 1);
    assert_eq!(second.questions[0].number, 3);
    assert_eq!(second.questions[0].answer, Choice::C);
}

#[test]
fn inline_elements_do_not_span_lines() {
    let source = "\
@module 数量关系

  @question
    @stem 公式 @math{x
      + y} 没闭合。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 4);
    assert!(error.message().contains('}'), "{}", error.message());
    assert!(error.message().contains("跨行"), "{}", error.message());
}

#[test]
fn unclosed_math_is_an_error() {
    let error = parse_err(&document_stem_with("公式 @math{x+1。"));

    assert_eq!(error.line(), 4);
    assert!(error.message().contains("@math"), "{}", error.message());
    assert!(error.message().contains('}'), "{}", error.message());
}

#[test]
fn empty_math_is_an_error() {
    let error = parse_err(&document_stem_with("公式 @math{}。"));

    assert_eq!(error.line(), 4);
    assert!(error.message().contains("@math"), "{}", error.message());
    assert!(error.message().contains("空"), "{}", error.message());
}

#[test]
fn math_without_braces_is_an_error() {
    let error = parse_err(&document_stem_with("公式 @math x。"));

    assert_eq!(error.line(), 4);
    assert!(error.message().contains("@math"), "{}", error.message());
    assert!(error.message().contains('{'), "{}", error.message());
}

#[test]
fn unclosed_image_is_an_error() {
    let error = parse_err(&document_stem_with("见 @image{assets/图1.png 图。"));

    assert_eq!(error.line(), 4);
    assert!(error.message().contains("@image"), "{}", error.message());
    assert!(error.message().contains('}'), "{}", error.message());
}

#[test]
fn empty_image_path_is_an_error() {
    let error = parse_err(&document_stem_with("见 @image{}。"));

    assert_eq!(error.line(), 4);
    assert!(error.message().contains("@image"), "{}", error.message());
    assert!(error.message().contains("路径"), "{}", error.message());
}

#[test]
fn image_without_braces_is_an_error() {
    let error = parse_err(&document_stem_with("见 @image assets/图1.png。"));

    assert_eq!(error.line(), 4);
    assert!(error.message().contains("@image"), "{}", error.message());
    assert!(error.message().contains('{'), "{}", error.message());
}

#[test]
fn blank_takes_no_value() {
    let error = parse_err(&document_stem_with("在 @blank{} 处填数。"));

    assert_eq!(error.line(), 4);
    assert!(error.message().contains("@blank"), "{}", error.message());
}

#[test]
fn unknown_inline_element_is_an_error() {
    let error = parse_err(&document_stem_with("见 @figure{图1.png}。"));

    assert_eq!(error.line(), 4);
    assert!(error.message().contains("@figure"), "{}", error.message());
    assert!(
        error.message().contains("@math"),
        "错误应列出可用的行内元素：{}",
        error.message()
    );
}

#[test]
fn block_keyword_in_a_content_line_is_an_error() {
    let source = "\
@module 资料分析

  @question
    @stem 题干。
      @answer B
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 5);
    assert!(error.message().contains("@answer"), "{}", error.message());
    assert!(error.message().contains("内容行"), "{}", error.message());
}

#[test]
fn material_without_content_is_an_error() {
    let source = "\
@module 资料分析

  @material

  @question
    @stem 小题。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 3);
    assert!(error.message().contains("材料"), "{}", error.message());
}

#[test]
fn material_without_questions_is_an_error() {
    let source = "\
@module 资料分析

  @material
    根据以下资料，回答下列问题。
";

    let error = parse_err(source);

    assert_eq!(error.line(), 3);
    assert!(error.message().contains("材料"), "{}", error.message());
    assert!(error.message().contains("一道"), "{}", error.message());
}

#[test]
fn material_takes_no_value() {
    let source = "\
@module 资料分析

  @material 材料一

  @question
    @stem 小题。
    @option A 甲
    @option B 乙
    @option C 丙
    @option D 丁
    @answer A
";

    let error = parse_err(source);

    assert_eq!(error.line(), 3);
    assert!(error.message().contains("@material"), "{}", error.message());
}

#[test]
fn material_outside_a_section_is_an_error() {
    let source = "\
@material
  材料一。

  @question
    @stem 小题。
";

    let error = parse_err(source);

    assert_eq!(error.line(), 1);
    assert!(error.message().contains("分节"), "{}", error.message());
}

/// 一份带占位题干的最小文档：把 `@stem` 的正文交给调用方，用于行内元素的用例。
fn document_stem_with(stem: &str) -> String {
    format!(
        "@module 数量关系\n\n  @question\n    @stem {stem}\n    @option A 甲\n    @option B 乙\n    @option C 丙\n    @option D 丁\n    @answer A\n"
    )
}
