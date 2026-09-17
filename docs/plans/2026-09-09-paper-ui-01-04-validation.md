# Paper UI Tickets 01–04 实施与验收记录

2026-09-09。用户将本轮实施范围收窄为 Ticket 01–04。四票代码已实现；部分原生验收仍待完成，不能将本记录理解为全部平台验收通过。05–12 未实施。

## 实现结果与代码边界

| Ticket | 本轮结果 |
| --- | --- |
| 01 | Settings 单列表单、Today/Vocabulary/Insights/Settings 导航、Paper 语义主题和基础控件。保存成功后应用外观；主窗口与 Capture 启动读取已保存设置并监听通知。 |
| 02 | 根页 plus-only Manual Capture，Dialog 首字段焦点、Cancel/Escape、保存与 Achieved 冲突恢复。Sonner 保存反馈保留无限期 Undo/dismiss，失败保留草稿和可重试反馈。 |
| 03 | Active/Mastered 语义 Table、原搜索及每页 10 项分页、右侧 Sheet。保留 Encounter 次序和完整来源，详情请求成功后才打开。 |
| 04 | Mastered 行/详情红字描边 Achieve；Achieved 筛选、跨过滤选择、批量 Unachieve/Undo、原生确认永久删除。 |

局部展示提取为 SettingsForm、VocabularyTable、VocabularyDetail、UndoNotification。接入 shadcn-svelte/Tailwind/Bits UI，仅保留当前切片使用的组件族；生成模板的状态选择器已适配实际安装的 Bits UI `data-state` 属性。

Rust 仅在两条设置保存命令成功后发出 `settings-changed`。没有修改数据库、领域模型、Review 调度、Achieve 资格、保留期限、翻译服务或平台捕获算法。Native/Shared Capture 只接入共享颜色基础，其新布局和流程迁移属于 09/10。Insights 仅承接原有指标入口以保持导航可用，未实施 07/08。

## 自动化证据

| 检查 | 结果 |
| --- | --- |
| `pnpm --dir ui test` | 9 文件，92/92 通过，包含 Windows/Shared/Capture/OCR 现有回归 |
| `pnpm --dir ui check` | 0 errors，0 warnings |
| `pnpm --dir ui build` | 生产构建通过 |
| `cargo test -p vocab-desktop --lib` | 9/9 通过 |
| `rustfmt`（两条修改的命令文件） | 通过 |

新增或更新的行为证据包括：保存外观与跨窗口通知读取、过期初始化结果不能覆盖新外观、通知订阅失败仍读取持久化值、设置部分失败、Manual Capture 保存/恢复/Undo 失败保留、详情请求失败和焦点返回、语义表格、跨筛选批量选择、Unachieve/Undo 失败重试、原生 Achieve 确认。

实施过程在预先约定的 UI 行为边界采用红绿测试；保留已有业务断言，只随已批准导航/表格语义调整相应定位器。Toast 使用实例隔离的 host ID，避免卸载后的通知污染后续挂载。

## 视觉和实机证据

- 对照 Paper 当前文件 `01M1P47W73ZMCYR34TWR3C1459` 的已确认设计与交接说明检查。
- 浏览器检查 1040×720、840×600，Light/Dark/System 当前系统外观；Settings、Manual Capture、Vocabulary/Sheet、Mastered/Achieved 均有对应检查。
- 开发专用 `ui/src/test/windows-preview.html` 提供 `theme`、`text=150`、`scenario=long/achieved` 场景。150% 使用根字体 24px，独立于设备 DPI；长中英德文本可换行，表格允许横向滚动，Sheet/Dialog 可纵向滚动，Settings 保存与 Achieved 批量操作可达。
- 视觉检查修复了旧样式层覆盖新控件、Shared 浅色前景/背景失配、150% 标题行高和开关滑块尺寸问题。
- 使用独立应用标识 `app.vocabcollector.paper-ui-validation` 启动真实 Windows 开发窗口，隔离默认用户数据库。已确认主窗口、原生标题栏、Manual Capture 渲染与首字段可见焦点。
- Windows 键盘自动化两次返回 `user input was detected in this window; call get_window_state before continuing`，已停止该路径，未将原生键盘完整验收标为通过。

仍待实机验证：完整原生键盘循环/焦点返回、NVDA 等读屏、OS Forced Colors、实时切换系统外观、两个真实 WebView 的通知同步，以及 macOS Shared 实机表现。现有自动化和 CSS 审查不能代替这些证据。

## Code review

基线：本轮实施前 `ea41e7c3ee119ada1588bfd9655d3e51e89fbf7a`，审查本轮待提交工作树及新增文件。

- Standards：1 项重复深色声明建议及 Forced Colors 优先级风险，已修复并复核；剩余 0 项。
- Spec：1 项 Shared Light 对比度回归，已修复并复核；剩余 0 项。

本轮没有创建 GitHub issue、Linear ticket，也没有修改或关闭原五张设计票。
