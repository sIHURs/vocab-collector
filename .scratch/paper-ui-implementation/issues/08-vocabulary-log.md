# 08: 交付真实 Vocabulary log：保存到每日热力图

**What to build:** 用户成功保存或 Undo 后，在 Insights 看到准确更新的每日热力图；重复词、时区、升级历史和永久删除均遵循 ADR 0005。

**Blocked by:** 02: 交付 Manual Capture 保存与 Undo; 07: 交付 Insights 已有指标与历史覆盖状态.

**Status:** implemented-pending-native-verification

## Acceptance criteria

- [x] 端到端贯通持久化、共享应用逻辑、桌面接口、真实/演示 Backend 与 Insights；在接图前完成数据契约测试，不单独交付只有数据库表的横向任务。
- [x] 覆盖 Manual Capture、各平台既有 Native Capture 保存与 Achieved recapture；每次实际成功保存计一次，失败、Review、翻译预览不计。
- [x] 按保存时系统本地日期固定归属；成功 Undo 原子扣回原日期一次，重复或失败 Undo 不重复扣减；任何写入失败不产生明细与计数漂移。
- [x] 长期保留匿名逐日计数，Achieve/Unachieve/purge 不减少；不得额外保留已删除的词条/上下文。
- [x] 迁移前不可靠日期为未知，升级当日无完整证据则 partial，之后首个完整本地日起可声明完整；未知不能当零，部分期间总数不得标为完整精确总量。
- [x] 呈现过去 12 个月、Sunday-first 日历、蓝色五档 0/1–2/3–5/6–9/10+；未来/区间外日期留白；精确日期/次数 tooltip 与可访问文本。
- [x] 单 tab stop、方向键按日/周移动、Escape 关闭 tooltip；最小窗口保持字号/格子尺寸，水平滚动且初始可见最新日期。
- [x] 保存/Undo 后读取真实更新值，重启后保留；当前日期跨日时刷新窗口范围与覆盖状态。
- [x] Rust 与界面集成测试覆盖事务回滚、重复保存/Undo、跨日/夏令时/时区变化、迁移重启、recapture 与 purge；完整/部分/未知/loading/error 状态均可演示。


## Code changing boundary

允许端到端修改每日统计相关的 domain 查询模型/存储契约、application 编排、SQLite 迁移和事务、桌面查询注册、前端类型/真实与 DemoBackend、热力图及测试。所有核心层变更必须直接服务 ADR 0005；不改 Review、归一化、翻译、Achieve 资格/清理期限。UI 不能独立加减计数；保留原日期的关联不得导致永久删除后保留词条或上下文。

本票受总规格的 Code changing boundary 约束；实施前读取总规格。仅做当前切片所需的局部提取和依赖接入，保持其他工作流可运行。越界需求须单独说明原因、影响与范围调整，不能隐含在 UI 迁移内完成。

## Verification

- [x] 盘点本切片现有状态/命令/测试并保留行为断言；验证适用的正常、加载、空、失败、恢复状态。
- [x] 运行相关 UI 测试及类型检查、构建；涉及 Rust 时运行对应契约/存储/应用测试。记录结果，区分既有失败与新增回归。
- [ ] 对照对应 Paper 页面验证 Light/Dark、适用窗口尺寸、150% 文本、多语言、键盘和焦点；原生行为须有对应平台实机证据，未测项不得勾选通过。

## Planning record

用户于 2026-09-09 确认拆分与代码修改边界。ready-for-agent 表示规格可交付，并不表示阻塞项已完成或本次已开始实现；按 Blocked by 的完成状态取票。

## Implementation record — 2026-09-09

代码及自动化验证已完成；综合实机验收仍待完成。 各捕获来源的保存统计通过共享应用/存储契约验证；这不代表所有平台的原生捕获均已实测。

验证：UI 103/103、相关 Rust 106/106、类型检查 0 errors / 0 warnings、生产构建通过。详细证据、代码边界和未验证项见[验收记录](../../../docs/plans/2026-09-09-paper-ui-05-12-validation.md)。未勾选的综合验收项不视为通过。
