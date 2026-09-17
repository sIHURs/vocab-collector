# 04: 优化词汇列表的读取与交互

**What to build:** 用户打开、搜索、排序和翻页浏览词汇时，减少每个 Vocabulary Item 引起的重复读取，保持列表含义一致。

**Blocked by:** 01：建立数据库性能基线

**Status:** implemented-pending-acceptance

- [ ] 用批量查询或聚合得到 Encounter 次数、最近遇到时间与显示译文；一次列表请求避免为每个词重复读取设置和完整 Encounter。
- [ ] 验证 Learning/Mastered/Paused、Achieved 排除、软删除记录、多语言译文选择和最近遇到时间的行为；不盲目固化有 Encounter 才展示的条件，兼容 Starter Vocabulary Item。
- [ ] 用查询计划和相同数据集验证改进，记录数据库及端到端耗时，达到 01 冻结的正常规模目标。
- [ ] 先保留现有 API 与交互语义；只有批量读取后仍未达到目标才引入后端分页，并记录触发证据。
- [ ] 如引入后端分页，同时覆盖搜索、排序、稳定次级排序键、总数、空结果和页边界，防止只对当前页搜索或重复遗漏词汇。
- [ ] 正确性及必要接口/界面回归随本票进入 CI；不依赖 03 才开始，不机械添加连接池或 WAL。

## Implementation record

主要实现已提交；验收清单保留供后续逐项核验，不代表全部完成。性能结果、测试记录和未验证边界见 [优化报告](../../../docs/database-p1-optimization-report.md)。Ticket 02 的压力规模并发保存等待仍需收尾；原生端到端及安装生命周期验证尚未执行。
