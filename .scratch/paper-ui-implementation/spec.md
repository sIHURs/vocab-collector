# 新版 Paper UI 实施规格与执行入口

状态：2026-09-09，01–11 的代码已实现；12 处于 verification-partial。自动化通过，尚未完成的实机项目保留未勾选。详见[05–12 验收记录](../../docs/plans/2026-09-09-paper-ui-05-12-validation.md)。

## 权威依据

- [开发方案](../../docs/plans/2026-09-09-paper-ui-implementation.md)
- [已确认拆分](../../docs/plans/2026-09-09-paper-ui-ticket-breakdown.md)
- [UI 设计规格](../../docs/shadcn-svelte-ui-design-spec.md)
- [领域词汇](../../CONTEXT.md)、[每日统计 ADR 0005](../../docs/adr/0005-local-date-vocabulary-log.md)，并遵循 ADR 0001–0004。
- [Paper 当前文件](https://app.paper.design/file/01M1P47W73ZMCYR34TWR3C1459)：Foundation/Settings/Capture、Today/Review、Vocabulary/Detail、Insights、Native Capture/Accessibility 五页。
- [原 Tickets 01–02 交接](../shadcn-svelte-ui-redesign/paper-ticket-01-02-handoff.md)、[原 Tickets 03–05 交接](../shadcn-svelte-ui-redesign/paper-ticket-03-05-handoff.md)。原五张票保留历史，不修改或关闭；本目录为新版实际执行清单，不按两套重复执行。

现有运行行为与测试优先，明确批准的呈现调整及 ADR 0005 新数据规则除外。Windows 主窗口为主线，Shared 保留平台边界与已有 Capture 行为；不新增平台功能。

## 执行票

| Ticket | Blocked by |
| --- | --- |
| [01 · 交付新版 Settings、应用导航与主题](issues/01-settings-shell-theme.md) | None (can start immediately). |
| [02 · 交付 Manual Capture 保存与 Undo](issues/02-manual-capture-undo.md) | 01: 交付新版 Settings、应用导航与主题. |
| [03 · 交付 Vocabulary 查询与 Encounter 详情](issues/03-vocabulary-encounter-detail.md) | 01: 交付新版 Settings、应用导航与主题. |
| [04 · 交付 Achieved 生命周期管理](issues/04-achieved-lifecycle.md) | 03: 交付 Vocabulary 查询与 Encounter 详情. |
| [05 · 交付 Today 与近期 Encounter 浏览](issues/05-today-recent-encounters.md) | 02: 交付 Manual Capture 保存与 Undo; 03: 交付 Vocabulary 查询与 Encounter 详情. |
| [06 · 交付完整 Focused Review 与完成页](issues/06-focused-review-completion.md) | 05: 交付 Today 与近期 Encounter 浏览. |
| [07 · 交付 Insights 已有指标与历史覆盖状态](issues/07-insights-existing-metrics.md) | 01: 交付新版 Settings、应用导航与主题. |
| [08 · 交付真实 Vocabulary log：保存到每日热力图](issues/08-vocabulary-log.md) | 02: 交付 Manual Capture 保存与 Undo; 07: 交付 Insights 已有指标与历史覆盖状态. |
| [09 · 交付 Windows Native Capture 新界面](issues/09-windows-native-capture.md) | 02: 交付 Manual Capture 保存与 Undo; 04: 交付 Achieved 生命周期管理; 06: 交付完整 Focused Review 与完成页; 08: 交付真实 Vocabulary log：保存到每日热力图. |
| [10 · 交付 Shared Native Capture 新界面](issues/10-shared-native-capture.md) | 02: 交付 Manual Capture 保存与 Undo; 04: 交付 Achieved 生命周期管理; 06: 交付完整 Focused Review 与完成页; 08: 交付真实 Vocabulary log：保存到每日热力图. |
| [11 · 交付 Region OCR Overlay 视觉与坐标验收](issues/11-region-ocr-overlay.md) | 09: 交付 Windows Native Capture 新界面. |
| [12 · 完成新版 UI 跨表面验收与旧样式清理](issues/12-cross-surface-acceptance.md) | 09: 交付 Windows Native Capture 新界面; 10: 交付 Shared Native Capture 新界面; 11: 交付 Region OCR Overlay 视觉与坐标验收. |

代码已按依赖顺序完成，包括主窗口之后的 09/10 原生 Capture 迁移。各票状态区分实现与实机验收；12 的完整跨平台验收仍待完成。

旧交接映射：旧 01 → 新 01–02；旧 02 → 新 05–06；旧 03 → 新 03–04；旧 04 → 新 07–08；旧 05 → 新 09–12。

08 内部先验证存储/应用/桌面每日契约，再连接热力图；它作为端到端票交付。共享统计必须覆盖既有 Native Capture 写入，即使其新视觉尚未迁移。

## Code changing boundary

本边界适用于全部 12 张执行票。允许目录表示可触及的最大范围，不代表需要修改其中所有文件；每张票的更窄范围优先。代码路径为当前仓库定位，职责边界在文件拆分后仍有效。

| 修改区域 | 允许内容 | 约束 |
| --- | --- | --- |
| `ui/src/windows/` | Windows 页面、控件组合、布局、可访问性、局部展示提取 | 保留工作流、命令顺序与错误恢复；OCR 坐标不变 |
| `ui/src/components/` 与按项目别名新增的 UI 组件目录 | 最小 shadcn-svelte primitives 与产品展示组件 | 通用控件不负责数据库、调度和平台业务分支 |
| `ui/src/styles.css` | tokens、主题、字号、尺寸、响应式、Forced Colors | 不用全局样式干扰全屏 OCR 或原生窗口行为 |
| `ui/src/App.svelte`、`ui/src/FloatingCapture.svelte` | Shared 可复用视觉基础、Capture 呈现 | 不补齐 Windows 独有业务，不统一保存流程 |
| `ui/src/lib/`、`ui/src/main.ts` | 主题机制、08 每日类型/查询/真实与 DemoBackend、必要启动初始化 | 保留已有接口语义和平台/窗口选择机制 |
| UI package、Vite/TypeScript、shadcn 配置及锁文件 | 组件、样式、图标所需依赖和配置 | 使用 pnpm；不更换技术栈或升级无关依赖 |
| `crates/domain/` | 仅 08 的每日查询模型、覆盖状态和存储契约 | 不改 Review、Vocabulary 归一化或生命周期规则 |
| `crates/application/` | 仅 08 每日查询及各保存/Undo/recapture 的统计接入 | 保留业务结果，避免重复或旁路计数 |
| `crates/storage/` | 仅 08 的迁移、日期关联、匿名聚合与原子/幂等更新 | 不改变清理期限，永久删除不额外保留词条和上下文 |
| `apps/desktop/src-tauri/src/commands/`、必要命令注册与事件模块 | 08 每日查询，01 必要的已保存外观通知 | 优先复用现有机制，不扩大 OS 权限或改平台捕获实现 |
| 对应测试与开发记录 | 行为保护、统计/迁移测试、实机验收记录 | 不通过削弱原有断言掩盖行为回归 |

### 核心约束

- UI 迁移不改变 Review 调度、评分、submission ID 重试、归一化和重复词识别、Achieve 资格、保留期限或清理规则。
- 每日计数只在 Rust 的真实保存/Undo 事务中更新。前端不得独立执行加一/减一。保存日关联应足以幂等撤销，但不能导致永久删除后残留词条/上下文。
- Azure/DeepL/Apple Translation 的实现、凭据、请求策略，以及 platform 中的选词、OCR、权限、Swift/原生桥接算法均不在本轮范围。
- 保留原生激活、焦点、超时暂停/退出和 OCR 坐标转换。Paper 标题栏与 14px 圆角不授权重写原生窗口框架；只有实现已批准尺寸所必需的最小 Tauri 配置调整属于范围。
- 同步、账号、安装器、发布流水线、全局状态框架替换及无关重构不在范围内。
- 允许服务于当前切片的组件、主题和展示函数提取，保持每个切片可运行；不合并 Windows/Shared 业务页面。旧样式仅在替换已验证、没有调用者后删除。
- 若发现确需越界，先记录对应票、原因、影响及范围变化，沿用用户的范围确认流程；范围外业务修复不能作为本次视觉修改顺带实施。


## 共同完成条件

每票盘点当前状态和测试，只作切片所需局部整理，保留正常/加载/空/失败/恢复路径。每票运行相关测试、类型检查和构建；08 另需 Rust 事务、迁移、时区与幂等验证。检查相应窗口尺寸、Light/Dark/System、独立 150% 文本、多语言、键盘与焦点；原生激活/读屏/Forced Colors/OCR 多显示器需实机证据。最后整体验收不代替每票验收，未验证平台不得标为通过。

## 已确认每日规则

保存时系统本地日期固定归属；Undo 成功从原日扣回一次，失败/重复不多扣。匿名逐日计数长期保留，Achieve/Unachieve/永久删除不减少，页面展示过去 12 个月。升级前不可靠历史为未知，升级当日无完整证据为部分覆盖；从迁移后首个完整本地日起保证覆盖。未知不当零，不用现存 Encounter 或 lifetime 总数伪造历史。
