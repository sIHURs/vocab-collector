# 01: 建立数据库性能基线

**What to build:** 开发者可以用可重复的合成数据测量词汇操作，获得正常规模的验收门槛和大规模退化报告。

**Blocked by:** None (can start immediately)

**Status:** implemented-pending-acceptance

- [ ] 提供固定种子的文件数据库生成方式，正常规模为 1 万 Vocabulary Item / 10 万 Encounter，压力规模为 10 万 Vocabulary Item / 100 万 Encounter；覆盖多语言翻译、Learning/Mastered/Paused、Achieved 及 Review 场景。
- [ ] 只使用临时或明确隔离的合成数据库，不读取、修改或复制真实 guest.db。
- [ ] 在当前 Windows 开发机测量 release 构建，记录提交、机器、SQLite 版本、数据规模、重复次数、首次和重复操作耗时。
- [ ] 分别测量保存、打开列表、搜索、切页的数据库查询和端到端时间；记录中位数和尾部延迟，核对结果数量与语义。
- [ ] 对快速检查建立存储层可运行的测量场景；Settings 功能完成后补测端到端耗时，不以尚未存在的 UI 阻塞本票。
- [ ] 依据基线明确正常规模的验收门槛和检查运行预算，写入报告后冻结；压力规模只报告退化，不承诺同样速度。
- [ ] 提供可重复执行说明与小规模夹具正确性验证；大规模性能测试按需执行，不将共享 CI runner 的噪声作为毫秒级阻塞条件。

## Implementation record

主要实现已提交；验收清单保留供后续逐项核验，不代表全部完成。性能结果、测试记录和未验证边界见 [优化报告](../../../docs/database-p1-optimization-report.md)。Ticket 02 的压力规模并发保存等待仍需收尾；原生端到端及安装生命周期验证尚未执行。
