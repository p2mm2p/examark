# 领域文档

工程技能在探索代码库时应如何消费本仓库的领域文档。

## 探索前先读这些

- 根目录的 **`CONTEXT.md`**，或
- 若存在根目录的 **`CONTEXT-MAP.md`**：它指向每个上下文各自的 `CONTEXT.md`。读取与你将要工作的主题相关的每一个。
- **`docs/adr/`**：阅读与你即将工作的区域相关的 ADR。多上下文仓库中还要检查 `src/<上下文>/docs/adr/` 中上下文级的决策。

若这些文件不存在，**静默继续**。不要标记缺失，也不要主动建议创建。`/domain-modeling` 技能（经 `/grill-with-docs` 与 `/improve-codebase-architecture` 触达）会在术语或决策真正被敲定时惰性创建它们。

## 文件结构

单上下文仓库（大多数仓库）：

```
/
├── CONTEXT.md
├── docs/adr/
│   ├── 0001-event-sourced-orders.md
│   └── 0002-postgres-for-write-model.md
└── src/
```

多上下文仓库（根目录存在 `CONTEXT-MAP.md`）：

```
/
├── CONTEXT-MAP.md
├── docs/adr/                          ← 全系统决策
└── src/
    ├── ordering/
    │   ├── CONTEXT.md
    │   └── docs/adr/                  ← 上下文级决策
    └── billing/
        ├── CONTEXT.md
        └── docs/adr/
```

## 使用词汇表的术语

当你的输出命名一个领域概念时（issue 标题、重构提案、假设、测试名），使用 `CONTEXT.md` 中定义的术语。不要漂移到词汇表明确规避的同义词。

若你需要的概念尚不在词汇表中，那是一个信号：要么你在发明项目不用的语言（重新考虑），要么确有缺口（记下供 `/domain-modeling` 处理）。

## 标记 ADR 冲突

若你的输出与既有 ADR 冲突，明确浮出而非悄悄覆盖：

> _与 ADR-0007（event-sourced orders）冲突，但值得重开，因为…_
