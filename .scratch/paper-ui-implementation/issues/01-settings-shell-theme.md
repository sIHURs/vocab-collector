# 01: 交付新版 Settings、应用导航与主题

**What to build:** 在 Windows 中通过新版导航打开完整 Settings，修改并保存现有设置；成功保存的外观同步到适用窗口，失败保留已应用外观。

**Blocked by:** None (can start immediately).

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] 先盘点本切片现有行为与测试；仅做切片所需的局部整理，再接入最小 shadcn-svelte 基础，保持其他页面可运行。
- [x] 落实语义 Light/Dark/System tokens、固定导航顺序、品牌、平台安全标题/拖动区域；不用页面硬编码配色。
- [x] Languages、Review、Capture、Appearance 的现有字段、范围、快捷键输入方式、保存时机和错误路径完整保留；不把 Shared 的 ShortcutRecorder 行为新增到 Windows。
- [x] 设置保存成功后才应用主题和 reduced motion；持久化成功但页面刷新失败仍保留新偏好。
- [x] 主窗口与 Capture 的已保存外观初始化/同步可验证；System 跟随系统，失败不广播草稿；Shared 仅接入可复用外观基础，不补齐 Windows 独有业务。
- [ ] 正常、最小窗口、150% 文本、键盘、Forced Colors、语言长文本验证；现有测试和类型检查/构建通过。

## Code changing boundary

允许修改 Windows Settings/导航呈现、共享基础控件、语义主题、必要构建配置及主窗口/Capture 的外观初始化。桌面层仅在复用现有机制不足时补充保存成功后的设置通知。保留设置字段语义、平台快捷键方式、窗口选择与激活行为；不改 Rust 领域规则或数据库结构。

本票受总规格的 Code changing boundary 约束；实施前读取总规格。仅做当前切片所需的局部提取和依赖接入，保持其他工作流可运行。越界需求须单独说明原因、影响与范围调整，不能隐含在 UI 迁移内完成。

## Verification

- [x] 盘点本切片现有状态/命令/测试并保留行为断言；验证适用的正常、加载、空、失败、恢复状态。
- [x] 运行相关 UI 测试及类型检查、构建；涉及 Rust 时运行对应契约/存储/应用测试。记录结果，区分既有失败与新增回归。
- [ ] 对照对应 Paper 页面验证 Light/Dark、适用窗口尺寸、150% 文本、多语言、键盘和焦点；原生行为须有对应平台实机证据，未测项不得勾选通过。

## Planning record

用户于 2026-09-09 确认拆分与代码修改边界。ready-for-agent 表示规格可交付，并不表示阻塞项已完成或本次已开始实现；按 Blocked by 的完成状态取票。

## Implementation record — 2026-09-09

本轮用户范围为 01–04。代码与自动化验证已完成；原生验收未全部完成，故未标记 fully verified。详见 [实施与验收记录](../../../docs/plans/2026-09-09-paper-ui-01-04-validation.md)。原生键盘自动化因检测到窗口用户输入而停止；读屏、OS Forced Colors 和真实跨 WebView 同步仍待实机验证。
