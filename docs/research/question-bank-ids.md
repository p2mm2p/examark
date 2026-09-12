# 题库与错题本如何标识一道题

本文件回答：题库类产品与规范怎么给题目标识、以及题目被改动或删除后引用如何维持。

- 调研时间：2026-09-12｜方法：联网检索规范文档、产品帮助与社区实践，逐条附出处

## 事实清单

**1. QTI `identifier` 的约束**
- 格式：XSD 中 `IdentifierDType = xs:NCName`（须以字母或下划线开头，不能含空格/冒号）；`UniqueIdentifierDType = xs:ID`。item 的 `identifier` 为必填。[xsd](https://purl.imsglobal.org/spec/qti/v3p0/schema/xsd/imsqti_asiv3p0p1_v1p0.xsd)
- 作用域：item 的 identifier 是 principal identifier，被 `qti-assessment-item-ref` 引用时两者值 MUST 相同；section、item-ref、testPart 的 identifier 须在 test 内唯一；规范建议 ≤32 字符。[info model](https://www.imsglobal.org/spec/qti/v3p0/info/)
- 用途：item identifier 被描述为「全局唯一、供其他文档引用」。[QTI Playground](https://qtiplayground.app/learn/assessment-item)

**2. Anki 的 note guid**
- Anki 自动分配唯一 ID 用于查重；导出含 GUID，**不改 GUID 即可重导入原地更新**；提供非空 GUID 则不新建重复条目。官方建议自造 ID（如 `MYNOTE0001`）放**第一字段**，而不是占用内部 GUID。[Anki Manual](https://docs.ankiweb.net/importing/text-files.html)
- genanki 默认 guid ＝ 字段内容 SHA-256 → base91：**内容一改，guid 即变**。[DeepWiki](https://deepwiki.com/kerrickstaley/genanki/8-advanced-topics)
- 社区 add-on 专门把 note id 放第一字段，使「第一字段改变后仍能匹配」。[AnkiWeb](https://ankiweb.net/shared/info/1672832404)

**3. 中文与通用产品**
- 粉笔、华图对同一道真题会给出不同答案。[知乎](https://www.zhihu.com/question/265415797)
- 粉笔错题本：答对后自动移出错题本、云同步可导出；无公开的「题目 id」文档。[php.cn](https://www.php.cn/faq/2187629.html)
- Notion 唯一 ID：自动分配、**页码永不改变**，且**包含已删除页面**。[Notion Help](https://www.notion.com/zh-cn/help/unique-id)
- Obsidian 间隔重复插件把 Block ID 附在卡片行尾，**内容与结构变动后仍保住复习记录**。[DeepWiki](https://deepwiki.com/open-spaced-repetition/obsidian-spaced-repetition-recall/4.5-card-block-ids-and-inline-scheduling)
- Obsidian 重命名默认更新链接，否则静默断链（红链）。[论坛](https://forum.obsidian.md/t/obsidian-note-renaming-bug-causing-broken-links-and-delayed-updates/59355)

**4. 「年份＋卷别＋题号」算不算 id**
- 真题素材普遍以「2024 国考副省级＋题号」呈现。[kkgwy](https://www.kkgwy.com/zl/gwy/14662.html)
- 回忆版真题「可能与官方有出入」；跨机构答案与编号不一致。[知乎](https://zhuanlan.zhihu.com/p/2081792682754687848)
- 【推断】它只是人类定位标签，不是稳定 id：机构不一、修订重排、回忆版失真都会让它漂移。

**5. 悬空引用的处理**
- 301 重定向 / 404 / **410 Gone**（永久移除）三态。[http.dev/410](https://http.dev/410)
- Wikipedia 重命名后旧标题变重定向页自动跳转。[Wikipedia:Redirect](https://en.wikipedia.org/wiki/Wikipedia:Redirect)
- DOI/DataCite 删除后**解析到 tombstone 落地页，保留引用元数据**。[Dataverse](https://guides.dataverse.org/en/latest/user/dataset-management.html) ｜ [DesignSafe](https://designsafe-ci.org/user-guide/curating/policies/)
- ARK 规范：撤回的标识符应指向 tombstone 页；「persistent identifiers are never guaranteed」。[ARK FAQ](https://arks.org/about/ark-faq-en/)

**6. 手写 id 的成败**
- BibTeX 重复 key **被静默只留一个**，合并 .bib 尤其危险。[bibtexchecker](https://www.bibtexchecker.com/news/5-common-bibtex-errors)
- Zotero 原生导出的 key「通常」唯一，但非唯一时后缀非确定，且 typo 修正后 key 会变、导出前看不到 key。[BBT citing](https://retorque.re/zotero-better-bibtex/citing/)

**7. 工具自动生成并写回**
- Better BibTeX：生成 key 写入 extra 字段、可 Pin，Auto-Export 重写 .bib；**改 pattern 不回溯，须手动 Refresh**，键会在编辑后漂移，故需 Pin。[wiki](https://github.com/retorquere/zotero-better-bibtex/wiki/Citation-Keys)
- Pandoc `auto_identifiers`：id 由标题文本派生，文本变即变，重名加 `-1`。[pandoc](https://pandoc.org/demo/example33/7.2-headings-and-sections.html)

## 对本设计最相关的五条教训

1. **id 不要从内容派生**：genanki/pandoc 的教训是内容派生 id 一改正文就换身份 → 重复或断链。id 应是短、稳定、与文本无关的 token。
2. **显示编号与身份 id 分离**：「2024 国考副省级 106」可读但不可靠（跨机构、回忆版、重排）；Notion/ARK 都主张不透明主体名 ＋ 人类可读 label。
3. **材料与小题同源**：QTI 强制 `item-ref.identifier == item.identifier` 且 test 内唯一；材料及其小题应共享可校验的父 id，防重排/改名错配。
4. **删除要留墓碑，不静默丢引用**：对照 DOI/ARK 的 tombstone 与粉笔「答对即消失」，错题本遇到失效题应显示「已移除/已替换」。
5. **工具兜底 ＋ 生成后固定**：手写 id 的常见错误是拼错、重复、漏写（BibTeX 的教训）；应像 BBT 一样 lint 重复并可自动补 id，再 Pin 防漂移。

> 边界说明（推断）：未找到中文题库 App 公开「题目 id 与下架后错题表现」的一手文档，第 3、4 条中关于「下架导致错题引用丢失/编号漂移」的部分为推断。
