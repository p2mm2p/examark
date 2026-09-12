//! `examark-syntax` 解析接缝的测试：字符串进，AST 出。
//!
//! 语法：`@module` 顶格，其下每深一级缩进 2 个空格；内容行比它的关键字深一级。

use examark_syntax::{Choice, Document, ParseError, parse};

fn parse_ok(source: &str) -> Document {
    parse(source).expect("文档应解析成功")
}

fn parse_err(source: &str) -> ParseError {
    parse(source).expect_err("文档应解析失败")
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
    assert_eq!(section.module, "言语理解与表达");
    assert_eq!(section.sub_category.as_deref(), Some("逻辑填空"));
    assert_eq!(section.questions.len(), 1);

    let question = &section.questions[0];
    assert_eq!(question.number, 1);
    assert_eq!(
        question.stem,
        "填入划横线部分最恰当的一项是：\n他山之石，可以攻玉。两者差异。"
    );
    assert_eq!(
        question.options,
        ["截然不同", "大相径庭", "迥然不同", "天壤之别"]
    );
    assert_eq!(question.answer, Choice::B);
    assert_eq!(
        question.explanation.as_deref(),
        Some("上下文强调差距悬殊，故选 B。")
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
    assert_eq!(document.sections[0].questions[0].explanation, None);
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
        .flat_map(|section| section.questions.iter().map(|question| question.number))
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
        document.sections[0].questions[0].stem,
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

        assert_eq!(document.sections[0].questions[0].answer, choice);
    }
}

#[test]
fn accepts_windows_line_endings() {
    let source = "@module 资料分析\r\n\r\n  @question\r\n    @stem 题干。\r\n    @option A 甲\r\n    @option B 乙\r\n    @option C 丙\r\n    @option D 丁\r\n    @answer B\r\n";

    let document = parse_ok(source);

    assert_eq!(document.sections[0].questions[0].answer, Choice::B);
}

#[test]
fn accepts_a_utf8_bom() {
    let source = "\u{feff}@module 资料分析\n\n  @question\n    @stem 题干。\n    @option A 甲\n    @option B 乙\n    @option C 丙\n    @option D 丁\n    @answer A\n";

    let document = parse_ok(source);

    assert_eq!(document.sections[0].module, "资料分析");
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

    let question = &document.sections[0].questions[0];
    assert_eq!(question.options, ["甲", "乙", "丙", "丁"]);
    assert_eq!(question.explanation.as_deref(), Some("解析。"));
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
