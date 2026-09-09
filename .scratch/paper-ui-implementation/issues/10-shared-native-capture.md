# 10: 交付 Shared Native Capture 新界面

**What to build:** Shared 平台使用同一视觉系统完成既有 Selection/OCR 捕获与 Undo，保留其现有自动保存和权限流程。

**Blocked by:** 02: 交付 Manual Capture 保存与 Undo; 04: 交付 Achieved 生命周期管理; 06: 交付完整 Focused Review 与完成页; 08: 交付真实 Vocabulary log：保存到每日热力图.

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] 保留 Selection 翻译成功后自动保存；OCR 为只读建议与 Use this text 后同一流程；显式记录与 ADR 0003 的既有冲突，不在此改行为。
- [x] typed translation_unavailable/failed 才显示既有 Retry 或 Save without translation，generic diagnostics 不新增动作。
- [x] 保留 Accessibility/Screen Recording typed 权限命令，以及 capability 允许时的 Region OCR 恢复；不复制 Windows 独有 Achieved UI。
- [x] Saved 内容、Undo、一般失败替换结果、4 秒 timeout、hover 暂停及 request ID 防护保持原样；不增加当前没有的 Escape 处理器。
- [ ] 380×280、150% 文本、两主题与 reduced motion 下动作可达；对应平台实机验证权限、窗口行为以及保存→08 的统计更新。


## Code changing boundary

允许修改 Shared Capture 呈现、复用视觉组件、主题接入与测试。保留既有自动保存、typed 权限恢复、hover timeout 和 Undo 分支；不复制 Windows 工作流、不改 Swift/平台适配器、不顺带补齐 Shared 主窗口。

本票受总规格的 Code changing boundary 约束；实施前读取总规格。仅做当前切片所需的局部提取和依赖接入，保持其他工作流可运行。越界需求须单独说明原因、影响与范围调整，不能隐含在 UI 迁移内完成。

## Verification

- [x] 盘点本切片现有状态/命令/测试并保留行为断言；验证适用的正常、加载、空、失败、恢复状态。
- [x] 运行相关 UI 测试及类型检查、构建；涉及 Rust 时运行对应契约/存储/应用测试。记录结果，区分既有失败与新增回归。
- [ ] 对照对应 Paper 页面验证 Light/Dark、适用窗口尺寸、150% 文本、多语言、键盘和焦点；原生行为须有对应平台实机证据，未测项不得勾选通过。

## Planning record

用户于 2026-09-09 确认拆分与代码修改边界。ready-for-agent 表示规格可交付，并不表示阻塞项已完成或本次已开始实现；按 Blocked by 的完成状态取票。

## Implementation record — 2026-09-09

代码及自动化验证已完成；综合实机验收仍待完成。 Shared 继续自动保存、只读 OCR 建议；与 ADR 0003 编辑确认后另行保存的既有冲突按批准边界保留。

验证：UI 103/103、相关 Rust 106/106、类型检查 0 errors / 0 warnings、生产构建通过。详细证据、代码边界和未验证项见[验收记录](../../../docs/plans/2026-09-09-paper-ui-05-12-validation.md)。未勾选的综合验收项不视为通过。
