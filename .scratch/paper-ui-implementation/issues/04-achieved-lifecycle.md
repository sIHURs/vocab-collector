# 04: 交付 Achieved 生命周期管理

**What to build:** 用户从 Mastered 将 Vocabulary Item 放入 Achieved，在 Achieved 中筛选、批量 Unachieve、Undo 或确认永久删除。

**Blocked by:** 03: 交付 Vocabulary 查询与 Encounter 详情.

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] Mastered 行与详情 Achieve 使用 destructive 红字/描边；保留原生确认边界与明确删除日期，不新增自定义确认流程。
- [x] Achieve 成功切换 Achieved 并刷新；失败保留可恢复错误。
- [x] Achieved 保留滚动过滤列表而非新增分页；保留 lemma 搜索、截止日期/剩余天数与文本状态。
- [x] Select all 作用于过滤结果，并保留现有跨过滤选择行为。
- [x] Unachieve 返回 Mastered，Undo 重新 Achieve 相同集合；永久删除需确认且没有 Undo。
- [x] 覆盖空态、批量选择、操作失败/Undo 失败、截止日期显示与最小窗口下批量动作可达；领域和持久化规则不变。

## Code changing boundary

允许修改 Mastered/Achieved 操作的呈现、选择状态、确认接入、反馈及测试。保留原生确认边界、Achieve 资格/截止日期、Unachieve/Undo 与删除规则；不修改后台生命周期计算或清理算法。

本票受总规格的 Code changing boundary 约束；实施前读取总规格。仅做当前切片所需的局部提取和依赖接入，保持其他工作流可运行。越界需求须单独说明原因、影响与范围调整，不能隐含在 UI 迁移内完成。

## Verification

- [x] 盘点本切片现有状态/命令/测试并保留行为断言；验证适用的正常、加载、空、失败、恢复状态。
- [x] 运行相关 UI 测试及类型检查、构建；涉及 Rust 时运行对应契约/存储/应用测试。记录结果，区分既有失败与新增回归。
- [ ] 对照对应 Paper 页面验证 Light/Dark、适用窗口尺寸、150% 文本、多语言、键盘和焦点；原生行为须有对应平台实机证据，未测项不得勾选通过。

## Planning record

用户于 2026-09-09 确认拆分与代码修改边界。ready-for-agent 表示规格可交付，并不表示阻塞项已完成或本次已开始实现；按 Blocked by 的完成状态取票。

## Implementation record — 2026-09-09

本轮用户范围为 01–04。代码与自动化验证已完成；原生验收未全部完成，故未标记 fully verified。详见 [实施与验收记录](../../../docs/plans/2026-09-09-paper-ui-01-04-validation.md)。原生键盘自动化因检测到窗口用户输入而停止；读屏、OS Forced Colors 和真实跨 WebView 同步仍待实机验证。
