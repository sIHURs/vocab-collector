# 11: 交付 Region OCR Overlay 视觉与坐标验收

**What to build:** 用户在新版全屏 OCR Overlay 框选屏幕区域、确认后回到 Capture，缩放和拖动方向不改变识别区域。

**Blocked by:** 09: 交付 Windows Native Capture 新界面.

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] Overlay 保持全屏，不套用主窗口圆角；选择前说明可见，拖动时按现有规则隐藏。
- [x] 矩形使用主题独立的双明暗边界；Forced Colors 使用系统角色而不是固定示例颜色。
- [x] 保留 display-local 坐标、逆向拖动归一化、小于 4px 忽略、outerPosition/scaleFactor 转换及 Escape 取消。
- [x] 与 09 的 editable OCR Confirmation 接通并验证 Confirm→preview→Save→Undo 路径。
- [ ] 实机覆盖多显示器、不同缩放、逆向框选和取消，既有坐标测试保持通过；不引入新的 Shared OCR overlay 功能。


## Code changing boundary

允许修改 Windows OCR Overlay 说明、选区边界、主题/Forced Colors 呈现及测试。保留已有坐标换算、显示器选择、缩放、最小框选阈值及取消行为；不改 OCR 识别算法或原生桥接。

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
