# 09: 交付 Windows Native Capture 新界面

**What to build:** Windows 用户在 380×280 的新版 Capture 中预览、编辑、确认、保存与 Undo，所有原有失败恢复路径继续有效。

**Blocked by:** 02: 交付 Manual Capture 保存与 Undo; 04: 交付 Achieved 生命周期管理; 06: 交付完整 Focused Review 与完成页; 08: 交付真实 Vocabulary log：保存到每日热力图.

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] 按 Vocabulary Item、Translation、Context、source/status、actions 排布，滚动正文与可达底部操作适配长文本和 150%。
- [x] Selection 自动翻译预览后显式 Save capture；独立字段 Apply changes、stale translation、Translate again 和空翻译保存语义保留。
- [x] OCR Confirmation 为可编辑草稿，Confirm 后翻译且另行 Save；空词禁用，Confirm 失败可重试。
- [x] 保留 Achieved Return to Learning/Cancel 与原子保存；permission、empty selection、unsupported element、OCR failure 使用原可用动作。
- [x] 只在已有 Cancel/Cancel OCR 的指定状态去掉重复 X；Escape 与窗口原生关闭边界不变。
- [x] 保留 request ID 过期抑制、4 秒 saved timeout、hover/focus 暂停、Undo 失败保留结果；不显示新增 stale-request 错误。
- [ ] 实机验证保存→08 日志更新、焦点/激活/退出；主题、Forced Colors、两类缩放和现有 Capture 测试通过。


## Code changing boundary

允许修改 Windows Native Capture 呈现、相关控件、主题接入与测试。保留预览→显式保存、编辑、OCR Confirmation、Undo、request ID 和退出计时语义；不改系统选词/OCR/权限/翻译实现或原生窗口行为。08 的统计仅做接入回归，不另建计数路径。

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
