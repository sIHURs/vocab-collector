# 03: 交付 Vocabulary 查询与 Encounter 详情

**What to build:** 用户在 Active/Mastered 中搜索、分页并打开新版右侧详情，近期记录和 Review contexts 可复用同一详情展示。

**Blocked by:** 01: 交付新版 Settings、应用导航与主题.

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] Table 提供 Word、Translation、Status、Encounters、Last Seen；Active 保留 Learning/Paused，Mastered 独立。
- [x] 保留 search trim/case-fold 规则、视图切换清空搜索和回到第一页、每页 10 项与页码 clamp/floor；不改变尚未迁移的 Achieved 路径。
- [x] 详情仍先请求数据再打开 Sheet，按 Word/Translation/Status/Encounter Timeline 展示；保留原 Encounter 次序与完整来源文本。
- [x] Sheet 的打开、关闭、Escape、焦点陷阱/返回及失败页面错误符合既有行为；不新增编辑、删除或来源链接操作。
- [x] Mastered 既有 Achieve 动作保持可用，可暂沿用原确认与处理器，交由 04 完成视觉迁移。
- [x] 加载、空库、无搜索结果、请求失败及长中英德文本均可验收；正常/最小窗口与 150% 保持身份和动作可达。

## Code changing boundary

允许修改 Vocabulary 表格、搜索/分页呈现、右侧 Sheet 和 Encounter 展示及测试。保留既有查询规则、详情请求顺序与来源数据，不新增后台查询协议或词条编辑能力。

本票受总规格的 Code changing boundary 约束；实施前读取总规格。仅做当前切片所需的局部提取和依赖接入，保持其他工作流可运行。越界需求须单独说明原因、影响与范围调整，不能隐含在 UI 迁移内完成。

## Verification

- [x] 盘点本切片现有状态/命令/测试并保留行为断言；验证适用的正常、加载、空、失败、恢复状态。
- [x] 运行相关 UI 测试及类型检查、构建；涉及 Rust 时运行对应契约/存储/应用测试。记录结果，区分既有失败与新增回归。
- [ ] 对照对应 Paper 页面验证 Light/Dark、适用窗口尺寸、150% 文本、多语言、键盘和焦点；原生行为须有对应平台实机证据，未测项不得勾选通过。

## Planning record

用户于 2026-09-09 确认拆分与代码修改边界。ready-for-agent 表示规格可交付，并不表示阻塞项已完成或本次已开始实现；按 Blocked by 的完成状态取票。

## Implementation record — 2026-09-09

本轮用户范围为 01–04。代码与自动化验证已完成；原生验收未全部完成，故未标记 fully verified。详见 [实施与验收记录](../../../docs/plans/2026-09-09-paper-ui-01-04-validation.md)。原生键盘自动化因检测到窗口用户输入而停止；读屏、OS Forced Colors 和真实跨 WebView 同步仍待实机验证。
