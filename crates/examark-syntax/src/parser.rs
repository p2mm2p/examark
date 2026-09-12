//! 题目文档的解析器。
//!
//! 语法（v1）：
//!
//! ```text
//! @title 题名              ← 元数据（缩进 0，键封闭）
//! @module 模块名            ← 模块分节（缩进 0）
//!   @subcategory 子分类      ← 可选，必须在题目之前
//!   @question              ← 题目（缩进 2）
//!     @stem 题干首行         ← 字段（缩进 4）
//!       题干续行             ← 内容行（缩进 6）
//!     @option A 选项
//!     @answer B
//!     @explanation 解析
//! ```
//!
//! 缩进是结构的一部分：每层 2 个空格，内容行必须比它的关键字深一级，回退必须命中已有的层级列。

use crate::ast::{Choice, Document, Metadata, Question, Section};
use crate::error::ParseError;

/// 每层缩进的空格数。
const INDENT: usize = 2;
/// 字段相对题目的缩进。
const FIELD_INDENT: usize = 2 * INDENT;

/// 解析题目文档源文本。
pub fn parse(source: &str) -> Result<Document, ParseError> {
    let mut parser = Parser {
        lines: preprocess(source)?,
        index: 0,
        next_number: 1,
    };

    parser.parse_document()
}

/// 预处理后的一行：行号、缩进宽度与去掉了缩进与行尾空白的正文。
#[derive(Clone, Copy)]
struct Line<'a> {
    number: usize,
    indent: usize,
    content: &'a str,
}

/// 切分源文本：剥掉 BOM，逐行算出缩进，并挡住 TAB 与非 2 的倍数的缩进。
///
/// 空白行不参与层级判定——它既不是结构行也不是内容行，行尾的空格与 TAB 不该判错。
fn preprocess(source: &str) -> Result<Vec<Line<'_>>, ParseError> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let mut lines = Vec::new();

    for (index, raw) in source.lines().enumerate() {
        let number = index + 1;

        if raw.trim().is_empty() {
            lines.push(Line {
                number,
                indent: 0,
                content: "",
            });
            continue;
        }

        let mut indent = 0;
        for character in raw.chars() {
            match character {
                ' ' => indent += 1,
                '\t' => {
                    return Err(ParseError::new(number, "缩进只能用空格，不能用 TAB"));
                }
                _ => break,
            }
        }

        if indent % INDENT != 0 {
            return Err(ParseError::new(
                number,
                format!("缩进必须是 {INDENT} 个空格的倍数（当前 {indent} 个空格）"),
            ));
        }

        lines.push(Line {
            number,
            indent,
            content: raw[indent..].trim_end(),
        });
    }

    Ok(lines)
}

struct Parser<'a> {
    lines: Vec<Line<'a>>,
    index: usize,
    next_number: usize,
}

impl<'a> Parser<'a> {
    fn parse_document(&mut self) -> Result<Document, ParseError> {
        if self.lines.iter().all(|line| line.content.is_empty()) {
            return Err(ParseError::new(1, "文档为空"));
        }

        let metadata = self.parse_metadata()?;
        let sections = self.parse_sections()?;

        if sections.is_empty() {
            return Err(ParseError::new(1, "文档不含任何模块分节"));
        }

        Ok(Document { metadata, sections })
    }

    fn line(&self) -> Option<Line<'a>> {
        self.lines.get(self.index).copied()
    }

    fn skip_blank(&mut self) {
        while self.line().is_some_and(|line| line.content.is_empty()) {
            self.index += 1;
        }
    }

    fn parse_metadata(&mut self) -> Result<Metadata, ParseError> {
        let mut metadata = Metadata::default();

        loop {
            self.skip_blank();
            let Some(line) = self.line() else {
                break;
            };
            if line.indent != 0 {
                break;
            }
            let Some(keyword) = keyword_of(line.content) else {
                break;
            };
            let Some(slot) = metadata.slot(keyword) else {
                break;
            };

            let value = value_of(line.content);
            if value.is_empty() {
                return Err(ParseError::new(
                    line.number,
                    format!("「@{keyword}」缺少取值"),
                ));
            }
            if slot.is_some() {
                return Err(ParseError::new(
                    line.number,
                    format!("元数据「@{keyword}」重复"),
                ));
            }

            *slot = Some(value.to_string());
            self.index += 1;
        }

        Ok(metadata)
    }

    fn parse_sections(&mut self) -> Result<Vec<Section>, ParseError> {
        let mut sections = Vec::new();

        loop {
            self.skip_blank();
            let Some(line) = self.line() else {
                break;
            };

            if line.indent != 0 {
                return Err(line_indent_error(line, 0));
            }
            let Some(keyword) = keyword_of(line.content) else {
                return Err(ParseError::new(
                    line.number,
                    "无法识别的行：结构行必须以 @ 开头，且顶格书写",
                ));
            };

            match keyword {
                "module" => {
                    let name = value_of(line.content);
                    if name.is_empty() {
                        return Err(ParseError::new(line.number, "模块分节缺少模块名"));
                    }

                    let header = line.number;
                    let name = name.to_string();
                    self.index += 1;
                    sections.push(self.parse_section(&name, header)?);
                }
                "subcategory" | "question" => {
                    return Err(ParseError::new(
                        line.number,
                        format!("「@{keyword}」必须位于模块分节之内，且缩进 {INDENT} 个空格"),
                    ));
                }
                keyword if Metadata::is_key(keyword) => {
                    return Err(ParseError::new(line.number, "元数据必须写在文档开头"));
                }
                keyword if is_known_keyword(keyword) => {
                    return Err(ParseError::new(
                        line.number,
                        format!("「@{keyword}」只能出现在题目块内，且缩进 {FIELD_INDENT} 个空格"),
                    ));
                }
                keyword => {
                    return Err(ParseError::new(
                        line.number,
                        format!("未知关键字「@{keyword}」"),
                    ));
                }
            }
        }

        Ok(sections)
    }

    fn parse_section(&mut self, module: &str, header: usize) -> Result<Section, ParseError> {
        let mut sub_category = None;
        let mut questions = Vec::new();

        loop {
            self.skip_blank();
            let Some(line) = self.line() else {
                break;
            };
            if line.indent == 0 {
                if keyword_of(line.content).is_none() {
                    return Err(ParseError::new(
                        line.number,
                        "无法识别的行：结构行必须以 @ 开头，且顶格书写",
                    ));
                }
                break;
            }

            let Some(keyword) = keyword_of(line.content) else {
                return Err(content_indent_error(line));
            };

            match keyword {
                "subcategory" if line.indent == INDENT => {
                    if !questions.is_empty() {
                        return Err(ParseError::new(
                            line.number,
                            "子分类必须用「@subcategory」写在题目之前",
                        ));
                    }
                    if sub_category.is_some() {
                        return Err(ParseError::new(line.number, "「@subcategory」重复"));
                    }

                    let name = value_of(line.content);
                    if name.is_empty() {
                        return Err(ParseError::new(line.number, "「@subcategory」缺少子分类名"));
                    }

                    sub_category = Some(name.to_string());
                    self.index += 1;
                }
                "question" if line.indent == INDENT => {
                    questions.push(self.parse_question(line)?);
                }
                keyword if is_known_keyword(keyword) => {
                    return Err(ParseError::new(
                        line.number,
                        format!("「@{keyword}」只能出现在题目块内，且缩进 {FIELD_INDENT} 个空格"),
                    ));
                }
                keyword => {
                    return Err(ParseError::new(
                        line.number,
                        format!("未知关键字「@{keyword}」"),
                    ));
                }
            }
        }

        if questions.is_empty() {
            return Err(ParseError::new(
                header,
                format!("模块分节「{module}」不含题目"),
            ));
        }

        Ok(Section {
            module: module.to_string(),
            sub_category,
            questions,
        })
    }

    fn parse_question(&mut self, header: Line<'a>) -> Result<Question, ParseError> {
        let start = header.number;
        if !value_of(header.content).is_empty() {
            return Err(ParseError::new(start, "「@question」不接受取值"));
        }

        let number = self.next_number;
        self.next_number += 1;
        self.index += 1;

        let stem = self.parse_stem(start)?;
        let options = self.parse_options(start)?;
        let answer = self.parse_answer(start)?;
        let explanation = self.parse_explanation()?;

        Ok(Question {
            number,
            stem,
            options,
            answer,
            explanation,
        })
    }

    fn parse_stem(&mut self, start: usize) -> Result<String, ParseError> {
        let Some((line, keyword)) = self.next_field()? else {
            return Err(ParseError::new(start, "题目缺少题干"));
        };
        if keyword != "stem" {
            return Err(ParseError::new(
                line.number,
                format!("此处应为「@stem」，实际是「@{keyword}」"),
            ));
        }

        let (header, stem) = self.collect_content(line, "stem")?;
        if stem.is_empty() {
            return Err(ParseError::new(header, "题目缺少题干"));
        }

        Ok(stem)
    }

    fn parse_options(&mut self, start: usize) -> Result<[String; 4], ParseError> {
        let mut options: Vec<String> = Vec::with_capacity(4);

        for expected in ['A', 'B', 'C', 'D'] {
            let Some((line, keyword)) = self.next_field()? else {
                return Err(ParseError::new(start, format!("题目缺少选项 {expected}")));
            };

            match keyword {
                "option" => {
                    let value = value_of(line.content);
                    let mut parts = value.splitn(2, char::is_whitespace);
                    let letter = parts.next().unwrap_or_default();
                    let text = parts.next().unwrap_or_default().trim();

                    match choice(letter) {
                        Some(_) if letter.starts_with(expected) => {}
                        Some(_) => {
                            return Err(ParseError::new(
                                line.number,
                                format!(
                                    "期望选项 {expected}，实际为 {letter}；选项必须按 A、B、C、D 顺序各出现一次"
                                ),
                            ));
                        }
                        None => {
                            return Err(ParseError::new(
                                line.number,
                                format!("选项字母必须是 A、B、C、D 之一（当前「{letter}」）"),
                            ));
                        }
                    }

                    if text.is_empty() {
                        return Err(ParseError::new(
                            line.number,
                            format!("选项 {expected} 的内容为空"),
                        ));
                    }

                    options.push(text.to_string());
                    self.index += 1;
                }
                "answer" => {
                    return Err(ParseError::new(line.number, "答案必须位于选项之后"));
                }
                "explanation" => {
                    return Err(ParseError::new(line.number, "解析必须位于答案之后"));
                }
                keyword => {
                    return Err(ParseError::new(
                        line.number,
                        format!("此处应为「@option {expected}」，实际是「@{keyword}」"),
                    ));
                }
            }
        }

        options
            .try_into()
            .map_err(|_| ParseError::new(start, "题目缺少选项"))
    }

    fn parse_answer(&mut self, start: usize) -> Result<Choice, ParseError> {
        let Some((line, keyword)) = self.next_field()? else {
            return Err(ParseError::new(start, "题目缺少答案"));
        };

        match keyword {
            "answer" => {
                let value = value_of(line.content);
                let Some(answer) = choice(value) else {
                    return Err(ParseError::new(
                        line.number,
                        format!("答案必须是 A、B、C、D 之一（当前「{value}」）"),
                    ));
                };
                self.index += 1;
                Ok(answer)
            }
            "explanation" => Err(ParseError::new(line.number, "解析必须位于答案之后")),
            keyword => Err(ParseError::new(
                line.number,
                format!("此处应为「@answer」，实际是「@{keyword}」"),
            )),
        }
    }

    fn parse_explanation(&mut self) -> Result<Option<String>, ParseError> {
        let Some((line, keyword)) = self.next_field()? else {
            return Ok(None);
        };

        match keyword {
            "explanation" => {
                let (header, explanation) = self.collect_content(line, "explanation")?;
                if explanation.is_empty() {
                    return Err(ParseError::new(header, "解析内容为空"));
                }
                Ok(Some(explanation))
            }
            "answer" => Err(ParseError::new(line.number, "题目中出现了多个答案")),
            keyword => Err(ParseError::new(
                line.number,
                format!("这里不能出现「@{keyword}」；题目块以「@explanation」收尾"),
            )),
        }
    }

    /// 题目块内的下一条字段行，以及它的关键字。
    ///
    /// 缩进比字段浅（或已到末尾）视为题目块结束，返回 `None`；正文行出现在结构位置、或缩进
    /// 比字段更深时报错——这两类都是护栏而不是静默跳过。
    fn next_field(&mut self) -> Result<Option<(Line<'a>, &'a str)>, ParseError> {
        self.skip_blank();
        let Some(line) = self.line() else {
            return Ok(None);
        };
        if line.indent < FIELD_INDENT {
            if keyword_of(line.content).is_some() {
                return Ok(None);
            }
            return Err(content_indent_error(line));
        }
        if line.indent > FIELD_INDENT {
            return Err(line_indent_error(line, FIELD_INDENT));
        }

        match keyword_of(line.content) {
            Some(keyword) => Ok(Some((line, keyword))),
            None => Err(content_indent_error(line)),
        }
    }

    /// 读取一条内容字段（`@stem`、`@explanation`）：关键字行内可带首行，续行须比它深一级。
    fn collect_content(
        &mut self,
        header: Line<'a>,
        keyword: &str,
    ) -> Result<(usize, String), ParseError> {
        let content_indent = header.indent + INDENT;
        let mut content = vec![value_of(header.content)];
        self.index += 1;

        while let Some(line) = self.line() {
            if line.content.is_empty() {
                content.push("");
                self.index += 1;
                continue;
            }
            if line.indent < content_indent {
                break;
            }
            if line.indent > content_indent {
                return Err(ParseError::new(
                    line.number,
                    format!("内容行必须比「@{keyword}」深一级（缩进 {content_indent} 个空格）"),
                ));
            }
            if let Some(inner) = keyword_of(line.content) {
                return Err(ParseError::new(
                    line.number,
                    format!("「@{inner}」不能出现在内容行里"),
                ));
            }

            content.push(line.content);
            self.index += 1;
        }

        Ok((header.number, trim_content(&content)))
    }
}

/// 识别行首关键字：`@xxx`。
fn keyword_of(content: &str) -> Option<&str> {
    content
        .strip_prefix('@')
        .map(|rest| rest.split(char::is_whitespace).next().unwrap_or_default())
}

/// 取关键字之后的取值（去掉分隔用的空白）。
fn value_of(content: &str) -> &str {
    match content.find(char::is_whitespace) {
        Some(index) => content[index..].trim_start(),
        None => "",
    }
}

fn is_known_keyword(keyword: &str) -> bool {
    Metadata::is_key(keyword)
        || matches!(
            keyword,
            "module" | "subcategory" | "question" | "stem" | "option" | "answer" | "explanation"
        )
}

fn choice(value: &str) -> Option<Choice> {
    match value {
        "A" => Some(Choice::A),
        "B" => Some(Choice::B),
        "C" => Some(Choice::C),
        "D" => Some(Choice::D),
        _ => None,
    }
}

/// 内容块：去掉首尾空行，逐行去掉行尾空白，内部空行保留。
fn trim_content(lines: &[&str]) -> String {
    let first = lines.iter().position(|line| !line.trim().is_empty());
    let last = lines.iter().rposition(|line| !line.trim().is_empty());

    match (first, last) {
        (Some(first), Some(last)) => lines[first..=last]
            .iter()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// 结构行的缩进不对：指名关键字与应有的缩进。
fn line_indent_error(line: Line<'_>, expected: usize) -> ParseError {
    match keyword_of(line.content) {
        Some(keyword) => ParseError::new(
            line.number,
            format!(
                "「@{keyword}」的缩进应为 {expected} 个空格（当前 {}）",
                line.indent
            ),
        ),
        None => content_indent_error(line),
    }
}

/// 内容行出现在不该出现的位置。
fn content_indent_error(line: Line<'_>) -> ParseError {
    ParseError::new(
        line.number,
        "内容行必须写在关键字之下，并比它的关键字深一级",
    )
}
