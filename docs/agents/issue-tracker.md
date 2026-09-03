# Issue 跟踪：GitHub

本仓库的 issue 与 spec 以 GitHub issue 形式存在。所有操作使用 `gh` CLI。

## 约定

- **创建 issue**：`gh issue create --title "..." --body "..."`。多行正文用 heredoc。
- **读取 issue**：`gh issue view <编号> --comments`，用 `jq` 过滤评论并获取标签。
- **列出 issue**：`gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`，配合适当的 `--label` 与 `--state` 过滤。
- **评论 issue**：`gh issue comment <编号> --body "..."`
- **打标签 / 去标签**：`gh issue edit <编号> --add-label "..."` / `--remove-label "..."`
- **关闭**：`gh issue close <编号> --comment "..."`

仓库从 `git remote -v` 推断；在克隆内运行时 `gh` 会自动完成。

## 把拉取请求作为 triage 入口

**将 PR 作为请求入口：否。** _（若本仓库将来把外部 PR 视为功能请求，改为 `yes`；`/triage` 会读取此标记。）_

设为 `yes` 时，PR 走与 issue 相同的标签与状态，使用 `gh pr` 等价命令：

- **读取 PR**：`gh pr view <编号> --comments` 与 `gh pr diff <编号>`（差异）。
- **列出待 triage 的外部 PR**：`gh pr list --state open --json number,title,body,labels,author,authorAssociation,comments`，只保留 `authorAssociation` 为 `CONTRIBUTOR`、`FIRST_TIME_CONTRIBUTOR` 或 `NONE` 的（丢弃 `OWNER`/`MEMBER`/`COLLABORATOR`）。
- **评论 / 标签 / 关闭**：`gh pr comment`、`gh pr edit --add-label`/`--remove-label`、`gh pr close`。

GitHub 的 issue 与 PR 共用一套编号，裸 `#42` 可能是任一：用 `gh pr view 42` 解析，失败则回退 `gh issue view 42`。

## 当技能说「发布到 issue 跟踪器」

创建一个 GitHub issue。

## 当技能说「获取相关 ticket」

运行 `gh issue view <编号> --comments`。

## Wayfinding 操作

由 `/wayfinder` 使用。**地图**是单个 issue，其**子 ticket** 是子 issue。

- **地图**：一个带 `wayfinder:map` 标签的 issue，承载 Notes / Decisions-so-far / Fog 正文。`gh issue create --label wayfinder:map`。
- **子 ticket**：作为 GitHub 子 issue 关联到地图的 issue（对子 issue 端点使用 `gh api`）。若子 issue 未启用，则把子项加进地图正文的任务清单，并在子项正文顶部写 `Part of #<地图>`。标签：`wayfinder:<类型>`（`research`/`prototype`/`grilling`/`task`）。被认领后 ticket 指派给主导开发者。
- **阻塞**：GitHub 的**原生 issue 依赖**，是规范的、UI 可见的表示。用 `gh api --method POST repos/<owner>/<repo>/issues/<child>/dependencies/blocked_by -F issue_id=<阻塞者数据库id>` 添加边；其中 `<阻塞者数据库id>` 是阻塞者的数字**数据库 id**（`gh api repos/<owner>/<repo>/issues/<n> --jq .id`，_不是_ `#编号` 或 `node_id`）。GitHub 通过 `issue_dependencies_summary.blocked_by`（仅开放阻塞者，即实时闸门）上报。依赖不可用时，回退为在子项正文顶部写 `Blocked by: #<n>, #<n>`。当所有阻塞者关闭后 ticket 即解除阻塞。
- **前沿查询**：列出地图的开放子项（`gh issue list --state open`，限定到地图的子 issue / 任务清单），丢弃带开放阻塞者（`issue_dependencies_summary.blocked_by > 0`，或 `Blocked by` 行中有开放 issue）或已有指派者的；地图顺序中第一个胜出。
- **认领**：`gh issue edit <n> --add-assignee @me`，即本次会话的第一次写入。
- **解决**：`gh issue comment <n> --body "<答案>"`，然后 `gh issue close <n>`，再把上下文指针（gist + 链接）追加到地图的 Decisions-so-far。
