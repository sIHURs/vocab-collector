# 新版 UI 实施 tickets 拆分方案

状态：2026-09-09 用户已确认粒度、依赖与代码修改边界；12 张本地执行票已发布为 ready-for-agent，未开始实现。

依据：已确认的新版 Paper UI 开发方案与 ADR 0001–0005。产品决策不重新开放。本提案仅涉及实施任务拆分，不授权开始实现。

## 发布安排

保留原五张设计交接实施单及其历史，不修改或关闭父单。12 张执行票已发布到 `.scratch/paper-ui-implementation/issues/`，以 [执行规格](../../.scratch/paper-ui-implementation/spec.md) 为入口。旧五张单是交接来源，本套是执行清单，避免按两套重复实现。票据正文按职责描述边界，当前代码定位统一记录在总规格。原阶段 0 的行为盘点与局部整理纳入每张切片起点，不创建无法独立演示的全局重构任务。

## Code changing boundary

完整边界见[执行规格的 Code changing boundary](../../.scratch/paper-ui-implementation/spec.md#code-changing-boundary)，并在各执行票中进一步收窄。

- UI 页面、基础组件、主题和必要构建配置是主要修改区；保留平台选择、工作流与异步恢复规则。
- 只有 08 可扩展每日统计相关 domain/application/storage、事务/迁移和桌面查询接口；01 可补充必要的已保存外观同步。保留现有业务语义。
- 不修改 Review 调度/评分、归一化、生命周期资格/期限、翻译服务或平台选词/OCR/权限算法；不重写原生窗口框架，不引入账号、同步或无关重构。
- 每票只做必要局部提取，测试随切片执行；旧展示代码仅在替换验证且无调用者后清理。越界需求必须记录原因、影响和范围调整。

所有票据保留现有命令、状态机和平台差异。每票完成必须验证本切片正常/加载/空/错误/恢复状态及适用的可访问性，并运行相关测试；不把基本测试拖到最后一票。最后一票承担跨界面的实机整体验收。

## 拆分总览

1. **交付新版 Settings、应用导航与主题** — Blocked by: None。在 Windows 中通过新版导航打开完整 Settings，修改并保存现有设置；成功保存的外观同步到适用窗口，失败保留已应用外观。

2. **交付 Manual Capture 保存与 Undo** — Blocked by: 01。用户从 Today/Vocabulary/Insights 根页右上角打开新版 Manual Capture，保存 Encounter、处理 Achieved 冲突并撤销保存。

3. **交付 Vocabulary 查询与 Encounter 详情** — Blocked by: 01。用户在 Active/Mastered 中搜索、分页并打开新版右侧详情，近期记录和 Review contexts 可复用同一详情展示。

4. **交付 Achieved 生命周期管理** — Blocked by: 03。用户从 Mastered 将 Vocabulary Item 放入 Achieved，在 Achieved 中筛选、批量 Unachieve、Undo 或确认永久删除。

5. **交付 Today 与近期 Encounter 浏览** — Blocked by: 02, 03。用户在新版 Today 查看真实复习计划，打开近期 Vocabulary Item 的 Encounter 详情，并能继续进入现有 Review。

6. **交付完整 Focused Review 与完成页** — Blocked by: 05。用户在 Today 内完成 recall、reveal、rating、结果、Next 和完成总结，也能安全暂停和从失败恢复。

7. **交付 Insights 已有指标与历史覆盖状态** — Blocked by: 01。用户打开新版 Insights 查看真实 lifetime 与当前 due 指标，并明确区分缺失数据、未完成历史和没有 session insight。

8. **交付真实 Vocabulary log：保存到每日热力图** — Blocked by: 02, 07。用户成功保存或 Undo 后，在 Insights 看到准确更新的每日热力图；重复词、时区、升级历史和永久删除均遵循 ADR 0005。

9. **交付 Windows Native Capture 新界面** — Blocked by: 02, 04, 06, 08。Windows 用户在 380×280 的新版 Capture 中预览、编辑、确认、保存与 Undo，所有原有失败恢复路径继续有效。

10. **交付 Shared Native Capture 新界面** — Blocked by: 02, 04, 06, 08。Shared 平台使用同一视觉系统完成既有 Selection/OCR 捕获与 Undo，保留其现有自动保存和权限流程。

11. **交付 Region OCR Overlay 视觉与坐标验收** — Blocked by: 09。用户在新版全屏 OCR Overlay 框选屏幕区域、确认后回到 Capture，缩放和拖动方向不改变识别区域。

12. **完成新版 UI 跨表面验收与旧样式清理** — Blocked by: 09, 10, 11。用户在各个已迁移界面获得一致且可用的新版 UI，开发记录明确哪些平台和辅助功能已通过实机验收。

## 已确认票据快照

以下保留确认时的范围与验收快照；后续状态与边界以本地执行票为准。

### 01: 交付新版 Settings、应用导航与主题

**What to build:** 在 Windows 中通过新版导航打开完整 Settings，修改并保存现有设置；成功保存的外观同步到适用窗口，失败保留已应用外观。

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] 先盘点本切片现有行为与测试；仅做切片所需的局部整理，再接入最小 shadcn-svelte 基础，保持其他页面可运行。
- [ ] 落实语义 Light/Dark/System tokens、固定导航顺序、品牌、平台安全标题/拖动区域；不用页面硬编码配色。
- [ ] Languages、Review、Capture、Appearance 的现有字段、范围、快捷键输入方式、保存时机和错误路径完整保留；不把 Shared 的 ShortcutRecorder 行为新增到 Windows。
- [ ] 设置保存成功后才应用主题和 reduced motion；持久化成功但页面刷新失败仍保留新偏好。
- [ ] 主窗口与 Capture 的已保存外观初始化/同步可验证；System 跟随系统，失败不广播草稿；Shared 仅接入可复用外观基础，不补齐 Windows 独有业务。
- [ ] 正常、最小窗口、150% 文本、键盘、Forced Colors、语言长文本验证；现有测试和类型检查/构建通过。

### 02: 交付 Manual Capture 保存与 Undo

**What to build:** 用户从 Today/Vocabulary/Insights 根页右上角打开新版 Manual Capture，保存 Encounter、处理 Achieved 冲突并撤销保存。

**Blocked by:** 01: 交付新版 Settings、应用导航与主题.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] 入口为 plus-only 按钮，accessible name 保持 Manual capture；Settings、Review、Complete 和其他嵌套页面不显示。
- [ ] Dialog 打开清空草稿并聚焦首字段；Word/Context trim 后必填，Translation 可空；Cancel 为唯一可见关闭动作，保留 Escape、焦点陷阱与返回。
- [ ] 保存中禁用重复提交；失败保留草稿；成功关闭并刷新，保留重复 Vocabulary Item 的 Encounter 结果、Undo 与 dismiss。
- [ ] Achieved 冲突需显式 Return to Learning，恢复和保存原子完成；失败不丢草稿。
- [ ] 成功反馈使用合适 toast，但保留现有 Undo 可用性和时机；撤销失败不丢失保存提示。
- [ ] 覆盖正常/最小窗口、150% 文本、两主题、键盘与保存/恢复/撤销失败回归。

### 03: 交付 Vocabulary 查询与 Encounter 详情

**What to build:** 用户在 Active/Mastered 中搜索、分页并打开新版右侧详情，近期记录和 Review contexts 可复用同一详情展示。

**Blocked by:** 01: 交付新版 Settings、应用导航与主题.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] Table 提供 Word、Translation、Status、Encounters、Last Seen；Active 保留 Learning/Paused，Mastered 独立。
- [ ] 保留 search trim/case-fold 规则、视图切换清空搜索和回到第一页、每页 10 项与页码 clamp/floor；不改变尚未迁移的 Achieved 路径。
- [ ] 详情仍先请求数据再打开 Sheet，按 Word/Translation/Status/Encounter Timeline 展示；保留原 Encounter 次序与完整来源文本。
- [ ] Sheet 的打开、关闭、Escape、焦点陷阱/返回及失败页面错误符合既有行为；不新增编辑、删除或来源链接操作。
- [ ] Mastered 既有 Achieve 动作保持可用，可暂沿用原确认与处理器，交由 04 完成视觉迁移。
- [ ] 加载、空库、无搜索结果、请求失败及长中英德文本均可验收；正常/最小窗口与 150% 保持身份和动作可达。

### 04: 交付 Achieved 生命周期管理

**What to build:** 用户从 Mastered 将 Vocabulary Item 放入 Achieved，在 Achieved 中筛选、批量 Unachieve、Undo 或确认永久删除。

**Blocked by:** 03: 交付 Vocabulary 查询与 Encounter 详情.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] Mastered 行与详情 Achieve 使用 destructive 红字/描边；保留原生确认边界与明确删除日期，不新增自定义确认流程。
- [ ] Achieve 成功切换 Achieved 并刷新；失败保留可恢复错误。
- [ ] Achieved 保留滚动过滤列表而非新增分页；保留 lemma 搜索、截止日期/剩余天数与文本状态。
- [ ] Select all 作用于过滤结果，并保留现有跨过滤选择行为。
- [ ] Unachieve 返回 Mastered，Undo 重新 Achieve 相同集合；永久删除需确认且没有 Undo。
- [ ] 覆盖空态、批量选择、操作失败/Undo 失败、截止日期显示与最小窗口下批量动作可达；领域和持久化规则不变。

### 05: 交付 Today 与近期 Encounter 浏览

**What to build:** 用户在新版 Today 查看真实复习计划，打开近期 Vocabulary Item 的 Encounter 详情，并能继续进入现有 Review。

**Blocked by:** 02: 交付 Manual Capture 保存与 Undo; 03: 交付 Vocabulary 查询与 Encounter 详情.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] 计划数量、total due 和估时来自既有 Today 数据，不加入新计算、分数或 streak。
- [ ] 近期记录使用紧凑列表和有界滚动，打开 03 的详情；根页保留 02 的 Capture 入口。
- [ ] 保留有效队列、刷新状态和 Start/Resume 的原有门槛；在 06 完成前现有 Review 仍可用。
- [ ] 初始加载保持稳定布局；无记录、无 due、刷新失败和稳定内容恢复分别覆盖，仅提供原有重试动作。
- [ ] 正常/最小窗口字号一致，150% 与多语言内容不遮挡复习入口；验证数据/导航/错误回归。

### 06: 交付完整 Focused Review 与完成页

**What to build:** 用户在 Today 内完成 recall、reveal、rating、结果、Next 和完成总结，也能安全暂停和从失败恢复。

**Blocked by:** 05: 交付 Today 与近期 Encounter 浏览.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] Review 保留侧栏且 Today 激活，不显示 Capture；recall 仅显示 word/context，不泄露 translation 或评分控件。
- [ ] Show answer 后显示 Translation 与 Forgot/Remembered；提交中禁用关闭和重复评分。
- [ ] 提交失败保留原揭晓卡片，用相同逻辑 submission ID 和原 rating 重试，禁止提前 Next 或改评分。
- [ ] 成功展示 rating、next due、Encounter count；重复忘记的既有 Review Insight 与 Review contexts 保留。
- [ ] 关闭暂停并清理 reveal/result、刷新后返回 Today；刷新失败阻止 stale Resume 并提供原重试。
- [ ] 最终 Next 与 session insight/刷新失败恢复完整保留；Complete 展示真实计数与 attention words，空 attention 不显示，不新增其上下文按钮。
- [ ] 完成时聚焦标题；Back to Today、详情返回、键盘及两主题/尺寸回归通过。

### 07: 交付 Insights 已有指标与历史覆盖状态

**What to build:** 用户打开新版 Insights 查看真实 lifetime 与当前 due 指标，并明确区分缺失数据、未完成历史和没有 session insight。

**Blocked by:** 01: 交付新版 Settings、应用导航与主题.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] Captured 使用 lifetime Vocabulary，Reviewed 使用 lifetime Reviews，Due 使用 Today backlog；不改为每日保存计数。
- [ ] 展示既有 lifetime Encounter/rating totals、Currently achieved 和 incomplete-rating-history 解释；永久删除后的匿名总数语义保留。
- [ ] ReviewSessionInsight 仅在现有 session object 可用时展示；不添加持久推荐或伪造 session。
- [ ] Vocabulary log 区域按 Paper 显示明确 unavailable，保留布局；不使用样本格子或硬编码历史。
- [ ] GlobalInsight 缺失与零值不同；loading、partial、error 与原页面刷新恢复可验证。
- [ ] 在最小窗口和 150% 下可滚动访问所有指标，读屏可理解数字与状态。

### 08: 交付真实 Vocabulary log：保存到每日热力图

**What to build:** 用户成功保存或 Undo 后，在 Insights 看到准确更新的每日热力图；重复词、时区、升级历史和永久删除均遵循 ADR 0005。

**Blocked by:** 02: 交付 Manual Capture 保存与 Undo; 07: 交付 Insights 已有指标与历史覆盖状态.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] 端到端贯通持久化、共享应用逻辑、桌面接口、真实/演示 Backend 与 Insights；在接图前完成数据契约测试，不单独交付只有数据库表的横向任务。
- [ ] 覆盖 Manual Capture、各平台既有 Native Capture 保存与 Achieved recapture；每次实际成功保存计一次，失败、Review、翻译预览不计。
- [ ] 按保存时系统本地日期固定归属；成功 Undo 原子扣回原日期一次，重复或失败 Undo 不重复扣减；任何写入失败不产生明细与计数漂移。
- [ ] 长期保留匿名逐日计数，Achieve/Unachieve/purge 不减少；不得额外保留已删除的词条/上下文。
- [ ] 迁移前不可靠日期为未知，升级当日无完整证据则 partial，之后首个完整本地日起可声明完整；未知不能当零，部分期间总数不得标为完整精确总量。
- [ ] 呈现过去 12 个月、Sunday-first 日历、蓝色五档 0/1–2/3–5/6–9/10+；未来/区间外日期留白；精确日期/次数 tooltip 与可访问文本。
- [ ] 单 tab stop、方向键按日/周移动、Escape 关闭 tooltip；最小窗口保持字号/格子尺寸，水平滚动且初始可见最新日期。
- [ ] 保存/Undo 后读取真实更新值，重启后保留；当前日期跨日时刷新窗口范围与覆盖状态。
- [ ] Rust 与界面集成测试覆盖事务回滚、重复保存/Undo、跨日/夏令时/时区变化、迁移重启、recapture 与 purge；完整/部分/未知/loading/error 状态均可演示。

### 09: 交付 Windows Native Capture 新界面

**What to build:** Windows 用户在 380×280 的新版 Capture 中预览、编辑、确认、保存与 Undo，所有原有失败恢复路径继续有效。

**Blocked by:** 02: 交付 Manual Capture 保存与 Undo; 04: 交付 Achieved 生命周期管理; 06: 交付完整 Focused Review 与完成页; 08: 交付真实 Vocabulary log：保存到每日热力图.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] 按 Vocabulary Item、Translation、Context、source/status、actions 排布，滚动正文与可达底部操作适配长文本和 150%。
- [ ] Selection 自动翻译预览后显式 Save capture；独立字段 Apply changes、stale translation、Translate again 和空翻译保存语义保留。
- [ ] OCR Confirmation 为可编辑草稿，Confirm 后翻译且另行 Save；空词禁用，Confirm 失败可重试。
- [ ] 保留 Achieved Return to Learning/Cancel 与原子保存；permission、empty selection、unsupported element、OCR failure 使用原可用动作。
- [ ] 只在已有 Cancel/Cancel OCR 的指定状态去掉重复 X；Escape 与窗口原生关闭边界不变。
- [ ] 保留 request ID 过期抑制、4 秒 saved timeout、hover/focus 暂停、Undo 失败保留结果；不显示新增 stale-request 错误。
- [ ] 实机验证保存→08 日志更新、焦点/激活/退出；主题、Forced Colors、两类缩放和现有 Capture 测试通过。

### 10: 交付 Shared Native Capture 新界面

**What to build:** Shared 平台使用同一视觉系统完成既有 Selection/OCR 捕获与 Undo，保留其现有自动保存和权限流程。

**Blocked by:** 02: 交付 Manual Capture 保存与 Undo; 04: 交付 Achieved 生命周期管理; 06: 交付完整 Focused Review 与完成页; 08: 交付真实 Vocabulary log：保存到每日热力图.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] 保留 Selection 翻译成功后自动保存；OCR 为只读建议与 Use this text 后同一流程；显式记录与 ADR 0003 的既有冲突，不在此改行为。
- [ ] typed translation_unavailable/failed 才显示既有 Retry 或 Save without translation，generic diagnostics 不新增动作。
- [ ] 保留 Accessibility/Screen Recording typed 权限命令，以及 capability 允许时的 Region OCR 恢复；不复制 Windows 独有 Achieved UI。
- [ ] Saved 内容、Undo、一般失败替换结果、4 秒 timeout、hover 暂停及 request ID 防护保持原样；不增加当前没有的 Escape 处理器。
- [ ] 380×280、150% 文本、两主题与 reduced motion 下动作可达；对应平台实机验证权限、窗口行为以及保存→08 的统计更新。

### 11: 交付 Region OCR Overlay 视觉与坐标验收

**What to build:** 用户在新版全屏 OCR Overlay 框选屏幕区域、确认后回到 Capture，缩放和拖动方向不改变识别区域。

**Blocked by:** 09: 交付 Windows Native Capture 新界面.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] Overlay 保持全屏，不套用主窗口圆角；选择前说明可见，拖动时按现有规则隐藏。
- [ ] 矩形使用主题独立的双明暗边界；Forced Colors 使用系统角色而不是固定示例颜色。
- [ ] 保留 display-local 坐标、逆向拖动归一化、小于 4px 忽略、outerPosition/scaleFactor 转换及 Escape 取消。
- [ ] 与 09 的 editable OCR Confirmation 接通并验证 Confirm→preview→Save→Undo 路径。
- [ ] 实机覆盖多显示器、不同缩放、逆向框选和取消，既有坐标测试保持通过；不引入新的 Shared OCR overlay 功能。

### 12: 完成新版 UI 跨表面验收与旧样式清理

**What to build:** 用户在各个已迁移界面获得一致且可用的新版 UI，开发记录明确哪些平台和辅助功能已通过实机验收。

**Blocked by:** 09: 交付 Windows Native Capture 新界面; 10: 交付 Shared Native Capture 新界面; 11: 交付 Region OCR Overlay 视觉与坐标验收.

**Status:** ready-for-agent (published snapshot; see execution ticket for current status)

- [ ] 汇总各票验收证据，对照全部五个 Paper 设计页面与平台适用状态，无遗漏业务动作和恢复路径。
- [ ] 检查 Light/Dark/System、1040×720、840×600、380×280、独立 150% 文本缩放及长中英德文本。
- [ ] 实测键盘导航、Dialog/Sheet 焦点陷阱与返回、读屏名称/状态、Reduced Motion、Windows Forced Colors。
- [ ] 实测窗口拖动/激活/退出、跨 WebView 已保存主题同步及 OCR 坐标；macOS/Linux 适用检查需对应环境，不用截图或 jsdom 代替。
- [ ] 仅删除替代项已验收且没有调用者的旧样式/展示代码；不合并 Windows/Shared 业务页面或变更行为。
- [ ] 完整类型检查、UI 测试、构建及相关 Rust 测试通过；记录命令、平台、结果和未验证项，未测项目不标为完成。

## 依赖说明

- 01 为新视觉基础；02、03、07 可在其后分别开展。
- 05 依赖新版 Capture 入口和可复用详情；06 沿用 05 的 Today/Review 边界。
- 08 以真实保存到图表为一个完整切片，内部先验证每日契约再接热力图；避免数据层和 UI 票分别完成却不可验收。
- 09/10 延续已确认方案在主窗口阶段完成后再迁移 Native Capture 的顺序；其阻塞链覆盖 Settings、Manual Capture、Vocabulary、Today/Review、Insights，不隐式取消原阶段门槛。
- 11 依赖 Windows Capture 的 OCR Confirmation 回接流程；12 的传递依赖覆盖全部切片。

旧五张交接单映射：旧 01 → 新 01–02；旧 02 → 新 05–06；旧 03 → 新 03–04；旧 04 → 新 07–08；旧 05 → 新 09–12。
