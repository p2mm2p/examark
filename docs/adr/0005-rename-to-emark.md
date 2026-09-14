# 项目名与格式名：examark → emark，扩展名 `.emark`

项目名、格式名与 crate 名由 `examark` 改为 `emark`，文件扩展名由 `.examark` 改为 `.emark`；扩展名从此有唯一的常量来源。输入侧不因后缀做任何校验或拒绝——扩展名是标识，不是判据。

## 背景

1. **撞名**。调研既有题目标记格式时（`docs/research/prior-art-formats.md`）检索到一个同为题目对象的 Markdown 格式也叫 **Examark**（`data-wise.github.io/examark`）。两者在检索结果与叙述里反复混淆：同一份材料里出现「Examark」时，读者无法从名字判断说的是本格式还是那个第三方格式。
2. **改名窗口**。v1 尚未发布：crates.io 上 `examark`／`examark-syntax`／`examark-cli` 与 `emark`／`emark-syntax`／`emark-cli` 全都未被占用，没有任何下游消费者。此刻改名的成本只有仓库内部的机械替换，之后再改则要背上已发布命名空间与用户既有文档的迁移代价。
3. **旧扩展名是纯装饰**。CLI 从不看输入文件的后缀：`build` 只要求路径可读且内容能解析成功（`main.rs` 取第一个非选项参数，`build.rs` 按 UTF-8 读入后直接 `parse`）。因此扩展名改名不涉及兼容层、迁移工具或格式版本号。

## 决策

1. **名与后缀同时改**：crate 名 `emark-syntax`／`emark-cli`，二进制名 `emark`，`FORMAT_NAME = "emark"`，扩展名 `.emark`。只改一半会留下「emark 格式的文件叫 `.examark`」这类长期不一致。
2. **扩展名进常量**：`FORMAT_EXTENSION` 与 `FORMAT_NAME` 并列定义在 `emark-syntax` 的 `lib.rs`，值是含前导点的 `.emark`。此前后缀只以字面量形式散落在测试与文档里，没有单一来源；golden 加载等由变量拼后缀的位置改用该常量。
3. **不校验后缀**：维持「可读 UTF-8 ＋ 解析成功即为合法文档」的现状。硬性只认 `.emark` 会新增一个失败模式（作者须先改名才能构建），而收益为零——内容解析比后缀可靠，且 v1 无外部用户来证明这条约束必要。
4. **不重写历史**：旧名留在历史提交与已关闭的 issue 里。本文记录改名事实，供后来者把检索到的旧名对齐到新名。

## 被否决的方案

- **保留 `examark`**：撞名会在「检索题目标记格式」这一最需要名字可辨识的场景里持续造成混淆，而改名成本此刻最低。
- **只改项目名、保留 `.examark`**（或反之）：两个名字必须同时被解释，文档与对话里每次都要补一句「格式现在叫 emark，但后缀还是 .examark」。
- **重写 git 历史**（`filter-repo` 或全量改写提交）：仓库未发布，历史里出现旧名不构成兼容负担；重写需要 force push，并会打断在途分支（`test/add-papers-tests`）。
- **输入侧只接受 `.emark`**：见决策 3。

## 后果

- 旧 `.examark` 文档无需改名即可继续构建；仓库内 5 个 fixture／golden 文件已改为 `.emark`，仓库里只存在一种后缀写法。
- 本仓库更早写的文档与 ADR 正文中的 `examark`／`.examark` 已同步为新名；`docs/research/prior-art-formats.md` 里指向第三方格式 Examark 的两处引用保持原样——那是别人的专名。
- GitHub 仓库名、本地目录名与 crate 名保持一致（仓库载体层面的改名由仓库主人手动执行）。
