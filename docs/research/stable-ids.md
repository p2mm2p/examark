# 纯文本条目的稳定 id：方案与失效模式

本文件回答：人类手写的纯文本文档里，如何给条目分配「插入不漂移、删除不跳号、编辑不换身份」的 id。

- 调研时间：2026-09-12｜方法：联网检索各方案官方文档与实证帖，逐条附出处
- 结论先行：位置序号与内容哈希**都不满足**「同一题编辑后 id 不变、删除不跳号」；须在文件里手写一个**与内容、位置都解耦的稳定 id**。

## 一、候选方案

**A. 自增序号 / 文档内序号**（题号、issue 编号）

- 形式 `1,2,3…`；作者按序写。**插入**：仅尾部稳定，中间插入需重排，其后全部移动；GitHub 用仓库级共享计数器避免重排（issue/PR/discussion 共用一个递增计数器）。**删除**：留空洞，且编号**不复用**。**编辑**：改内容不改号。
- 失效模式：`材料一-3` 这类位置引用，前面删一题即指错。
- https://github.com/orgs/community/discussions/69759 ｜ https://github.com/orgs/community/discussions/23580

**B. 不透明随机 id**（UUIDv4 / v7 / ULID / KSUID / nanoid / Snowflake）

- 形式：v4＝32 位十六进制随机；v7/ULID/KSUID/Snowflake 带时间前缀、按创建可排序。谁生成：解析器或工具写回文件。**插入**：与位置无关，任意位置插入不影响他人，不存在空洞概念。**删除**：其余 id 不动、**永不复用**。**编辑**：id 与内容解耦，改错别字不变。**碰撞**：v4 有 122 随机位（≈5.3×10³⁶），需 2.71×10¹⁸ 个才有 50% 碰撞概率。
- 失效模式：长、不可读、需复制粘贴、手抄易错。
- https://en.wikipedia.org/wiki/Universally_unique_identifier ｜ https://en.wikipedia.org/wiki/Snowflake_ID

**C. 内容寻址**（git blob SHA、IPFS CID）

- 形式：id ＝ 内容哈希；同内容同 id，改一字节即换 id。**插入/删除**：与位置无关。**编辑**：**改错别字或标点就换 id，旧引用全失效**——对错题本是致命伤。
- 缓解：①加可变指针层（IPNS 用稳定名指向会变的 CID）；②Git 的做法是树条目按**路径**引用 blob，哈希只做校验、不当身份。
- https://git-scm.com/book/en/v2/Git-Internals-Git-Objects ｜ https://docs.ipfs.tech/concepts/immutability/

**D. 人类可读 slug**（BibTeX key / Sphinx、LaTeX `\label` / DOI）

- 形式：作者自定短名（如 `lecun1989backprop`）；Zotero 默认按 `作者+标题前三词+年份` 生成。谁生成：作者手写或工具按模板生成。**冲突**：Sphinx 同名 label 重复即告警；Zotero 标准导出的 key 在导出时现算，**非唯一时谁被加后缀并不确定**，且改作者名/标题 typo 就换 key——故 Better BibTeX 支持把 key **钉死（pin）**，冲突默认加 a、b、c。
- 失效模式：作者顺手改 key → 引用断。可读、可手抄，但需要唯一性治理。
- https://retorque.re/zotero-better-bibtex/citing/ ｜ https://stackoverflow.com/questions/62631362

**E. 混合：稳定 id ＋ 人类可读前缀**（Stripe `cus_`/`pi_`、Jira `PROJ-123`）

- 形式 `<前缀>_<随机或序号>`（如 `2024gk-fs_9f3a2c`）。前缀作用：Stripe 称前缀提升可读性、可据前缀判断对象类型、可写正则防误用手抄。**跨文档不撞**：前缀自带命名空间（文档/年份），随机段保证唯一。Jira：项目键＋计数器，**issue key 永不复用**，移动后旧键保留为跳转。
- https://dev.to/4thzoa/designing-apis-for-humans-object-ids-3o5a ｜ https://support.atlassian.com/jira/kb/reset-the-project-counter-and-reuse-jira-issue-keys-after-moving-issues/

## 二、删除语义与悬空引用

- 软删除/墓碑：删除时留 `deleted_at` 或墓碑页，保留期后清理——可审计、可撤销、防跨系统引用悬空。
- 事实：GitHub 删号不回收、Jira 旧 key 保留重定向。悬空引用的常见处理是重定向映射表 / 显示「已删除」/ 解析校验时告警。

## 三、对本设计最相关的三条教训

1. 位置序号＝引用脆弱，内容哈希＝编辑即换身份；**用与内容、位置解耦的稳定 id 写进文件**（B 或 E）。
2. 手写 id 要防手抄错与跨文档碰撞：**前缀带文档命名空间 ＋ 随机长尾段**（E）；比纯 slug 抗碰撞、比纯 UUID 可读。
3. 删除**不回收 id**：墓碑或永不复用 ＋ id 与内容分离；解析时对找不到的 id **报悬空，而不是静默错位**。

## 事实与推断

- 查证到的事实：上述带出处的机制（UUID 熵与碰撞概率、Snowflake 位结构、git/IPFS 内容寻址、Zotero/BBT、Sphinx 重复 label 告警、Stripe 前缀、Jira 不复用、GitHub 共享计数器）均引自对应 URL。
- 推断：把上述机制套到「行测题文档」的取舍系基于事实的工程判断，非出处直接结论；「GitHub 删 issue 后编号不回填」由 community 讨论推断，官方文档未明文承诺。
