# 07: 交付 Insights 已有指标与历史覆盖状态

**What to build:** 用户打开新版 Insights 查看真实 lifetime 与当前 due 指标，并明确区分缺失数据、未完成历史和没有 session insight。

**Blocked by:** 01: 交付新版 Settings、应用导航与主题.

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] Captured 使用 lifetime Vocabulary，Reviewed 使用 lifetime Reviews，Due 使用 Today backlog；不改为每日保存计数。
- [x] 展示既有 lifetime Encounter/rating totals、Currently achieved 和 incomplete-rating-history 解释；永久删除后的匿名总数语义保留。
- [x] ReviewSessionInsight 仅在现有 session object 可用时展示；不添加持久推荐或伪造 session。
- [x] Vocabulary log 区域按 Paper 显示明确 unavailable，保留布局；不使用样本格子或硬编码历史。
- [x] GlobalInsight 缺失与零值不同；loading、partial、error 与原页面刷新恢复可验证。
- [ ] 在最小窗口和 150% 下可滚动访问所有指标，读屏可理解数字与状态。


## Code changing boundary

允许修改 Insights 页面和已有指标展示/缺失状态及测试。使用既有全局指标与 session insight；不改统计公式、不增加持久推荐或每日存储，本票热力图仍为明确 unavailable。

本票受总规格的 Code changing boundary 约束；实施前读取总规格。仅做当前切片所需的局部提取和依赖接入，保持其他工作流可运行。越界需求须单独说明原因、影响与范围调整，不能隐含在 UI 迁移内完成。

## Verification

- [x] 盘点本切片现有状态/命令/测试并保留行为断言；验证适用的正常、加载、空、失败、恢复状态。
- [x] 运行相关 UI 测试及类型检查、构建；涉及 Rust 时运行对应契约/存储/应用测试。记录结果，区分既有失败与新增回归。
- [ ] 对照对应 Paper 页面验证 Light/Dark、适用窗口尺寸、150% 文本、多语言、键盘和焦点；原生行为须有对应平台实机证据，未测项不得勾选通过。

## Planning record

用户于 2026-09-09 确认拆分与代码修改边界。ready-for-agent 表示规格可交付，并不表示阻塞项已完成或本次已开始实现；按 Blocked by 的完成状态取票。

## Implementation record — 2026-09-09

代码及自动化验证已完成；综合实机验收仍待完成。 第 4 项 unavailable 占位已由依赖票 08 的真实日志替换；读屏实测仍待完成。

验证：UI 103/103、相关 Rust 106/106、类型检查 0 errors / 0 warnings、生产构建通过。详细证据、代码边界和未验证项见[验收记录](../../../docs/plans/2026-09-09-paper-ui-05-12-validation.md)。未勾选的综合验收项不视为通过。
