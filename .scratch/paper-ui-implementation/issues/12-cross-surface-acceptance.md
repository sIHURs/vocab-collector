# 12: 完成新版 UI 跨表面验收与旧样式清理

**What to build:** 用户在各个已迁移界面获得一致且可用的新版 UI，开发记录明确哪些平台和辅助功能已通过实机验收。

**Blocked by:** 09: 交付 Windows Native Capture 新界面; 10: 交付 Shared Native Capture 新界面; 11: 交付 Region OCR Overlay 视觉与坐标验收.

**Status:** verification-partial

## Acceptance criteria

- [ ] 汇总各票验收证据，对照全部五个 Paper 设计页面与平台适用状态，无遗漏业务动作和恢复路径。
- [ ] 检查 Light/Dark/System、1040×720、840×600、380×280、独立 150% 文本缩放及长中英德文本。
- [ ] 实测键盘导航、Dialog/Sheet 焦点陷阱与返回、读屏名称/状态、Reduced Motion、Windows Forced Colors。
- [ ] 实测窗口拖动/激活/退出、跨 WebView 已保存主题同步及 OCR 坐标；macOS/Linux 适用检查需对应环境，不用截图或 jsdom 代替。
- [x] 仅删除替代项已验收且没有调用者的旧样式/展示代码；不合并 Windows/Shared 业务页面或变更行为。
- [x] 完整类型检查、UI 测试、构建及相关 Rust 测试通过；记录命令、平台、结果和未验证项，未测项目不标为完成。


## Code changing boundary

允许修复本套票引入的展示/可访问性回归，补充验收证据，并删除已验证替换且无调用者的旧展示代码/样式。不得以清理为由重构业务、合并平台页面、重写窗口框架或删除仍被未迁移路径使用的代码。

本票受总规格的 Code changing boundary 约束；实施前读取总规格。仅做当前切片所需的局部提取和依赖接入，保持其他工作流可运行。越界需求须单独说明原因、影响与范围调整，不能隐含在 UI 迁移内完成。

## Verification

- [x] 盘点本切片现有状态/命令/测试并保留行为断言；验证适用的正常、加载、空、失败、恢复状态。
- [x] 运行相关 UI 测试及类型检查、构建；涉及 Rust 时运行对应契约/存储/应用测试。记录结果，区分既有失败与新增回归。
- [ ] 对照对应 Paper 页面验证 Light/Dark、适用窗口尺寸、150% 文本、多语言、键盘和焦点；原生行为须有对应平台实机证据，未测项不得勾选通过。

## Planning record

用户于 2026-09-09 确认拆分与代码修改边界。ready-for-agent 表示规格可交付，并不表示阻塞项已完成或本次已开始实现；按 Blocked by 的完成状态取票。

## Implementation record — 2026-09-09

自动化、局部浏览器与 Windows 主窗口实机验收已完成；完整跨平台验收尚未完成，不能关闭本票。

验证：UI 103/103、相关 Rust 106/106、类型检查 0 errors / 0 warnings、生产构建通过。详细证据、代码边界和未验证项见[验收记录](../../../docs/plans/2026-09-09-paper-ui-05-12-validation.md)。未勾选的综合验收项不视为通过。
