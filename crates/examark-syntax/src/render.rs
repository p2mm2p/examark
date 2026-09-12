//! 从 AST 渲染为完整的独立 HTML 文档。
//!
//! 渲染是纯 AST→HTML 的函数：它只读 AST，不接触源文本。

use crate::ast::{Document, Metadata, Question, Section};

/// 无题名文档的兜底题名。
const DEFAULT_TITLE: &str = "题目文档";

/// 缩进宽度。
const INDENT: &str = "  ";

/// 选项字母，按 A、B、C、D 顺序。
const LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];

/// 最小样式表：只让输出在浏览器里可读。
const STYLE: &str = "\
body { margin: 0 auto; max-width: 40rem; padding: 2rem 1rem; font-family: system-ui, sans-serif; line-height: 1.8; color: #222; }
h1 { font-size: 1.4rem; }
h2 { font-size: 1.15rem; margin-top: 2.5rem; }
h3 { font-size: 1rem; color: #666; }
.paper-meta { color: #666; font-size: .9rem; }
.question { margin: 1.5rem 0; }
.options { list-style: none; padding: 0; }
.options li { margin: .25rem 0; }
.answer { font-weight: 600; }
.explanation { color: #444; }";

/// 把题目文档渲染为完整的独立 HTML 文档。
pub fn render(document: &Document) -> String {
    let mut html = Html::new();

    html.line("<!DOCTYPE html>");
    html.open("<html lang=\"zh-CN\">");
    html.open("<head>");
    html.line("<meta charset=\"utf-8\">");
    html.line("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    html.line(&format!("<title>{}</title>", escape(title(document))));
    html.open("<style>");
    for rule in STYLE.lines() {
        html.line(rule);
    }
    html.close("</style>");
    html.close("</head>");
    html.open("<body>");
    html.open("<article class=\"paper\">");
    render_header(&mut html, document);
    for section in &document.sections {
        render_section(&mut html, section);
    }
    html.close("</article>");
    html.close("</body>");
    html.close("</html>");

    html.finish()
}

fn render_header(html: &mut Html, document: &Document) {
    html.open("<header class=\"paper-header\">");
    html.line(&format!("<h1>{}</h1>", escape(title(document))));

    let meta = meta_line(&document.metadata);
    if !meta.is_empty() {
        html.line(&format!("<p class=\"paper-meta\">{meta}</p>"));
    }

    html.close("</header>");
}

fn render_section(html: &mut Html, section: &Section) {
    html.open("<section class=\"module\">");
    html.line(&format!("<h2>{}</h2>", escape(section.module.name())));

    if let Some(sub_category) = section.sub_category {
        html.line(&format!(
            "<h3 class=\"sub-category\">{}</h3>",
            escape(sub_category.name())
        ));
    }

    for question in &section.questions {
        render_question(html, question);
    }

    html.close("</section>");
}

fn render_question(html: &mut Html, question: &Question) {
    html.open("<article class=\"question\">");
    render_stem(html, question);

    html.open("<ul class=\"options\">");
    for (letter, option) in LETTERS.iter().zip(&question.options) {
        html.line(&format!(
            "<li><span class=\"option-letter\">{letter}.</span> {}</li>",
            escape(option)
        ));
    }
    html.close("</ul>");

    html.line(&format!(
        "<p class=\"answer\">答案：{}</p>",
        question.answer.letter()
    ));

    if let Some(explanation) = &question.explanation {
        render_explanation(html, explanation);
    }

    html.close("</article>");
}

fn render_stem(html: &mut Html, question: &Question) {
    let number = format!(
        "<span class=\"question-number\">{}.</span> ",
        question.number
    );

    for (index, paragraph) in paragraphs(&question.stem).iter().enumerate() {
        let number = if index == 0 { number.as_str() } else { "" };
        html.line(&format!(
            "<p class=\"stem\">{number}{}</p>",
            paragraph_html(paragraph)
        ));
    }
}

fn render_explanation(html: &mut Html, explanation: &str) {
    html.open("<div class=\"explanation\">");

    for (index, paragraph) in paragraphs(explanation).iter().enumerate() {
        let label = if index == 0 {
            "<span class=\"label\">解析：</span>"
        } else {
            ""
        };
        html.line(&format!("<p>{label}{}</p>", paragraph_html(paragraph)));
    }

    html.close("</div>");
}

/// 卷首元数据行：考试、年份、卷别、地区、出处按字段顺序用 ` · ` 连接；一个值都没有时为空串。
///
/// 参考时限与总分不出现在卷首行——卷面不印分值，它们留待需要时另行呈现。
fn meta_line(metadata: &Metadata) -> String {
    let fields = [
        &metadata.exam,
        &metadata.year,
        &metadata.paper,
        &metadata.region,
        &metadata.source,
    ];

    fields
        .iter()
        .filter_map(|field| field.as_deref())
        .map(escape)
        .collect::<Vec<_>>()
        .join(" · ")
}

/// 把一段正文切成段落：空行分段，段内换行保留。
fn paragraphs(text: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join("\n"));
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        paragraphs.push(current.join("\n"));
    }

    paragraphs
}

/// 段落在 HTML 里的样子：先转义，再把段内换行照写成 `<br>`。
fn paragraph_html(paragraph: &str) -> String {
    escape(paragraph).replace('\n', "<br>")
}

fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());

    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(character),
        }
    }

    escaped
}

fn title(document: &Document) -> &str {
    document.metadata.title.as_deref().unwrap_or(DEFAULT_TITLE)
}

/// 逐行写 HTML 的缓冲区；`depth` 决定每行行首的缩进。
struct Html {
    output: String,
    depth: usize,
}

impl Html {
    fn new() -> Self {
        Self {
            output: String::new(),
            depth: 0,
        }
    }

    fn line(&mut self, markup: &str) {
        for _ in 0..self.depth {
            self.output.push_str(INDENT);
        }
        self.output.push_str(markup);
        self.output.push('\n');
    }

    /// 写开标签，并把后续行加深一级。
    fn open(&mut self, markup: &str) {
        self.line(markup);
        self.depth += 1;
    }

    /// 先退回一级，再写闭标签。
    fn close(&mut self, markup: &str) {
        self.depth -= 1;
        self.line(markup);
    }

    fn finish(self) -> String {
        self.output
    }
}
