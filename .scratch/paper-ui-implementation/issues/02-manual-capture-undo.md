# 02: 交付 Manual Capture 保存与 Undo

**What to build:** 用户从 Today/Vocabulary/Insights 根页右上角打开新版 Manual Capture，保存 Encounter、处理 Achieved 冲突并撤销保存。

**Blocked by:** 01: 交付新版 Settings、应用导航与主题.

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] 入口为 plus-only 按钮，accessible name 保持 Manual capture；Settings、Review、Complete 和其他嵌套页面不显示。
- [x] Dialog 打开清空草稿并聚焦首字段；Word/Context trim 后必填，Translation 可空；Cancel 为唯一可见关闭动作，保留 Escape、焦点陷阱与返回。
- [x] 保存中禁用重复提交；失败保留草稿；成功关闭并刷新，保留重复 Vocabulary Item 的 Encounter 结果、Undo 与 dismiss。
- [x] Achieved 冲突需显式 Return to Learning，恢复和保存原子完成；失败不丢草稿。
- [x] 成功反馈使用合适 toast，但保留现有 Undo 可用性和时机；撤销失败不丢失保存提示。
- [ ] 覆盖正常/最小窗口、150% 文本、两主题、键盘与保存/恢复/撤销失败回归。

## Code changing boundary

允许修改 Manual Capture Dialog、根页入口、反馈与相关展示组件及测试。复用现有保存、Achieved 恢复、Undo 接口；不改保存规则、事务、翻译服务或生命周期资格。每日统计由 08 负责。

本票受总规格的 Code changing boundary 约束；实施前读取总规格。仅做当前切片所需的局部提取和依赖接入，保持其他工作流可运行。越界需求须单独说明原因、影响与范围调整，不能隐含在 UI 迁移内完成。

## Verification

- [x] 盘点本切片现有状态/命令/测试并保留行为断言；验证适用的正常、加载、空、失败、恢复状态。
- [x] 运行相关 UI 测试及类型检查、构建；涉及 Rust 时运行对应契约/存储/应用测试。记录结果，区分既有失败与新增回归。
- [ ] 对照对应 Paper 页面验证 Light/Dark、适用窗口尺寸、150% 文本、多语言、键盘和焦点；原生行为须有对应平台实机证据，未测项不得勾选通过。

## Planning record

用户于 2026-09-09 确认拆分与代码修改边界。ready-for-agent 表示规格可交付，并不表示阻塞项已完成或本次已开始实现；按 Blocked by 的完成状态取票。

## Implementation record — 2026-09-09

本轮用户范围为 01–04。代码与自动化验证已完成；原生验收未全部完成，故未标记 fully verified。详见 [实施与验收记录](../../../docs/plans/2026-09-09-paper-ui-01-04-validation.md)。原生键盘自动化因检测到窗口用户输入而停止；读屏、OS Forced Colors 和真实跨 WebView 同步仍待实机验证。
