# examark 领域上下文

examark 是一种严格的、专用的、无歧义的纯文本格式，用于编写公务员行测题目文档并渲染为 HTML。仓库是一个 Cargo workspace，包含 `examark-syntax`（解析 + HTML 渲染）与 `examark-cli`（build/watch/preview）两个 crate。

## 语言（词汇表）

**题目文档**（question document）：
一种采用 examark 格式的文件，容纳任意数量的题目（一套卷子、一份练习、一个合集）。它是编写与解析的基本单位。
_避免_：题库、题目集

**卷子**（paper）：
一份以整卷或整份练习形式呈现的题目文档，按模块分节组织。
_避免_：模拟卷、测试卷

**题目**（question）：
AST 中的一个考试条目。v1 只有两种形态：独立单选题与材料题（共享材料 + 若干单选小题）。
_避免_：item、problem、题（单独使用）

**单选题**（single-choice question）：
由题干、选项（A/B/C/D）、必填答案与可选解析构成的题目。选项支持文本与行内 LaTeX，不支持图片。
_避免_：MCQ、multiple choice

**材料题**（material question）：
由一段共享材料（文本 + 图片）后接若干单选小题构成，每个小题各有必填答案与可选解析。
_避免_：材料阅读题、阅读理解题

**模块**（module）：
四个固定的顶级行测类别之一：言语理解与表达、判断推理、数量关系、资料分析。模块是固定的封闭枚举，不可自定义。
_避免_：常识（v1 之外）、分节

**子分类**（sub-category）：
模块内官方的二级分类，同样是固定的封闭枚举。例：判断推理 → 图形推理/定义判断/类比推理/逻辑判断。
_避免_：子类型、话题

**答案**（answer）：
题目的必填正确选项。每个单选题/小题各携带一个。
_避免_：key、solution

**解析**（explanation）：
解释题目为何选该项的可选推理内容，支持文本、图片与行内 LaTeX。
_避免_：rationale、note

**行内公式**（inline formula）：
以 `$...$` 定界、被解析为独立节点的 LaTeX 公式。后端用类 KaTeX 的 crate 解析；前端用 KaTeX 渲染。
_避免_：math、latex（单独使用）

**资源**（asset）：
题目文档引用的非文本文件（图片）。表格以截图形式作为图片嵌入；v1 无专用表格语法。
_避免_：resource、figure、attachment

**题目引用**（document reference）：
一条题目稳定指向另一条题目（按文件 + 序号）的能力，为将来的错题本提供基础。完整的跨文档引用语法在 v1 之后；v1 只保证稳定寻址。
_避免_：link、citation
