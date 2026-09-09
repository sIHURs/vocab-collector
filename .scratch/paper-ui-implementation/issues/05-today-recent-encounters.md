# 05: 交付 Today 与近期 Encounter 浏览

**What to build:** 用户在新版 Today 查看真实复习计划，打开近期 Vocabulary Item 的 Encounter 详情，并能继续进入现有 Review。

**Blocked by:** 02: 交付 Manual Capture 保存与 Undo; 03: 交付 Vocabulary 查询与 Encounter 详情.

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] 计划数量、total due 和估时来自既有 Today 数据，不加入新计算、分数或 streak。
- [x] 近期记录使用紧凑列表和有界滚动，打开 03 的详情；根页保留 02 的 Capture 入口。
- [x] 保留有效队列、刷新状态和 Start/Resume 的原有门槛；在 06 完成前现有 Review 仍可用。
- [x] 初始加载保持稳定布局；无记录、无 due、刷新失败和稳定内容恢复分别覆盖，仅提供原有重试动作。
- [x] 正常/最小窗口字号一致，150% 与多语言内容不遮挡复习入口；验证数据/导航/错误回归。


## Code changing boundary

允许修改 Today 布局、近期记录展示、已有详情与 Capture 的接入及测试。复用 Today 数据与 Review 入口，不改调度、估时、队列或全局指标计算。

本票受总规格的 Code changing boundary 约束；实施前读取总规格。仅做当前切片所需的局部提取和依赖接入，保持其他工作流可运行。越界需求须单独说明原因、影响与范围调整，不能隐含在 UI 迁移内完成。

## Verification

- [x] 盘点本切片现有状态/命令/测试并保留行为断言；验证适用的正常、加载、空、失败、恢复状态。
- [x] 运行相关 UI 测试及类型检查、构建；涉及 Rust 时运行对应契约/存储/应用测试。记录结果，区分既有失败与新增回归。
- [ ] 对照对应 Paper 页面验证 Light/Dark、适用窗口尺寸、150% 文本、多语言、键盘和焦点；原生行为须有对应平台实机证据，未测项不得勾选通过。

## Planning record

用户于 2026-09-09 确认拆分与代码修改边界。ready-for-agent 表示规格可交付，并不表示阻塞项已完成或本次已开始实现；按 Blocked by 的完成状态取票。

## Implementation record — 2026-09-09

代码及自动化验证已完成；综合实机验收仍待完成。

验证：UI 103/103、相关 Rust 106/106、类型检查 0 errors / 0 warnings、生产构建通过。详细证据、代码边界和未验证项见[验收记录](../../../docs/plans/2026-09-09-paper-ui-05-12-validation.md)。未勾选的综合验收项不视为通过。
