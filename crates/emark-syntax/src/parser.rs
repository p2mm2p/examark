//! 题目文档的解析器。
//!
//! 语法（v1）：
//!
//! ```text
//! @title 题名              ← 元数据（缩进 0，键封闭）
//! @module 模块名            ← 模块分节（缩进 0，模块封闭）
//!   @subcategory 子分类      ← 可选，必须在题目之前；须属于本节模块（子分类封闭）
//!   @question              ← 独立单选题（缩进 2）
//!     @stem 题干首行         ← 字段（缩进 4）
//!       题干续行             ← 内容行（缩进 6）
//!     @option A 选项
//!     @answer B
//!     @explanation 解析
//!   @material              ← 材料题：材料首行（缩进 2）
//!     材料续行              ← 内容行（缩进 4）
//!   @question              ← 紧随材料的题目都属于该材料
//! ```
//!
//! 缩进是结构的一部分：每层 2 个空格，内容行必须比它的关键字深一级，回退必须命中已有的层级列。
//! 正文里的行内元素是 `@math{…}`、`@image{…}` 与裸 token `@blank`；它们不跨行，不认识的一律报错。

use crate::ast::{
    Choice, Content, Document, Inline, MaterialQuestion, Metadata, Paragraph, Question, Section,
    SingleChoice,
};
use crate::category::{Module, SubCategory};
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
                    let Some(module) = Module::from_name(name) else {
                        return Err(ParseError::new(line.number, unknown_module_message(name)));
                    };

                    let header = line.number;
                    self.index += 1;
                    sections.push(self.parse_section(module, header)?);
                }
                "subcategory" | "question" | "material" => {
                    return Err(ParseError::new(
                        line.number,
                        format!("「@{keyword}」必须位于模块分节之内，且缩进 {INDENT} 个空格"),
                    ));
                }
                keyword if Metadata::is_key(keyword) => {
                    return Err(ParseError::new(line.number, "元数据必须写在文档开头"));
                }
                keyword if is_block_keyword(keyword) => {
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

    fn parse_section(&mut self, module: Module, header: usize) -> Result<Section, ParseError> {
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

                    match SubCategory::from_name(name) {
                        Some(declared) if declared.module() == module => {
                            sub_category = Some(declared);
                        }
                        Some(declared) => {
                            return Err(ParseError::new(
                                line.number,
                                format!(
                                    "子分类「{name}」属于「{}」，不能用在「{}」模块分节里",
                                    declared.module().name(),
                                    module.name()
                                ),
                            ));
                        }
                        None => {
                            return Err(ParseError::new(
                                line.number,
                                unknown_sub_category_message(module, name),
                            ));
                        }
                    }

                    self.index += 1;
                }
                "question" if line.indent == INDENT => {
                    let question = self.parse_single_choice(line)?;
                    questions.push(Question::Single(question));
                }
                "material" if line.indent == INDENT => {
                    let question = self.parse_material(line)?;
                    questions.push(Question::Material(question));
                }
                keyword if is_block_keyword(keyword) => {
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
                format!("模块分节「{}」不含题目", module.name()),
            ));
        }

        Ok(Section {
            module,
            sub_category,
            questions,
        })
    }

    fn parse_single_choice(&mut self, header: Line<'a>) -> Result<SingleChoice, ParseError> {
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

        Ok(SingleChoice {
            number,
            stem,
            options,
            answer,
            explanation,
        })
    }

    /// 材料题由「相邻绑定」构成：`@material` 的材料正文之后，紧跟的每一道 `@question`
    /// 都是它的小题，直到下一个 `@material`、子分类声明或分节结束。
    fn parse_material(&mut self, header: Line<'a>) -> Result<MaterialQuestion, ParseError> {
        let start = header.number;
        if !value_of(header.content).is_empty() {
            return Err(ParseError::new(start, "「@material」不接受取值"));
        }

        let lines = self.collect_content(header, "material")?;
        let material = content_of(&lines)?;
        if material.paragraphs.is_empty() {
            return Err(ParseError::new(start, "材料内容为空"));
        }

        let mut questions = Vec::new();
        loop {
            self.skip_blank();
            let Some(line) = self.line() else {
                break;
            };
            if line.indent != INDENT || keyword_of(line.content) != Some("question") {
                break;
            }
            questions.push(self.parse_single_choice(line)?);
        }

        if questions.is_empty() {
            return Err(ParseError::new(
                start,
                "材料之后必须紧跟至少一道「@question」小题",
            ));
        }

        Ok(MaterialQuestion {
            material,
            questions,
        })
    }

    fn parse_stem(&mut self, start: usize) -> Result<Content, ParseError> {
        let Some((line, keyword)) = self.next_field()? else {
            return Err(ParseError::new(start, "题目缺少题干"));
        };
        if keyword != "stem" {
            return Err(ParseError::new(
                line.number,
                format!("此处应为「@stem」，实际是「@{keyword}」"),
            ));
        }

        let lines = self.collect_content(line, "stem")?;
        let stem = content_of(&lines)?;
        if stem.paragraphs.is_empty() {
            return Err(ParseError::new(line.number, "题目缺少题干"));
        }

        Ok(stem)
    }

    fn parse_options(&mut self, start: usize) -> Result<[Content; 4], ParseError> {
        let mut options: Vec<Content> = Vec::with_capacity(4);

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

                    options.push(Content {
                        paragraphs: vec![Paragraph(inline_nodes(line, text)?)],
                    });
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

    fn parse_explanation(&mut self) -> Result<Option<Content>, ParseError> {
        let Some((line, keyword)) = self.next_field()? else {
            return Ok(None);
        };

        match keyword {
            "explanation" => {
                let lines = self.collect_content(line, "explanation")?;
                let explanation = content_of(&lines)?;
                if explanation.paragraphs.is_empty() {
                    return Err(ParseError::new(line.number, "解析内容为空"));
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

    /// 读取一条内容字段（`@stem`、`@explanation`、`@material`）：关键字行内可带首行，
    /// 续行须比它深一级。返回内容行本身（带行号，供行内元素报错），空行原样保留。
    fn collect_content(
        &mut self,
        header: Line<'a>,
        keyword: &str,
    ) -> Result<Vec<Line<'a>>, ParseError> {
        let content_indent = header.indent + INDENT;
        let mut lines = vec![Line {
            number: header.number,
            indent: content_indent,
            content: value_of(header.content),
        }];
        self.index += 1;

        while let Some(line) = self.line() {
            if line.content.is_empty() {
                lines.push(line);
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

            lines.push(line);
            self.index += 1;
        }

        Ok(lines)
    }
}

/// 识别行首关键字：`@xxx`，到空白或行内元素的 `{` 为止。
fn keyword_of(content: &str) -> Option<&str> {
    content.strip_prefix('@').map(|rest| {
        rest.split(|character: char| character.is_whitespace() || character == '{')
            .next()
            .unwrap_or_default()
    })
}

/// 取关键字之后的取值（去掉分隔用的空白）。
fn value_of(content: &str) -> &str {
    match content.find(char::is_whitespace) {
        Some(index) => content[index..].trim_start(),
        None => "",
    }
}

fn is_block_keyword(keyword: &str) -> bool {
    Metadata::is_key(keyword)
        || matches!(
            keyword,
            "module"
                | "subcategory"
                | "question"
                | "material"
                | "stem"
                | "option"
                | "answer"
                | "explanation"
        )
}

/// 关键字标识符的字符：ASCII 字母、数字与下划线。
fn is_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

/// 未知模块：点名，并列出 v1 的官方模块。
fn unknown_module_message(name: &str) -> String {
    let modules = Module::ALL
        .iter()
        .map(|module| module.name())
        .collect::<Vec<_>>()
        .join("、");

    format!("未知模块「{name}」；v1 的模块只有：{modules}")
}

/// 未知子分类：该模块有官方子分类时列出它们，没有时直说没有。
fn unknown_sub_category_message(module: Module, name: &str) -> String {
    let sub_categories = module
        .sub_categories()
        .iter()
        .map(|sub_category| sub_category.name())
        .collect::<Vec<_>>()
        .join("、");

    if sub_categories.is_empty() {
        format!("未知子分类「{name}」；「{}」没有子分类", module.name())
    } else {
        format!(
            "未知子分类「{name}」；「{}」的子分类只有：{sub_categories}",
            module.name()
        )
    }
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

/// 把内容行组成 `Content`：去掉首尾空行，空行分段，行内元素解析为节点。
fn content_of(lines: &[Line<'_>]) -> Result<Content, ParseError> {
    let first = lines.iter().position(|line| !line.content.is_empty());
    let last = lines.iter().rposition(|line| !line.content.is_empty());
    let (Some(first), Some(last)) = (first, last) else {
        return Ok(Content {
            paragraphs: Vec::new(),
        });
    };

    let mut paragraphs = Vec::new();
    let mut current: Vec<Inline> = Vec::new();

    for line in &lines[first..=last] {
        if line.content.is_empty() {
            if !current.is_empty() {
                paragraphs.push(Paragraph(std::mem::take(&mut current)));
            }
            continue;
        }
        if !current.is_empty() {
            current.push(Inline::Text("\n".to_string()));
        }
        current.extend(inline_nodes(*line, line.content)?);
    }
    if !current.is_empty() {
        paragraphs.push(Paragraph(current));
    }

    Ok(Content { paragraphs })
}

/// 解析一行正文里的行内元素：`@math{…}`、`@image{…}` 与裸 token `@blank`。
///
/// 行内元素的记号是「`@` + ASCII 字母开头的标识符」；其他位置上的 `@`（如 `@1`、`@张三`）
/// 按正文处理。不认识的标识符一律报错，绝不静默当正文；行内元素不跨行。
fn inline_nodes(line: Line<'_>, text: &str) -> Result<Vec<Inline>, ParseError> {
    let mut nodes = Vec::new();
    let mut rest = text;

    while let Some(index) = rest.find('@') {
        if index > 0 {
            nodes.push(Inline::Text(rest[..index].to_string()));
        }
        rest = &rest[index + 1..];

        let end = rest
            .find(|character: char| !is_identifier_char(character))
            .unwrap_or(rest.len());
        let identifier = &rest[..end];
        if !identifier.starts_with(|character: char| character.is_ascii_alphabetic()) {
            nodes.push(Inline::Text("@".to_string()));
            continue;
        }
        rest = &rest[end..];

        match identifier {
            "math" => {
                let Some(body) = rest.strip_prefix('{') else {
                    return Err(ParseError::new(
                        line.number,
                        "「@math」必须紧跟「{」，写成 @math{公式}",
                    ));
                };
                let (source, tail) = math_body(line, body)?;
                if source.is_empty() {
                    return Err(ParseError::new(line.number, "「@math」的公式内容为空"));
                }
                nodes.push(Inline::Math(source));
                rest = tail;
            }
            "image" => {
                let Some(body) = rest.strip_prefix('{') else {
                    return Err(ParseError::new(
                        line.number,
                        "「@image」必须紧跟「{」，写成 @image{相对路径}",
                    ));
                };
                let Some(closing) = body.find('}') else {
                    return Err(ParseError::new(
                        line.number,
                        "「@image」缺少闭合的「}」；行内元素不能跨行",
                    ));
                };
                let path = body[..closing].trim();
                if path.is_empty() {
                    return Err(ParseError::new(line.number, "「@image」缺少图片路径"));
                }
                nodes.push(Inline::Image(path.to_string()));
                rest = &body[closing + 1..];
            }
            "blank" => {
                if rest.starts_with('{') {
                    return Err(ParseError::new(
                        line.number,
                        "「@blank」不接受取值，直接写 @blank",
                    ));
                }
                nodes.push(Inline::Blank);
            }
            identifier if is_block_keyword(identifier) => {
                return Err(ParseError::new(
                    line.number,
                    format!("「@{identifier}」不能出现在内容行里"),
                ));
            }
            identifier => {
                return Err(ParseError::new(
                    line.number,
                    format!(
                        "未知行内元素「@{identifier}」；行内元素只有：@math{{公式}}、@image{{相对路径}}、@blank"
                    ),
                ));
            }
        }
    }

    if !rest.is_empty() {
        nodes.push(Inline::Text(rest.to_string()));
    }

    Ok(nodes)
}

/// 取 `@math{…}` 的公式源码：花括号按 LaTeX 语法配平，`\{`、`\}` 是转义不参与计数。
fn math_body<'a>(line: Line<'_>, body: &'a str) -> Result<(String, &'a str), ParseError> {
    let mut depth = 1usize;
    let mut characters = body.char_indices();

    while let Some((index, character)) = characters.next() {
        match character {
            '\\' => {
                characters.next();
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((body[..index].to_string(), &body[index + 1..]));
                }
            }
            _ => {}
        }
    }

    Err(ParseError::new(
        line.number,
        "「@math」缺少闭合的「}」；行内元素不能跨行",
    ))
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
