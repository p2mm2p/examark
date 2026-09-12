# 既有题目格式的取舍与中文元数据先例

本文件回答两个问题：别的题目格式在「题干/选项/答案/解析/材料题/图片/公式」上踩过什么坑；中文写作场景里文档级元数据的键该用什么语言。

- 调研时间：2026-09-12｜方法：联网检索各格式官方文档与实测文章，逐条附出处
- 标注：【证】= 查证到的公开资料；【推】= 调研者推断

## 1 既有题目标记格式的题目模型

- **Aiken（Moodle 最简）**：`1.题干` … `A. 选项` … `ANSWER: B`。官方事实：题干必须在一行；`ANSWER:` 必须全大写＋英文冒号＋空格；题块间恰好一个空行；仅单选，无解析/图片/富文本/多正确项。官方自述「GIFT 更少出错，但不如 Aiken 直白」。[docs.moodle.org/en/Aiken_Format](https://docs.moodle.org/en/Aiken_Format)、[moodlebuilder](https://moodlebuilder.com/en/guides/aiken-format-quiz-guide)
- **GIFT（Moodle 富格式）**：`::标题::题干 {=正确 ~干扰}`，另有 `#反馈`、`%50%` 权重、`$CATEGORY:`、`//注释`、匹配 `=左->右`、数值 `{#100:1}`。事实：`{ } = ~ #` 落进正文必须转义，否则结构错乱；块以空行分隔。[docs.moodle.org/en/GIFT_format](https://grokipedia.com/page/gift_file_format)
- **QTI 3.0**：`qti-assessment-item` ＝ `response-declaration`（答案）＋ `item-body`（呈现）＋ `modal-feedback`（解析），结构上答案与正文分离；公式用 MathML，图片用 XHTML。**关键事实：3.0 专设 `qti-assessment-stimulus` ＋ `stimulus-ref`，用「独立材料对象被多条 item 引用」表示材料题。**[imsglobal.org/spec/qti/v3p0/info](https://www.imsglobal.org/spec/qti/v3p0/info/)
- **Anki**：每行一条、字段用 Tab/逗号/分号；文档级元数据用顶部 `#separator/#notetype/#deck/#tags` 英文短键；媒体 `<img>`/`[sound:]` 且文件须在媒体目录；多行字段用引号或 `<br>`，cloze 跨行时引号转义失效。[docs.ankiweb.net](https://docs.ankiweb.net/importing/text-files.html)
- **LaTeX `exam` 文档类**：`questions/parts/subparts` 嵌套环境 ＋ `choices/checkboxes`，`\question[分值]`，`answers` 全局开关印解答。事实：该类不支持英语以外语言，默认词须手译。[overleaf.com](https://www.overleaf.com/learn/latex/Typesetting_exams_in_LaTeX)
- **Markdown 生态**：同名 Examark（`1. [MC] 题干 [2pts]`、`a) …`、`[x]` 标正确、`# Section:` 分组）——**事实：它弃用了「用 `**粗体**` 标正确项」的写法，理由是「与 LaTeX 冲突、预览泄露答案」；弃用 `*` 前缀是因撞列表。**[data-wise.github.io/examark](https://data-wise.github.io/examark/markdown/syntax/)。MkDocs Quiz 用 `<quiz>`＋`- [x]`，内容区全量 Markdown。[ewels.github.io/mkdocs-quiz](https://ewels.github.io/mkdocs-quiz/advanced-formatting/)
- **Quizlet / 中文题库**：Quizlet 术语|定义用制表/逗号/破折号。百分考（中文 SaaS，最贴近本题场景）：结构化文本 `1. 题干`/`A. 选项`/`答案:A`/`解析:`/`难度:`/`---` 分隔；多选写连续字母「ABC」并**明确禁用逗号顿号**；「自动清理全角字母与不可见字符」。**关键事实：「组合题」（材料＋多小题）不支持批量导入、必须手建；图片/公式/表格一律「先导文字再补」。**[docs.baifenkao.com](https://docs.baifenkao.com/admin/question-import.html)

## 2 中文场景的文档级元数据惯例

【证】

- Hexo、Hugo 的中文文档里 front-matter 键**一律英文**（title/date/tags/categories/lang）。[hexo.io/zh-cn](https://hexo.io/zh-cn/docs/front-matter)、[hugo.opendocs.io](https://hugo.opendocs.io/content-management/front-matter/)
- Obsidian 实践帖的结论明确：「代码层坚持只用英文属性名，正文用别名显示中文」，并列中文键的翻车点——**Dataview 把 `status` 与「状态」当成两个不同字段、无法合并**；中文键在查询里「特殊符号和空格容易引发报错」。[muyu-org.github.io](https://muyu-org.github.io/post/yaml-faq/)
- 另有 Hugo front matter 非 ASCII 值渲染异常、kebab-case 键在模板取不到等问题。[discourse.gohugo.io](https://discourse.gohugo.io/t/rendering-of-unicode-characters-in-front-matter/41317)

【推】YAML 语法本身允许中文键，问题全在下游模板/查询/导出。**元数据键用英文是事实标准，不算自相矛盾**；要中文观感就做显示层/别名映射，别改键名。

## 3 中文标点作为结构符的解析成本

【证】强证据：

- CommonMark 至今未解决「CJK 全角标点紧挨强调定界符」——`**テスト。**テスト` 不渲染为粗体；[issue #650](https://github.com/commonmark/commonmark-spec/issues/650) 于 2020 年提出且仍 open。
- 实测帖：`，。、；：？！""''（）【】《》—～…·` 等几乎全部中文标点在 `**…**` 内失败，80 万字小说中出现 2700+ 次，只能加空格/零宽字符/把标点移出定界符补救。[blog.rxliuli.com](https://blog.rxliuli.com/p/2029b35ae4094a48a3073f998f10af9c/)

【推】全角冒号 `：` 只作「整行标签结尾」是安全的；把 `、`/`（）`/`：` 当**行内**结构符风险高，会撞上 CJK 无空格习惯。

## 4 对本设计最相关的教训

1. **行内符号做结构最易翻车**：Aiken 的 `ANSWER:` 会被正文误判、GIFT 的 `= ~ { } #` 要转义、Markdown 版 Examark 弃用 `**bold**`（撞 LaTeX）。行首**整行前缀标签**是最耐久的形态——不依赖行内定界符，天生避开 CJK 标点冲突。
2. **元数据键用英文不矛盾**：Hexo/Hugo/Obsidian 的事实标准都是英文键，中文键只在下游（Dataview/模板）翻车。
3. **材料题要独立对象**：纯行扫格式（GIFT/Aiken）没有材料题；要「材料＋多小题」就学 QTI 的 stimulus 或 LaTeX 嵌套环境。中文 SaaS 的教训是反面教材——组合题直接不支持导入。
4. **媒体与公式别混进行文本**：Aiken 不支持；Anki/百分考都「先纯文字后补媒体」。公式给明确内联记号（`$…$`）。
5. **全角标点只做「标签冒号」**：用 `、`/`（）` 分隔或分组，等于把下游解析器的已知坑搬进自己的格式。
