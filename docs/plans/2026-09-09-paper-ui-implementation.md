# 新版 Paper UI 开发方案

日期：2026-09-09。状态：规划完成，三项待决策已由用户确认；未授权开始实现。

2026-09-09：用户进一步确认 12 张票的拆分和代码修改边界，已发布到[本地执行规格](../../.scratch/paper-ui-implementation/spec.md)。后续以该规格及逐票 Code changing boundary 为实施范围，以执行票记录进度；本方案的阶段划分保留作为架构说明。本次仅发布票据，不开始实现。

## 依据与范围

- 当前 Paper：<https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459>，名称 `vocab-collector-uiux`。已读取五个设计页面的画板目录、116 个 tokens，并查看 Today、Settings、Vocabulary Detail、Insights、Windows Capture 的代表性截图。
- 权威规格：`docs/shadcn-svelte-ui-design-spec.md`；行为细节：`.scratch/shadcn-svelte-ui-redesign/paper-ticket-01-02-handoff.md` 和 `paper-ticket-03-05-handoff.md`。
- 遵循 `CONTEXT.md` 与 ADR 0001–0005。现有应用行为/测试优先于设计样例；规格中明确批准的呈现调整及 ADR 0005 新增每日统计契约按确认方案实施。
- 本文只规划；没有安装依赖、修改产品代码、运行实现测试或创建远程 issue。现有设计文档的工作区修改保留。

## 已确定，无需重复提问

- Today / Vocabulary / Insights / Settings 固定导航；Review 位于 Today 内，保留侧栏。
- Capture 为根页面标题右侧加号，accessible name 为 Manual capture；Settings 和嵌套页面不显示。
- Light / Dark / System，共用语义 tokens；保存设置成功后应用主题，失败不应用草稿。
- 主窗口 1040×720，最小 840×600，Capture 380×280；最小尺寸保持字号，缩小间距并滚动。150% 文本缩放独立验收。
- 14px 窗口视觉圆角不代表引入新的原生窗口控制；OCR 保持全屏坐标模型。
- Vocabulary 使用 Table 和右侧 Sheet；保留 Active/Mastered/Achieved、搜索、分页、确认和 Undo 语义；Achieve 使用语义 destructive 红色描边与文字。
- 保留 Review 的 recall/reveal/submitting/result/next、相同提交 ID 重试、暂停与过期队列保护。
- 按 ADR 0001 保留 Windows 与 Shared 页面边界，共享 tokens 和合适的展示组件。保留已记录的 Native Capture 平台差异，不在此次视觉迁移中统一自动保存行为。Shared 与 ADR 0003 的冲突继续显式记录。
- Vocabulary log 统计每次成功保存，包括重复词；不计 Review、失败或翻译预览。滚动过去 12 个月、蓝色五档、精确日期/次数、键盘导航；无法确认的历史不显示为零。

## 开发阶段与依赖

| 阶段 | 工作与产物 | 完成条件 |
| --- | --- | --- |
| 0 行为基线 | 对照现有五张本地实施单，建立页面/状态/命令/测试映射；记录当前失败，区分既有问题与迁移回归 | 每个设计状态可追溯到实现行为或明确的新数据需求 |
| 1 基础、Settings、Manual Capture | 使用 pnpm 接入 shadcn-svelte 必需基础；配置实际项目别名、样式与 tokens；建立 AppShell、SettingsSection、Dialog 和反馈组件；处理多个 WebView 的已保存主题同步 | 一条完整可工作的设置与手动保存/Undo 路径；保存失败、焦点返回、最小尺寸通过 |
| 2 Today 与 Review | 迁移 Today、近期 Encounter 展示、ReviewCard 与完成页；保留现有状态机、命令和异步防护 | recall 不泄露答案；提交中不能关闭/重复评分；失败保持原卡片；Next/暂停/恢复/完成正确 |
| 3 Vocabulary 与 Detail | 迁移 Table、Tabs、分页、选择、生命周期动作、Sheet 和 EncounterTimeline | 搜索切页规则、Achieved 过滤选择、确认边界、Undo、完整上下文及焦点正确 |
| 4a 每日统计契约 | 按 ADR 0005，在共享 Rust/domain/application/storage 层定义聚合与覆盖状态，再通过桌面边界暴露前端类型 | 保存/Undo 与统计原子一致；时区、历史覆盖、清理、迁移和重试有测试 |
| 4b Insights | 连接已有全局指标与真实每日系列；实现可键盘访问的自定义热力图，组合 Tooltip/ScrollArea/Skeleton；保留 incomplete-history 与 no-session 状态 | 零值、未知、部分历史、加载及恢复可区分；图例与精确次数一致；无伪造历史 |
| 5 Native Capture 与 OCR | 分别迁移 Windows/Shared Capture；共用视觉基础；保留 4 秒退出及各平台暂停规则、请求 ID、编辑/确认/保存/Undo；更新 OCR 视觉 | 原生尺寸与 150% 文本下动作可达；焦点/激活/关闭/多显示器缩放坐标实机验证 |
| 6 跨表面验收与清理 | 核对五个 Paper 页面；完成主题、键盘、读屏、Forced Colors、reduced motion、多语言与原生验证；仅删除已被验证替换的重复样式 | 自动化检查通过，实机结果按平台记录；未测平台不标为完成 |

阶段 2、3、4a 在阶段 1 基础稳定后可独立推进；4b 依赖 1 和 4a；5 按原有实施单在 1–4 后收尾。主窗口以 Windows 为实施与验收主线；Shared 应用可复用的视觉基础，第 5 阶段覆盖 Shared Capture，不将 Windows 独有功能复制到 Shared 旧主窗口。其他平台需各自运行证据。

## 技术边界

当前 UI 为 Svelte 5 / Vite / TypeScript，尚未安装 shadcn-svelte。组件初始化是未来实施工作，不是已有能力。实施时读取官方组件 API 并锁定兼容依赖，使用仓库 pnpm runner；不凭 Paper JSX 直接覆盖 Svelte 页面。

tokens 与 primitives 集中维护，产品组件承载展示组合，页面保留平台工作流编排。精确样式从 Paper tokens/computed styles 获取，截图用于视觉验收。组件按阶段引入，不一次性安装全部库；不为热力图引入日期选择 Calendar 或完整 Data Table。

新每日统计是唯一已明确需要补充的数据契约。建议独立于可清除的词条/Encounter 明细保存匿名日期计数；UI 不通过当前词表重建历史，不在前端从 lifetime 总数猜分布。保存提交、撤销和统计必须共用事务边界；统计失败不能造成保存成功但计数漂移。覆盖 SQLite → application → Tauri → TypeScript Backend 和 DemoBackend；Captured 继续表示 lifetime Vocabulary 数，不改为热力图的保存次数。

## 已确认：每日统计

用户于 2026-09-09 接受以下三项建议；持久决策见 `docs/adr/0005-local-date-vocabulary-log.md`。

1. 日期归属：按保存时系统本地日期记账，记录后不随时区改变重新分桶；跨日 Undo 修正原保存日。避免旅行后过去的格子移动。
2. Undo：撤销成功扣回该次保存，失败不扣，重复撤销不重复扣。与现有 lifetime Encounter 排除已撤销记录一致。UI 文案应明确为未撤销的成功保存。
3. 保留与历史：长期保留匿名逐日计数，不随 Achieve、Unachieve 或永久删除减少；不额外保留词条或上下文。界面只显示过去 12 个月。旧库从迁移后首个完整本地日开始保证完整覆盖；升级当天若无完整证据标为部分覆盖，之前不能证明完整的数据标为未知，不回填伪精确值。

当前决策树已无待确认问题，本轮提问结束。已同步 Vocabulary log 术语、ADR 和现有第 04 张本地实施单；4a 数据契约必须先于 4b 真实图表接入验收。确认规划不等于开始实现授权。

## 验证计划

- 每阶段运行相关 Vitest；阶段完成运行 `pnpm check`、`pnpm test`、`pnpm build`。保留原行为断言，新增测试聚焦呈现迁移风险，不仅断言样式字符串。
- 每日统计补充 Rust 测试：重复词保存、失败回滚、Undo/重复 Undo、跨日、夏令时/时区变化、Achieved recapture、purge、数据库迁移和部分覆盖。
- 视觉矩阵：Light/Dark/System × 正常/最小窗口，另测 150% 文本及长德语/中英文上下文；含 loading/empty/disabled/error/retry/success。
- 交互矩阵：Dialog/Sheet 焦点陷阱和返回、Escape、保存中关闭保护、键盘热力图、读屏状态、Reduced Motion 和 Windows Forced Colors。
- 原生矩阵：窗口拖动/激活/退出时机、设置跨窗口同步、OCR 反向拖动与显示器缩放；macOS/Linux 适用路径须在对应平台验收。

规划阶段未运行测试；上述为实施验收要求，不是当前通过声明。
