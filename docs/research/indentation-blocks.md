# 缩进语法的语言如何界定块边界

本文件裁决一个问题：**既然 Python、Haskell、YAML 都不要求闭合关键字，我们为什么需要？** 背景是我们为中文行测题目文档设计格式，内容是长散文、需要从别处粘贴、解析必须严格报错。

- 调研时间：2026-09-12｜方法：联网检索语言规范、官方手册与实践抱怨，逐条附出处
- 标注：【证】= 查证到的公开资料；【推】= 调研者推断

## 1 谁用缩进定界，是否需要闭合关键字

| 格式 | 开块标志 | 块结束方式 | 闭合关键字 |
|---|---|---|---|
| Python | 行尾 `:` | DEDENT（缩进回退）+ 文件尾补 DEDENT | 无 |
| Haskell | `let/where/do/of` 后隐式 `{n}` | 缩进回退，或 `parse-error(t)` 触发隐式 `}` | 无（可用显式 `{}`） |
| YAML | 键后换行加深 | 缩进回退 / EOF | 无 |
| F# | `=`/`then`/`->` 之后的 offside 列 | 编译器插入 `$end`/`$done` | 无 |
| CoffeeScript | 行尾 | 反缩进 | 无 |
| Pug / Stylus / Sass(indented) / ABC | 行尾 | 反缩进 | 无 |
| Makefile | 行首**真 TAB** | 下一非 TAB 行 / 下一 target | 无 |
| Markdown | 4 空格或 1 TAB | 反缩进 | 无（只对代码块与列表续行有语义） |
| reStructuredText | 标记 + 缩进体 | 反缩进 | 无（用空注释 `..` 兜底） |
| Nix | `{}` + 分号 | 显式 `}` | **不是**缩进语言 |

【证】[Off-side rule](https://en.wikipedia.org/wiki/Off-side_rule)、[Haskell 2010 §10.3](https://www.haskell.org/onlinereport/haskell2010/haskellch10.html)、[Python 词法分析](https://docs.python.org/3/reference/lexical_analysis.html)、[F# 规范 §15](https://fsharp.github.io/fslang-spec/lexical-filtering/)、[Sass 语法](https://sass-lang.com/documentation/syntax/)。

**事实**：质疑者说对了一半——这些语言确实不强制闭合关键字。但**没有一个靠裸缩进**：它们都依赖 (a) 显式开块标记（Python 的 `:`、Haskell/F# 的关键字）＋ (b) DEDENT 栈比对。结束靠「下一行缩进变浅」，不是靠自然结束。

## 2 多行自由文本：它们都另造了定界机制

- 【证】Python 三引号 `"""…"""`（跨行保留换行）；YAML 块标量 `|`/`>` 与 chomping；CoffeeScript 的 `"""` 块字符串与 `///` heregex；Haskell 的 `\` gap——报告明确说字符串可跨行，故 layout 不为其插入 `<n>`；RST 的 `::` 字面块与 directive 体。
- 出处：[Python 词法分析](https://docs.python.org/3/reference/lexical_analysis.html)、[YAML 多行字符串](https://yamllint.yaml-multiline-strings/)、[CoffeeScript heregex](https://coffeescript-cookbook.github.io/chapters/regular_expressions/heregexes)、[Haskell 2010 §10.3](https://www.haskell.org/onlinereport/haskell2010/haskellch10.html)。
- 【推】缩进语言一致地把「散文/长字符串」挪出 layout 管辖、改用独立定界符——**缩进不适合承载长文本，它们自己都绕开了**。

## 3 缩进写错：报错还是静默改结构

- **报错**：【证】Python 抛 `IndentationError: expected an indented block`、`unexpected indent`、`unindent does not match any outer indentation level`，混用 TAB/空格抛 `TabError`；F# 报 FS0058「possible incorrect indentation」；CoffeeScript 空白不一致直接编译失败。
- **合法但语义变了（静默）**：【证】Haskell 报告自认「Some error conditions are not detected by the algorithm」；YAML 把多余缩进读成更深嵌套或折叠，**仍然是合法文档**；RST 的常见失败模式是缩进被当成 block quote，只发 system message 软警告而非硬错。
- **事实**：缩进语言只在「同一块内不一致」时报错；**「整体合法、结构却换了」一律静默**。

## 4 空块怎么写

【证】Python 的 suite 不能为空，必须写 `pass`（注释不算语句）；YAML 用 `{}`／`[]`，空标量读成 null；RST 用空注释 `..` 终止并隔离块；Haskell 报告允许插入空 `{}`。

【推】映射到本格式「材料后必须至少一道题」，缩进方案需要额外的空块占位 token，否则「材料后空着」会和「材料属于上一题」混淆。

## 5 真实抱怨

【证】PEP 8 规定 4 空格并禁止混用 TAB/空格，TAB/空格困惑是长期高频问题；CoffeeScript 的空白「完全搞晕编译器」；Makefile 的 `missing separator`（编辑器把 TAB 换成空格）；RST 的 "Unexpected indentation" 警告质量差。出处：[PEP 8](https://peps.python.org/pep-0008/)、[SO](https://stackoverflow.com/questions/45621722/im-getting-an-indentationerror-or-a-taberror-how-do-i-fix-it)、[SO](https://stackoverflow.com/questions/7962549/correct-indentation-for-coffee-script)、[SO](https://stackoverflow.com/questions/16931770/makefile4-missing-separator-stop)。

## 6 面向散文的缩进案例

- 【证】**正面但不干净**：RST 的 directive 体是真·缩进散文块，[spec 明说](https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html) directive 块 = 首行之后所有缩进文本。代价：规则极复杂（块引用、reference indentation、空注释兜底、简单 directive 后跟缩进文本即报错），报错都是软警告；中文因无空格更易误判。
- 【证】**最同构的先例恰恰放弃缩进**：Fountain（剧本格式）语法页原文——"Tabs do not 'hint' formatting to Fountain. They are ignored."，仅在 Action 内保留缩进。理由是文本本身就是内容，缩进必须留给作者排版自由。它改用行首显式标记（`INT.`、全大写、`>`、`@`）。
- 【推】本格式的顶格 `@关键字` 正是 Fountain 这条路线的同类。

## 7 若坚持缩进，必须自加哪些校验

先例：【证】Python 的 tokenizer 用栈生成 INDENT/DEDENT，DEDENT 必须匹配栈中某个已有列，否则报错；Haskell 的 `L` 函数要求嵌套缩进严格递增；F# 用 pre-parse offside 栈与平衡规则。

自加校验清单：① 禁止 TAB；② 缩进必须严格递增；③ DEDENT 必须命中栈中已存在的列，否则硬错；④ 每级缩进宽度固定或首行锚定；⑤ 空块必须显式占位 token；⑥ **粘贴之后无法比对原始空白**——这一步无法可靠自动化。

## 裁决

**在「长散文 ＋ 粘贴 ＋ 必须严格报错」这组约束下，纯缩进语义不可行**，致命约束是**粘贴**：

1. 缩进＝行首空白，而粘贴恰恰最容易破坏行首空白——PDF 复制常丢或合并前导空格，网页与 LLM 输出会被渲染层剥掉缩进（Fountain 与 RST 都因这类问题留了后路）。
2. 长散文的段落续行天然容易被误对齐成兄弟块，改动之后**整体仍然合法、只是语义变了**——正是我们禁止的静默改结构，而且发生在无法控制的输入端。
3. 所有缩进语言都靠 DEDENT 结束块，而 DEDENT 依赖行首空白可信；一旦空白不可信，报错与静默就分不开。

**对质疑者的一句话答复**：语言不强制闭合关键字，是因为它们用 DEDENT 代替闭合；而 DEDENT 的前提是「行首空白可信」——对粘贴进来的中文长散文，这个前提不成立。要保留缩进，就只能把它降级为**可选的严格模式**，并叠加第 7 节①–⑤ 的全部校验，错误信息质量与鲁棒性都达不到 Python 水平（Python 有官方 tokenizer 和三十年打磨，我们两样都没有）。
