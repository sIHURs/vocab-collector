# 06: 交付完整 Focused Review 与完成页

**What to build:** 用户在 Today 内完成 recall、reveal、rating、结果、Next 和完成总结，也能安全暂停和从失败恢复。

**Blocked by:** 05: 交付 Today 与近期 Encounter 浏览.

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] Review 保留侧栏且 Today 激活，不显示 Capture；recall 仅显示 word/context，不泄露 translation 或评分控件。
- [x] Show answer 后显示 Translation 与 Forgot/Remembered；提交中禁用关闭和重复评分。
- [x] 提交失败保留原揭晓卡片，用相同逻辑 submission ID 和原 rating 重试，禁止提前 Next 或改评分。
- [x] 成功展示 rating、next due、Encounter count；重复忘记的既有 Review Insight 与 Review contexts 保留。
- [x] 关闭暂停并清理 reveal/result、刷新后返回 Today；刷新失败阻止 stale Resume 并提供原重试。
- [x] 最终 Next 与 session insight/刷新失败恢复完整保留；Complete 展示真实计数与 attention words，空 attention 不显示，不新增其上下文按钮。
- [ ] 完成时聚焦标题；Back to Today、详情返回、键盘及两主题/尺寸回归通过。


## Code changing boundary

允许修改 Review 呈现和切片所需的局部组件提取、焦点处理及测试。保留 submission ID、评分/Next/暂停/完成状态转换及请求失效保护；不改 Rust 调度算法、rating 语义或服务契约。

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
