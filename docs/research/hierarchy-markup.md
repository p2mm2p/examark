# 层级表达法：既有做法与失败经验

本文件回答：格式如何表达层级，尤其**缩进敏感格式在「人手写、内容是长散文、需要从别处粘贴」场景下的真实表现**。这是评估「全嵌套缩进」提案的依据。

- 调研时间：2026-09-12｜方法：联网检索规范原文、手册与实践事故帖，逐条附出处
- 标注：【证】= 查证到的公开资料；【推】= 调研者推断

## 1 YAML 的缩进语义规范

- 【证】规范 6.1：缩进只能用空格，"To maintain portability, tab characters must not be used in indentation"。[yaml.org/spec/1.2.2#61](https://yaml.org/spec/1.2.2/#61-indentation-spaces)
- 【证】自由文本用块标量：`|` 保留换行、`>` 把单个换行折成空格，还有 chomping（`-`/`+`）与缩进指示符（`|2`）；无指示符时「内容缩进＝首个非空行的前导空格数」。[yaml.org/spec/1.2.2#81](https://yaml.org/spec/1.2.2/#81-block-scalar-styles)
- 【证】规范**没有错误处理章节**；错位常硬报错（`expected block end`、`mapping values are not allowed here`）。[stringto.com](https://stringto.com/guides/common-yaml-syntax-errors/)
- 【推】但「整体多缩进一层」仍然合法——规范只判良构性，于是结构可以被静默改掉。

## 2 缩进敏感格式的真实事故

- 【证】Makefile：recipe 行必须以 TAB 开头，否则 `missing separator`；编辑器把 tab 自动换成空格即中招（有 "VS Code quietly started indenting with spaces" 的实例）。[GNU Make 手册](https://www.gnu.org/software/make/manual/html_node/Recipe-Syntax.html)｜[SO](https://stackoverflow.com/questions/16931770)
- 【证】Python 混用 tab 与空格 → `TabError`。[PEP 8](https://peps.python.org/pep-0008/#tabs-or-spaces)
- 【证】reStructuredText 的 directive 体缩进规则被反复提问；docutils 报 `Content block expected for the "raw" directive; none found`。[SO](https://stackoverflow.com/questions/79814368)
- 【证】**复制粘贴导致静默错解**：把带缩进的 HTML 粘进 Markdown，整块变成缩进代码块（CommonMark 例 191）；列表用 1 个空格 vs 2 个空格 → 两棵不同的树（例 294/295）。CommonMark 规范自陈 Markdown 里没有「语法错误」这回事，分歧常常很久后才被发现。[CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/#indented-code-blocks)
- 【证】YAML 自身的静默陷阱：`>` 与 `|` 混用会把换行变空格；未加引号的散文里出现 ` #` 会被截断成注释。[yamlchecker](https://yamlchecker.com/guides/yaml-multiline-string)｜[yaml.info](https://www.yaml.info/learn/bestpractices.html)

## 3 隐式结束格式的歧义

- 【证】Markdown 里 `Foo/bar/---/baz` 官方承认有四种解释。[CommonMark 0.31.2#setext](https://spec.commonmark.org/0.31.2/#setext-headings)
- 【证】TOML 的 `[table]` 隐式结束、点键与父表冲突需要专门澄清，深层嵌套是公认弱项。[toml-lang/toml#631](https://github.com/toml-lang/toml/issues/631)
- 【证】INI 无统一标准；configparser 的 `DEFAULT` 段会「魔法式」注入所有 section。[Python 文档](https://docs.python.org/3/library/configparser.html)

## 4 显式闭合的噪声成本

- 【证】LaTeX 的报错最清晰且带行号：`\begin{center} on input line 5 ended by \end{flushleft}`。[tex64.com](https://tex64.com/learn/document-basics/environments)
- 【证】XML 的噪声有真实抱怨（"angle bracket tax"）。[Coding Horror](https://blog.codinghorror.com/xml-the-angle-bracket-tax/)
- 【证】AsciiDoc 用成对分隔符界定块，同类嵌套要靠加长分隔符。[AsciiDoc 文档](https://docs.asciidoctor.org/asciidoc/latest/blocks/delimited/)
- 【证】Fountain 明说取舍：「拿不准就当 Action」「错误防御是不跨过双换行去找闭合符」。[fountain.io](https://fountain.io/syntax/)

## 5 重复符号表深度

- 【证】Markdown 只有 6 级；写出 7 个以上 `#` 时「fails silently」，作者自己都读不回来。[strictdoc#3046](https://github.com/strictdoc-project/strictdoc/issues/3046)
- 【证】Org-mode 星号数＝层级，官方手册承认「Some people find the many stars too noisy」。[Org 手册](https://orgmode.org/manual/Headlines.html)

## 6 层级由关键字命名表达

- 【证】reStructuredText 的 directive（`.. note::`）、Sphinx 域指令、Pandoc 的 fenced div（`:::`）都是「命名 + 显式界定或缩进体」。[Sphinx 文档](https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html)｜[Pandoc 手册](https://pandoc.org/MANUAL.html#fenced-divs)
- 【证】代价：rST 的 directive 体仍要缩进加空行，换来语义自解释。
- 【证】Moodle GIFT 走显式闭包：题干 + `{=对 ~错}`。

## 7 长散文采用层级表达的真实案例

- 【证】**正面**：Fountain（剧本格式）由关键字、大写与空行驱动，**缩进被显式忽略**（只在 Action 里保留），明确是为长文本与粘贴设计的。[fountain.io](https://fountain.io/syntax/)
- 【证】**负面**：SRT 字幕以空行分块、正文内不得有空行，行业工具把「缺空行/多空行」列为最常见的文件损坏。[subtitling.net](https://subtitling.net/guides/srt-file-errors)
- 【推】试卷类格式（Aiken、GIFT、LaTeX `exam`）几乎都用关键字 + 显式闭包，而不是缩进。

## 适配度评估

- **缩进（YAML 式）**：证据最不利。tab 禁用、块标量缩进由首行决定、`|` 与 `>` 会静默改语义、粘贴带缩进的内容即改变结构；YAML 恰是「错位常静默改结构」，与「必须严格报错」直接冲突。若采用，须自造校验层（禁 tab、禁跳级、内容缩进必须等于父级加固定宽度），而这些 YAML 本身并不保证。
- **显式闭合**：报错最清晰（错配能指到行号），对 200~400 字散文的噪声成本真实但可控，粘贴最安全。
- **重复符号深度**：上限 6 级、超出静默失败；`#`/`*` 在中文散文里易与正文相撞【推】。
- **关键字命名层级**：证据最有利——Fountain、GIFT、rST directive 都在「人手写、长文本、需粘贴」场景成立。做法是把「谁是材料/题干/选项」写进关键字，而不是让解析器猜缩进；剩下的风险只在「裸散文从哪里开始、到哪里结束」，需要显式起止或用不会与散文混淆的启动规则（Fountain 用双换行，AsciiDoc 用成对分隔符）。
